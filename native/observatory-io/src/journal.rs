use anyhow::{Context, Result};
use observatory_domain::JournalEvent;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File, Metadata},
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, SystemTime},
};

const AUXILIARY_FILES: &[&str] = &[
    "Status.json",
    "Cargo.json",
    "NavRoute.json",
    "Market.json",
    "Outfitting.json",
    "Shipyard.json",
    "Backpack.json",
    "FCMaterials.json",
    "ModulesInfo.json",
    "ShipLocker.json",
];
const ANCHOR_BYTES: usize = 128;
// Bound live work before returning control to the worker command loop. Replay
// streams each event directly and therefore does not use these batch limits.
const MAX_SCAN_RECORDS: usize = 512;
const MAX_SCAN_BYTES: usize = 4 * 1024 * 1024;
const MAX_RECORD_BYTES: usize = 16 * 1024 * 1024;

struct ReadBudget {
    records: usize,
    bytes: usize,
}
impl ReadBudget {
    fn exhausted(&self) -> bool {
        self.records == 0 || self.bytes == 0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadIssue {
    pub path: PathBuf,
    /// One-based journal line, or zero for a directory/whole-file error.
    pub line: u64,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct AuxiliaryUpdate {
    /// Original game filename, including `.json`.
    pub name: String,
    /// All fields, including game additions unknown to this build, are preserved.
    pub data: Value,
}

#[derive(Default, Debug)]
pub struct ScanBatch {
    pub events: Vec<JournalEvent>,
    pub updates: Vec<AuxiliaryUpdate>,
    pub errors: Vec<ReadIssue>,
}

#[derive(Debug)]
pub enum ReplayItem {
    Event(JournalEvent),
    Error(ReadIssue),
    Progress {
        path: PathBuf,
        completed: usize,
        total: usize,
    },
}

#[derive(Clone, Debug, Default)]
pub struct ReplaySummary {
    pub files: usize,
    pub events: u64,
    pub errors: u64,
    pub cancelled: bool,
}

/// A journal cursor is independent of the currently newest file. Rotation,
/// appends to an older journal, and several new files between polls are all read.
/// A partial UTF-8 sequence or JSON line is retained until its newline arrives.
pub struct JournalTailer {
    directory: PathBuf,
    cursors: BTreeMap<PathBuf, Cursor>,
    ancillary: BTreeMap<String, AuxiliaryCursor>,
}

impl JournalTailer {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
            cursors: BTreeMap::new(),
            ancillary: BTreeMap::new(),
        }
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    pub fn set_directory(&mut self, directory: impl Into<PathBuf>) {
        self.directory = directory.into();
        self.cursors.clear();
        self.ancillary.clear();
    }

    /// Read new complete lines and the latest valid ancillary snapshots. Repeated
    /// scans without changes produce no events. Errors do not stop other files.
    /// Each call reads at most 512 records / 4 MiB; backlog continues on the next
    /// call without advancing beyond the records delivered to this batch.
    pub fn scan(&mut self, cancel: &AtomicBool) -> ScanBatch {
        let mut batch = ScanBatch::default();
        let files = match journal_files(&self.directory) {
            Ok(files) => files,
            Err(error) => {
                batch.errors.push(ReadIssue {
                    path: self.directory.clone(),
                    line: 0,
                    message: error.to_string(),
                });
                return batch;
            }
        };
        let present: HashSet<_> = files.iter().cloned().collect();
        self.cursors.retain(|path, _| present.contains(path));
        let mut budget = ReadBudget {
            records: MAX_SCAN_RECORDS,
            bytes: MAX_SCAN_BYTES,
        };
        for path in files {
            if cancel.load(Ordering::Relaxed) || budget.exhausted() {
                break;
            }
            let cursor = self.cursors.entry(path.clone()).or_default();
            read_incremental(
                &path,
                cursor,
                false,
                cancel,
                Some(&mut budget),
                |item| match item {
                    Record::Event(event) => batch.events.push(event),
                    Record::Error(issue) => batch.errors.push(issue),
                },
            );
        }
        if !cancel.load(Ordering::Relaxed) {
            self.scan_ancillary(cancel, &mut batch);
        }
        batch
    }

    /// Establish an end-of-file cursor without processing history. Existing
    /// incomplete lines are retained, so a later continuation is still delivered.
    pub fn prime(&mut self) -> Result<()> {
        self.cursors.clear();
        for path in journal_files(&self.directory)? {
            self.cursors.insert(path.clone(), cursor_at_end(&path)?);
        }
        Ok(())
    }

    /// Match Core's startup behavior: inspect at most two recent files, retain
    /// the current system since its last jump plus the latest session headers.
    /// All other journals are primed, preventing history from replaying live.
    pub fn preread(&mut self, cancel: &AtomicBool) -> ScanBatch {
        let mut batch = ScanBatch::default();
        let files = match journal_files(&self.directory) {
            Ok(files) => files,
            Err(error) => {
                batch.errors.push(ReadIssue {
                    path: self.directory.clone(),
                    line: 0,
                    message: error.to_string(),
                });
                return batch;
            }
        };
        self.cursors.clear();
        let recent_start = files.len().saturating_sub(2);
        let mut recent_events = Vec::new();
        let mut headers = Vec::new();
        let mut saw_jump = false;
        for (index, path) in files.iter().enumerate() {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            if index < recent_start {
                match cursor_at_end(path) {
                    Ok(cursor) => {
                        self.cursors.insert(path.clone(), cursor);
                    }
                    Err(error) => batch.errors.push(ReadIssue {
                        path: path.clone(),
                        line: 0,
                        message: error.to_string(),
                    }),
                }
                continue;
            }
            let cursor = self.cursors.entry(path.clone()).or_default();
            read_incremental(path, cursor, false, cancel, None, |item| match item {
                Record::Error(issue) => batch.errors.push(issue),
                Record::Event(event) => {
                    let jump = event.event == "FSDJump"
                        || (event.event == "CarrierJump"
                            && (event.boolean("Docked") || event.boolean("OnFoot")));
                    if jump {
                        recent_events.clear();
                        saw_jump = true;
                    } else if event.event == "Fileheader" {
                        if !saw_jump {
                            recent_events.clear();
                        }
                        headers.clear();
                        headers.push(event.clone());
                    } else if matches!(
                        event.event.as_str(),
                        "LoadGame" | "Statistics" | "CarrierLocation" | "Loadout"
                    ) {
                        headers.push(event.clone());
                    }
                    recent_events.push(event);
                }
            });
        }
        if saw_jump {
            // Header records may already occur after the final jump (for example
            // a new file was opened in this system). Deduplicate by full content.
            headers.retain(|header| !recent_events.iter().any(|event| event.data == header.data));
            headers.extend(recent_events);
            batch.events = headers;
        } else {
            batch.events = recent_events;
        }
        batch
    }

    /// Stream a chronological historical replay without loading the entire
    /// journal collection in memory. The next live scan continues at these exact
    /// cursors; callers suppress audio/popup effects while handling this callback.
    pub fn replay_all<F>(&mut self, cancel: &AtomicBool, mut emit: F) -> Result<ReplaySummary>
    where
        F: FnMut(ReplayItem),
    {
        let files = journal_files(&self.directory)?;
        self.cursors.clear();
        // Keep untouched history at EOF if replay is cancelled part way through.
        // Otherwise the next live scan would treat every unread old file as new.
        for path in &files {
            if let Ok(cursor) = cursor_at_end(path) {
                self.cursors.insert(path.clone(), cursor);
            }
        }
        let replay_boundaries = self.cursors.clone();
        let mut summary = ReplaySummary::default();
        let total = files.len();
        for path in files {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            emit(ReplayItem::Progress {
                path: path.clone(),
                completed: summary.files,
                total,
            });
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            let cursor = self.cursors.entry(path.clone()).or_default();
            *cursor = Cursor::default();
            read_incremental(&path, cursor, true, cancel, None, |item| match item {
                Record::Event(event) => {
                    summary.events += 1;
                    emit(ReplayItem::Event(event));
                }
                Record::Error(issue) => {
                    summary.errors += 1;
                    emit(ReplayItem::Error(issue));
                }
            });
            if cancel.load(Ordering::Relaxed) {
                // Unread historical lines must not become live notifications
                // when the pre-existing monitor resumes after Cancel. Retain
                // the initial EOF, allowing genuinely new appends through.
                if let Some(boundary) = replay_boundaries.get(&path) {
                    self.cursors.insert(path.clone(), boundary.clone());
                }
                break;
            }
            summary.files += 1;
            emit(ReplayItem::Progress {
                path,
                completed: summary.files,
                total,
            });
        }
        summary.cancelled = cancel.load(Ordering::Relaxed);
        Ok(summary)
    }

    fn scan_ancillary(&mut self, cancel: &AtomicBool, batch: &mut ScanBatch) {
        for &name in AUXILIARY_FILES {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            let path = self.directory.join(name);
            let metadata = match fs::metadata(&path) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    self.ancillary.remove(name);
                    continue;
                }
                Err(error) => {
                    batch.errors.push(ReadIssue {
                        path,
                        line: 0,
                        message: error.to_string(),
                    });
                    continue;
                }
            };
            let stamp = FileStamp::new(&metadata);
            let cursor = self.ancillary.entry(name.to_owned()).or_default();
            if cursor.stamp.as_ref() == Some(&stamp) {
                continue;
            }
            // A game write can truncate the snapshot before replacing its JSON.
            // Retry briefly, then leave its cursor uncommitted for the next scan.
            let mut failure = None;
            for attempt in 0..3 {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                let before = fs::metadata(&path).ok().map(|m| FileStamp::new(&m));
                match fs::read(&path) {
                    Ok(bytes) => match serde_json::from_slice::<Value>(&bytes) {
                        Ok(data) if data.is_object() => {
                            let after = fs::metadata(&path).ok().map(|m| FileStamp::new(&m));
                            if before != after {
                                failure =
                                    Some((bytes, "Snapshot changed while being read".to_owned()));
                                if attempt < 2 {
                                    thread::sleep(Duration::from_millis(5));
                                }
                                continue;
                            }
                            if cursor.last_good.as_ref() != Some(&bytes) {
                                batch.updates.push(AuxiliaryUpdate {
                                    name: name.to_owned(),
                                    data,
                                });
                            }
                            cursor.last_good = Some(bytes);
                            cursor.last_error = None;
                            cursor.stamp = after;
                            failure = None;
                            break;
                        }
                        Ok(_) => {
                            failure = Some((bytes, "Expected a JSON object".to_owned()));
                        }
                        Err(error) => {
                            failure = Some((bytes, error.to_string()));
                        }
                    },
                    Err(error) => {
                        failure = Some((Vec::new(), error.to_string()));
                    }
                }
                if attempt < 2 {
                    thread::sleep(Duration::from_millis(5));
                }
            }
            if let Some((bytes, message)) = failure {
                if cursor.last_error.as_ref() != Some(&(bytes.clone(), message.clone())) {
                    batch.errors.push(ReadIssue {
                        path,
                        line: 0,
                        message: message.clone(),
                    });
                }
                cursor.last_error = Some((bytes, message));
            }
        }
    }
}

#[derive(Default)]
struct AuxiliaryCursor {
    stamp: Option<FileStamp>,
    last_good: Option<Vec<u8>>,
    last_error: Option<(Vec<u8>, String)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FileStamp {
    length: u64,
    modified: Option<SystemTime>,
    identity: FileIdentity,
}
impl FileStamp {
    fn new(metadata: &Metadata) -> Self {
        Self {
            length: metadata.len(),
            modified: metadata.modified().ok(),
            identity: file_identity(metadata),
        }
    }
}

#[cfg(unix)]
type FileIdentity = (u64, u64);
#[cfg(not(unix))]
type FileIdentity = Option<SystemTime>;

#[cfg(unix)]
fn file_identity(metadata: &Metadata) -> FileIdentity {
    use std::os::unix::fs::MetadataExt;
    (metadata.dev(), metadata.ino())
}
#[cfg(not(unix))]
fn file_identity(metadata: &Metadata) -> FileIdentity {
    metadata.created().ok()
}

#[derive(Default, Clone)]
struct Cursor {
    offset: u64,
    line: u64,
    pending: Vec<u8>,
    oversized: bool,
    oversized_reported: bool,
    identity: Option<FileIdentity>,
    prefix: Vec<u8>,
    anchor: Vec<u8>,
    stamp: Option<FileStamp>,
}

enum Record {
    Event(JournalEvent),
    Error(ReadIssue),
}

fn read_incremental<F>(
    path: &Path,
    cursor: &mut Cursor,
    finish_last: bool,
    cancel: &AtomicBool,
    budget: Option<&mut ReadBudget>,
    mut emit: F,
) where
    F: FnMut(Record),
{
    let result = read_records(path, cursor, finish_last, cancel, budget, &mut emit);
    if let Err(error) = result {
        emit(Record::Error(ReadIssue {
            path: path.to_owned(),
            line: cursor.line,
            message: error.to_string(),
        }));
    }
}

fn read_records<F>(
    path: &Path,
    cursor: &mut Cursor,
    finish_last: bool,
    cancel: &AtomicBool,
    mut budget: Option<&mut ReadBudget>,
    emit: &mut F,
) -> Result<()>
where
    F: FnMut(Record),
{
    let initial_stamp = FileStamp::new(&fs::metadata(path)?);
    if !finish_last && cursor.stamp.as_ref() == Some(&initial_stamp) {
        return Ok(());
    }
    let mut file = File::open(path).with_context(|| format!("Cannot open {}", path.display()))?;
    let metadata = file.metadata()?;
    let identity = file_identity(&metadata);
    let replaced = cursor
        .identity
        .as_ref()
        .is_some_and(|previous| previous != &identity);
    if replaced || metadata.len() < cursor.offset || !cursor_matches(&mut file, cursor)? {
        *cursor = Cursor::default();
    }
    cursor.identity = Some(identity);
    file.seek(SeekFrom::Start(0))?;
    cursor
        .prefix
        .resize(ANCHOR_BYTES.min(metadata.len() as usize), 0);
    file.read_exact(&mut cursor.prefix)?;
    file.seek(SeekFrom::Start(cursor.offset))?;
    let mut reader = BufReader::new(file);
    while !cancel.load(Ordering::Relaxed) && !budget.as_deref().is_some_and(ReadBudget::exhausted) {
        // fill_buf/consume keeps an incomplete record resumable without reading
        // or allocating a whole giant line before checking cancellation again.
        let available = reader.fill_buf()?;
        if available.is_empty() {
            break;
        }
        let through_newline = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |position| position + 1);
        let count = budget
            .as_deref()
            .map_or(through_newline, |limit| through_newline.min(limit.bytes));
        let bytes = &available[..count];
        let newline = bytes.last() == Some(&b'\n');
        cursor.offset += count as u64;
        update_anchor(&mut cursor.anchor, bytes);
        if !cursor.oversized {
            if cursor.pending.len() + count > MAX_RECORD_BYTES {
                cursor.pending = Vec::new();
                cursor.oversized = true;
                oversized_issue(path, cursor.line + 1, emit);
                cursor.oversized_reported = true;
            } else {
                cursor.pending.extend_from_slice(bytes);
            }
        }
        reader.consume(count);
        if let Some(limit) = budget.as_deref_mut() {
            limit.bytes -= count;
        }
        if newline {
            cursor.line += 1;
            if cursor.oversized {
                if !cursor.oversized_reported {
                    oversized_issue(path, cursor.line, emit);
                }
                cursor.oversized = false;
                cursor.oversized_reported = false;
            } else {
                parse_record(path, cursor.line, &cursor.pending, emit);
            }
            clear_pending(cursor);
            if let Some(limit) = budget.as_deref_mut() {
                limit.records -= 1;
            }
        }
    }
    if finish_last && !cancel.load(Ordering::Relaxed) && !cursor.pending.is_empty() {
        // A read-all may run while the game is halfway through writing its last
        // line. Report that partial record but retain its bytes so the next live
        // scan can finish it. A valid unterminated historical record is usable.
        let parsed = std::str::from_utf8(&cursor.pending)
            .ok()
            .and_then(|text| JournalEvent::parse(text.trim().trim_start_matches('\u{feff}')).ok());
        if let Some(event) = parsed {
            cursor.line += 1;
            emit(Record::Event(event));
            clear_pending(cursor);
        } else {
            parse_record(path, cursor.line + 1, &cursor.pending, emit);
        }
    }
    // Do not reopen unchanged archives every 250 ms. Only cache a stamp when
    // the file stayed stable for the entire read and all its bytes were read;
    // otherwise an append racing EOF must remain visible on the next scan.
    let final_stamp = fs::metadata(path).ok().map(|m| FileStamp::new(&m));
    cursor.stamp =
        final_stamp.filter(|stamp| stamp == &initial_stamp && stamp.length == cursor.offset);
    Ok(())
}

fn parse_record<F>(path: &Path, line: u64, bytes: &[u8], emit: &mut F)
where
    F: FnMut(Record),
{
    let text = match std::str::from_utf8(bytes) {
        Ok(text) => text.trim().trim_start_matches('\u{feff}'),
        Err(error) => {
            emit(Record::Error(ReadIssue {
                path: path.to_owned(),
                line,
                message: error.to_string(),
            }));
            return;
        }
    };
    if text.is_empty() {
        return;
    }
    match JournalEvent::parse(text) {
        Ok(event) => emit(Record::Event(event)),
        Err(error) => emit(Record::Error(ReadIssue {
            path: path.to_owned(),
            line,
            message: error.to_string(),
        })),
    }
}

fn clear_pending(cursor: &mut Cursor) {
    if cursor.pending.capacity() > 64 * 1024 {
        cursor.pending = Vec::new();
    } else {
        cursor.pending.clear();
    }
}

fn oversized_issue<F: FnMut(Record)>(path: &Path, line: u64, emit: &mut F) {
    emit(Record::Error(ReadIssue {
        path: path.to_owned(),
        line,
        message: format!("Journal record exceeds the {MAX_RECORD_BYTES} byte limit"),
    }));
}

fn update_anchor(anchor: &mut Vec<u8>, bytes: &[u8]) {
    if bytes.len() >= ANCHOR_BYTES {
        anchor.clear();
        anchor.extend_from_slice(&bytes[bytes.len() - ANCHOR_BYTES..]);
    } else {
        let discard = (anchor.len() + bytes.len()).saturating_sub(ANCHOR_BYTES);
        anchor.drain(..discard);
        anchor.extend_from_slice(bytes);
    }
}

fn cursor_matches(file: &mut File, cursor: &Cursor) -> Result<bool> {
    if cursor.offset == 0 {
        return Ok(true);
    }
    for (position, expected) in [
        (0, &cursor.prefix),
        (
            cursor.offset.saturating_sub(cursor.anchor.len() as u64),
            &cursor.anchor,
        ),
    ] {
        file.seek(SeekFrom::Start(position))?;
        let mut actual = vec![0; expected.len()];
        if file.read_exact(&mut actual).is_err() || actual != *expected {
            return Ok(false);
        }
    }
    Ok(true)
}

fn cursor_at_end(path: &Path) -> Result<Cursor> {
    let mut file = File::open(path)?;
    let metadata = file.metadata()?;
    let offset = metadata.len();
    let mut cursor = Cursor {
        offset,
        identity: Some(file_identity(&metadata)),
        stamp: Some(FileStamp::new(&metadata)),
        ..Cursor::default()
    };
    cursor.prefix.resize(ANCHOR_BYTES.min(offset as usize), 0);
    file.read_exact(&mut cursor.prefix)?;
    // Find the beginning of a possible incomplete final record without reading
    // the complete historical journal into memory.
    let mut position = offset;
    let mut suffix_parts = Vec::new();
    let mut suffix_len = 0;
    while position > 0 {
        let count = position.min(8192) as usize;
        position -= count as u64;
        file.seek(SeekFrom::Start(position))?;
        let mut block = vec![0; count];
        file.read_exact(&mut block)?;
        if let Some(newline) = block.iter().rposition(|byte| *byte == b'\n') {
            let pending = block.split_off(newline + 1);
            suffix_len += pending.len();
            suffix_parts.push(pending);
            break;
        }
        suffix_len += block.len();
        if suffix_len > MAX_RECORD_BYTES {
            cursor.oversized = true;
            suffix_parts.clear();
            suffix_len = 0;
            break;
        }
        suffix_parts.push(block);
    }
    if suffix_len > MAX_RECORD_BYTES {
        cursor.oversized = true;
    } else {
        cursor.pending = Vec::with_capacity(suffix_len);
        for part in suffix_parts.into_iter().rev() {
            cursor.pending.extend(part);
        }
    }
    let anchor_count = ANCHOR_BYTES.min(offset as usize);
    cursor.anchor.resize(anchor_count, 0);
    file.seek(SeekFrom::Start(offset - anchor_count as u64))?;
    file.read_exact(&mut cursor.anchor)?;
    Ok(cursor)
}

/// Sort old `Journal.yyMMddHHmmss.01.log` and modern
/// `Journal.yyyy-MM-ddTHHmmss.01.log` names on a common timestamp key.
pub fn journal_files(directory: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory)
        .with_context(|| format!("Cannot read journal directory {}", directory.display()))?
    {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("Journal.") && name.ends_with(".log") && entry.file_type()?.is_file() {
            files.push(entry.path());
        }
    }
    files.sort_by_cached_key(|path| journal_sort_key(path));
    Ok(files)
}

fn journal_sort_key(path: &Path) -> (String, String) {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let date = name
        .strip_prefix("Journal.")
        .and_then(|text| text.rsplit_once('.').map(|(stem, _)| stem))
        .and_then(|stem| stem.rsplit_once('.').map(|(date, _)| date))
        .unwrap_or("");
    let digits: String = date.chars().filter(char::is_ascii_digit).collect();
    let date = match digits.len() {
        12 => format!("20{digits}"),
        14 => digits,
        _ => {
            // Unknown game filename formats retain deterministic mtime ordering.
            let seconds = fs::metadata(path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|time| time.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs())
                .unwrap_or(0);
            return (format!("~{seconds:020}"), name.into_owned());
        }
    };
    (date, name.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn event(name: &str) -> String {
        format!("{{\"timestamp\":\"2026-10-25T00:00:00Z\",\"event\":\"{name}\"}}\n")
    }
    fn append(path: &Path, bytes: &[u8]) {
        let mut file = fs::OpenOptions::new().append(true).open(path).unwrap();
        file.write_all(bytes).unwrap();
    }

    #[test]
    fn tail_retains_split_utf8_and_json_and_never_repeats_lines() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Journal.2026-10-25T000000.01.log");
        let full =
            "{\"timestamp\":\"2026-10-25T00:00:00Z\",\"event\":\"Test\",\"BodyName\":\"星\"}\n";
        let split = full.as_bytes().iter().position(|b| *b == 0xe6).unwrap() + 1;
        fs::write(&path, &full.as_bytes()[..split]).unwrap();
        let cancel = AtomicBool::new(false);
        let mut tailer = JournalTailer::new(dir.path());
        assert!(tailer.scan(&cancel).events.is_empty());
        append(&path, &full.as_bytes()[split..]);
        let batch = tailer.scan(&cancel);
        assert_eq!(batch.events.len(), 1);
        assert_eq!(batch.events[0].str("BodyName"), "星");
        assert!(batch.errors.is_empty());
        assert!(tailer.scan(&cancel).events.is_empty());
        append(&path, event("Next").as_bytes());
        assert_eq!(tailer.scan(&cancel).events[0].event, "Next");
    }

    #[test]
    fn rotation_truncation_and_replacement_are_observed() {
        let dir = tempfile::tempdir().unwrap();
        let first = dir.path().join("Journal.2026-10-25T000000.01.log");
        fs::write(&first, event("Initial")).unwrap();
        let cancel = AtomicBool::new(false);
        let mut tailer = JournalTailer::new(dir.path());
        assert_eq!(tailer.scan(&cancel).events.len(), 1);
        let second = dir.path().join("Journal.2026-10-25T010000.01.log");
        fs::write(&second, event("Rotated")).unwrap();
        append(&first, event("OlderAppend").as_bytes());
        assert_eq!(
            tailer
                .scan(&cancel)
                .events
                .iter()
                .map(|e| e.event.as_str())
                .collect::<Vec<_>>(),
            ["OlderAppend", "Rotated"]
        );
        fs::write(&second, event("Rewrite")).unwrap();
        assert_eq!(tailer.scan(&cancel).events[0].event, "Rewrite");
        fs::remove_file(&first).unwrap();
        fs::write(&first, event("Replacement")).unwrap();
        assert_eq!(tailer.scan(&cancel).events[0].event, "Replacement");
        assert!(tailer.scan(&cancel).events.is_empty());
    }

    #[test]
    fn malformed_record_does_not_block_later_events() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Journal.2026-10-25T000000.01.log"),
            format!("broken\n{}", event("Valid")),
        )
        .unwrap();
        let batch = JournalTailer::new(dir.path()).scan(&AtomicBool::new(false));
        assert_eq!(batch.errors.len(), 1);
        assert_eq!(batch.errors[0].line, 1);
        assert_eq!(batch.events[0].event, "Valid");
    }

    #[test]
    fn ancillary_partial_writes_retry_and_all_fields_survive() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Status.json");
        fs::write(&path, "{\"Flags\":").unwrap();
        let cancel = AtomicBool::new(false);
        let mut tailer = JournalTailer::new(dir.path());
        assert_eq!(tailer.scan(&cancel).errors.len(), 1);
        assert!(tailer.scan(&cancel).errors.is_empty());
        append(&path, b"42,\"FutureField\":true}");
        let batch = tailer.scan(&cancel);
        assert_eq!(batch.updates.len(), 1);
        assert_eq!(batch.updates[0].data["FutureField"], true);
        assert!(tailer.scan(&cancel).updates.is_empty());
    }

    #[test]
    fn replay_orders_old_and_new_names_and_live_continues_without_duplicates() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Journal.240101000000.01.log"), event("Old")).unwrap();
        let latest = dir.path().join("Journal.2026-10-25T000000.01.log");
        fs::write(&latest, event("New").trim_end()).unwrap();
        let cancel = AtomicBool::new(false);
        let mut tailer = JournalTailer::new(dir.path());
        let mut names = Vec::new();
        let summary = tailer
            .replay_all(&cancel, |item| {
                if let ReplayItem::Event(event) = item {
                    names.push(event.event);
                }
            })
            .unwrap();
        assert_eq!(names, ["Old", "New"]);
        assert_eq!(summary.events, 2);
        append(&latest, format!("\n{}", event("Live")).as_bytes());
        let batch = tailer.scan(&cancel);
        assert_eq!(batch.events.len(), 1);
        assert_eq!(batch.events[0].event, "Live");
    }

    #[test]
    fn prime_preserves_preexisting_partial_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Journal.2026-10-25T000000.01.log");
        fs::write(
            &path,
            format!(
                "{}{{\"timestamp\":\"2026-10-25T00:00:00Z\",\"event\":\"",
                event("Past")
            ),
        )
        .unwrap();
        let mut tailer = JournalTailer::new(dir.path());
        tailer.prime().unwrap();
        append(&path, b"Live\"}\n");
        let batch = tailer.scan(&AtomicBool::new(false));
        assert_eq!(batch.events.len(), 1);
        assert_eq!(batch.events[0].event, "Live");
    }

    #[test]
    fn preread_collects_session_headers_and_only_last_system() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Journal.2026-10-25T000000.01.log");
        fs::write(
            &path,
            [
                event("Fileheader"),
                event("LoadGame"),
                event("OldScan"),
                event("FSDJump"),
                event("Scan"),
            ]
            .concat(),
        )
        .unwrap();
        let cancel = AtomicBool::new(false);
        let mut tailer = JournalTailer::new(dir.path());
        let batch = tailer.preread(&cancel);
        assert_eq!(
            batch
                .events
                .iter()
                .map(|e| e.event.as_str())
                .collect::<Vec<_>>(),
            ["Fileheader", "LoadGame", "FSDJump", "Scan"]
        );
        assert!(tailer.scan(&cancel).events.is_empty());
    }

    #[test]
    fn cancelling_replay_does_not_import_unvisited_history_as_live() {
        let dir = tempfile::tempdir().unwrap();
        let first = dir.path().join("Journal.2026-10-25T000000.01.log");
        let second = dir.path().join("Journal.2026-10-25T010000.01.log");
        fs::write(&first, event("First")).unwrap();
        fs::write(&second, event("UnvisitedHistory")).unwrap();
        let cancel = AtomicBool::new(false);
        let mut tailer = JournalTailer::new(dir.path());
        let summary = tailer
            .replay_all(&cancel, |item| {
                if matches!(item, ReplayItem::Event(_)) {
                    cancel.store(true, Ordering::Relaxed);
                }
            })
            .unwrap();
        assert!(summary.cancelled);
        cancel.store(false, Ordering::Relaxed);
        assert!(tailer.scan(&cancel).events.is_empty());
        append(&second, event("NewLive").as_bytes());
        assert_eq!(tailer.scan(&cancel).events[0].event, "NewLive");
    }

    #[test]
    fn replay_keeps_incomplete_final_json_for_live_continuation() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Journal.2026-10-25T000000.01.log");
        let partial = "{\"timestamp\":\"2026-10-25T00:00:00Z\",\"event\":\"Par";
        fs::write(&path, format!("{}{partial}", event("History"))).unwrap();
        let cancel = AtomicBool::new(false);
        let mut tailer = JournalTailer::new(dir.path());
        let summary = tailer.replay_all(&cancel, |_| {}).unwrap();
        assert_eq!(summary.events, 1);
        assert_eq!(summary.errors, 1);
        append(&path, b"tial\"}\n");
        let batch = tailer.scan(&cancel);
        assert_eq!(batch.events.len(), 1);
        assert_eq!(batch.events[0].event, "Partial");
        assert!(batch.errors.is_empty());
    }

    #[test]
    fn cancelling_within_file_does_not_deliver_unread_history_as_live_events() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Journal.2026-10-25T000000.01.log");
        fs::write(
            &path,
            format!("{}{}", event("First"), event("UnreadHistorical")),
        )
        .unwrap();
        let cancel = AtomicBool::new(false);
        let mut tailer = JournalTailer::new(dir.path());
        let mut replayed = Vec::new();
        let summary = tailer
            .replay_all(&cancel, |item| {
                if let ReplayItem::Event(event) = item {
                    replayed.push(event.event);
                    cancel.store(true, Ordering::Relaxed);
                }
            })
            .unwrap();
        assert!(summary.cancelled);
        assert_eq!(replayed, vec!["First"]);
        cancel.store(false, Ordering::Relaxed);
        append(&path, event("NewLive").as_bytes());
        let batch = tailer.scan(&cancel);
        assert_eq!(
            batch
                .events
                .iter()
                .map(|e| e.event.as_str())
                .collect::<Vec<_>>(),
            vec!["NewLive"]
        );
    }

    #[test]
    fn stopping_at_replay_progress_preserves_primed_history() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Journal.2026-10-25T000000.01.log"),
            event("Past"),
        )
        .unwrap();
        let cancel = AtomicBool::new(false);
        let mut tailer = JournalTailer::new(dir.path());
        let summary = tailer
            .replay_all(&cancel, |_| cancel.store(true, Ordering::Relaxed))
            .unwrap();
        assert!(summary.cancelled);
        cancel.store(false, Ordering::Relaxed);
        assert!(tailer.scan(&cancel).events.is_empty());
    }

    #[test]
    fn live_backlog_is_bounded_ordered_and_resumes_ten_thousand_records_without_loss() {
        let dir = tempfile::tempdir().unwrap();
        let first = dir.path().join("Journal.241025000000.01.log");
        let second = dir.path().join("Journal.2026-10-25T000000.01.log");
        for (path, range) in [(&first, 0..6000), (&second, 6000..10_000)] {
            let mut file = File::create(path).unwrap();
            for index in range {
                writeln!(file, "{{\"event\":\"Test\",\"Index\":{index}}}").unwrap();
            }
        }
        let cancel = AtomicBool::new(false);
        let mut tailer = JournalTailer::new(dir.path());
        let first_batch = tailer.scan(&cancel);
        assert_eq!(first_batch.events.len(), MAX_SCAN_RECORDS);
        let cursor = &tailer.cursors[&first];
        let position = cursor.offset;
        assert!(position < fs::metadata(&first).unwrap().len());
        assert!(
            cursor.stamp.is_none(),
            "a partial read cannot cache its EOF stamp"
        );
        assert!(
            !tailer.cursors.contains_key(&second),
            "newer journals wait for older backlog"
        );
        cancel.store(true, Ordering::Relaxed);
        assert!(tailer.scan(&cancel).events.is_empty());
        assert_eq!(tailer.cursors[&first].offset, position);
        cancel.store(false, Ordering::Relaxed);
        let mut indices: Vec<_> = first_batch
            .events
            .iter()
            .map(|event| event.id("Index").unwrap())
            .collect();
        for _ in 0..30 {
            let batch = tailer.scan(&cancel);
            assert!(batch.events.len() <= MAX_SCAN_RECORDS);
            assert!(batch.errors.is_empty());
            if batch.events.is_empty() {
                break;
            }
            indices.extend(batch.events.iter().map(|event| event.id("Index").unwrap()));
        }
        assert_eq!(indices, (0..10_000).collect::<Vec<_>>());
        assert!(tailer.cursors[&first].stamp.is_some());
        assert!(tailer.cursors[&second].stamp.is_some());
        assert!(tailer.scan(&cancel).events.is_empty());
    }

    #[test]
    fn byte_budget_preserves_a_large_record_and_releases_its_buffer_after_delivery() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Journal.2026-10-25T000000.01.log");
        let blob = "x".repeat(MAX_SCAN_BYTES + 1024);
        fs::write(
            &path,
            format!("{{\"event\":\"Large\",\"Blob\":\"{blob}\"}}\n"),
        )
        .unwrap();
        let cancel = AtomicBool::new(false);
        let mut tailer = JournalTailer::new(dir.path());
        let first = tailer.scan(&cancel);
        assert!(first.events.is_empty());
        assert_eq!(tailer.cursors[&path].offset, MAX_SCAN_BYTES as u64);
        assert!(tailer.cursors[&path].stamp.is_none());
        assert!(tailer.cursors[&path].anchor.capacity() <= ANCHOR_BYTES);
        let second = tailer.scan(&cancel);
        assert_eq!(second.events.len(), 1);
        assert_eq!(second.events[0].str("Blob"), blob);
        assert!(second.errors.is_empty());
        assert!(tailer.cursors[&path].pending.capacity() <= 64 * 1024);
        assert!(tailer.scan(&cancel).events.is_empty());
    }

    #[test]
    fn oversized_incomplete_record_has_bounded_memory_and_recovers_at_its_newline() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Journal.2026-10-25T000000.01.log");
        fs::write(&path, vec![b'x'; MAX_RECORD_BYTES + 1]).unwrap();
        let cancel = AtomicBool::new(false);
        let mut tailer = JournalTailer::new(dir.path());
        tailer.prime().unwrap();
        assert!(tailer.cursors[&path].oversized);
        assert!(tailer.cursors[&path].pending.is_empty());
        append(&path, format!("\n{}", event("AfterOversize")).as_bytes());
        let batch = tailer.scan(&cancel);
        assert_eq!(batch.errors.len(), 1);
        assert!(batch.errors[0].message.contains("byte limit"));
        assert_eq!(batch.events[0].event, "AfterOversize");
        assert!(tailer.cursors[&path].pending.capacity() <= 64 * 1024);
    }

    #[test]
    fn replay_reports_oversized_unterminated_records_once_then_recovers_live() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Journal.2026-10-25T000000.01.log");
        fs::write(&path, vec![b'x'; MAX_RECORD_BYTES + 1]).unwrap();
        let cancel = AtomicBool::new(false);
        let mut tailer = JournalTailer::new(dir.path());
        let summary = tailer.replay_all(&cancel, |_| {}).unwrap();
        assert_eq!(summary.errors, 1);
        assert_eq!(summary.events, 0);
        assert!(tailer.cursors[&path].pending.is_empty());
        append(&path, format!("\n{}", event("Recovered")).as_bytes());
        let batch = tailer.scan(&cancel);
        assert!(batch.errors.is_empty());
        assert_eq!(batch.events[0].event, "Recovered");
    }
}
