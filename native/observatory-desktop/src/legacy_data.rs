//! Read-only original plugin storage import. SQLite online snapshots include WAL
//! state; every destination is staged and existing native records win conflicts.
use anyhow::{ensure, Context, Result};
use observatory_io::{ConfigPaths, Settings};
use observatory_plugins::bioinsights::{decode_legacy_codex, BioInsights};
use rusqlite::{Connection, DatabaseName, OpenFlags};
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};
const BIO: &str = "4d617474-4701-4249-4f49-4e5349474854";
const EVALUATOR: &str = "4d617474-4702-0045-5641-4c5541544f52";
const HECATE: &str = "4d617474-4707-0000-0000-484543415445";
#[derive(Default, Debug, Serialize)]
pub struct Receipt {
    pub source_root: Option<PathBuf>,
    pub files: Vec<FileReceipt>,
    pub commanders: usize,
    pub codex_entries: usize,
    pub plugin_settings: Value,
    pub backup_directory: Option<PathBuf>,
}
#[derive(Debug, Serialize)]
pub struct FileReceipt {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub rows_added: usize,
    pub previous: Option<PathBuf>,
}
impl Receipt {
    pub fn apply_to(&self, settings: &mut Settings) {
        if let Some(plugins) = self.plugin_settings.as_object() {
            for (name, patch) in plugins {
                merge(&mut settings.plugins[name], patch);
            }
        }
    }
}
fn merge(value: &mut Value, patch: &Value) {
    if let Some(fields) = patch.as_object() {
        if !value.is_object() {
            *value = json!({});
        }
        for (key, field) in fields {
            merge(&mut value[key], field);
        }
    } else {
        *value = patch.clone();
    }
}
fn child(directory: &Path, names: &[&str]) -> Option<PathBuf> {
    fs::read_dir(directory)
        .ok()?
        .filter_map(Result::ok)
        .find(|entry| {
            entry.path().is_dir()
                && names.iter().any(|name| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .eq_ignore_ascii_case(name)
                })
        })
        .map(|entry| entry.path())
}
fn has_plugins(path: &Path) -> bool {
    child(
        path,
        &[
            BIO,
            "ObservatoryBioInsights",
            HECATE,
            "ObservatoryHecate",
            EVALUATOR,
            "ObservatoryEvaluator",
        ],
    )
    .is_some()
}
fn source_root(path: &Path) -> Result<Option<PathBuf>> {
    let path = fs::canonicalize(path)?;
    let parent = path.parent().context("Original settings have no parent")?;
    let properties = crate::legacy_settings::source_properties(&path)?;
    let mut candidates = vec![
        parent.to_owned(),
        parent.join("ObservatoryCore"),
        parent.join("data/ObservatoryCore"),
    ];
    if let Some(settings) = properties.get("PluginSettings") {
        let settings = if let Some(text) = settings.as_str() {
            crate::core_theme::parse(text)?
        } else {
            settings.clone()
        };
        if let Some(settings) = settings.as_object() {
            for value in settings.values() {
                if let Some(data) = value["DataDirectory"].as_str() {
                    if let Some(data) = crate::legacy_settings::native_path(data, &path)? {
                        if let Some(parent) = data.parent() {
                            candidates.push(parent.to_owned());
                        }
                    }
                }
            }
        }
    }
    for ancestor in path.ancestors() {
        if ancestor
            .file_name()
            .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case("AppData"))
        {
            candidates.push(ancestor.join("Roaming/ObservatoryCore"));
        }
        if ancestor
            .file_name()
            .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case("Roaming"))
        {
            candidates.push(ancestor.join("ObservatoryCore"));
        }
    }
    let mut seen = BTreeSet::new();
    for candidate in candidates {
        if let Ok(candidate) = fs::canonicalize(candidate) {
            if seen.insert(candidate.clone()) && has_plugins(&candidate) {
                return Ok(Some(candidate));
            }
        }
    }
    Ok(None)
}
fn validate_destination(root: &Path, destination: &Path) -> Result<()> {
    let mut ancestor = destination.to_owned();
    let mut suffix = Vec::new();
    while !ancestor.exists() {
        suffix.push(
            ancestor
                .file_name()
                .context("Native destination has no existing ancestor")?
                .to_owned(),
        );
        ensure!(
            ancestor.pop(),
            "Native destination has no existing ancestor"
        );
    }
    let mut resolved = fs::canonicalize(ancestor)?;
    for component in suffix.into_iter().rev() {
        resolved.push(component);
    }
    ensure!(
        !resolved.starts_with(root),
        "Native destination resolves into original storage: {}",
        destination.display()
    );
    Ok(())
}
fn read_limited(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let metadata = fs::metadata(path)?;
    ensure!(
        metadata.is_file() && metadata.len() <= limit,
        "Unsupported size for {}",
        path.display()
    );
    Ok(fs::read(path)?)
}
fn snapshot(source: &Path, target: &Path) -> Result<()> {
    let db = Connection::open_with_flags(source, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("Cannot open original database {}", source.display()))?;
    db.backup(DatabaseName::Main, target, None)?;
    let copy = Connection::open(target)?;
    let check: String = copy.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    ensure!(check == "ok", "Database integrity failure: {check}");
    Ok(())
}
fn identifier(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}
fn merge_database(target: &Path, incoming: &Path) -> Result<usize> {
    let mut db = Connection::open(target)?;
    db.execute(
        "ATTACH DATABASE ?1 AS legacy",
        [incoming.to_string_lossy().as_ref()],
    )?;
    let transaction = db.transaction()?;
    let tables:Vec<(String,String)>=transaction.prepare("SELECT name,sql FROM legacy.sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")?.query_map([],|row|Ok((row.get(0)?,row.get(1)?)))?.collect::<rusqlite::Result<_>>()?;
    let mut added = 0;
    for (name, ddl) in tables {
        let exists: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM main.sqlite_master WHERE type='table' AND name=?1)",
            [&name],
            |row| row.get(0),
        )?;
        if !exists {
            transaction.execute_batch(&ddl)?;
        }
        let table = identifier(&name);
        let columns: Vec<(String, String)> = transaction
            .prepare(&format!("PRAGMA legacy.table_info({table})"))?
            .query_map([], |row| Ok((row.get(1)?, row.get(2)?)))?
            .collect::<rusqlite::Result<_>>()?;
        let native: BTreeSet<String> = transaction
            .prepare(&format!("PRAGMA main.table_info({table})"))?
            .query_map([], |row| row.get(1))?
            .collect::<rusqlite::Result<_>>()?;
        for (column, kind) in &columns {
            if !native.contains(column) {
                ensure!(
                    kind.chars()
                        .all(|c| c.is_ascii_alphanumeric() || "_ () ,".contains(c)),
                    "Unsupported SQLite column type"
                );
                transaction.execute_batch(&format!(
                    "ALTER TABLE {table} ADD COLUMN {} {kind}",
                    identifier(column)
                ))?;
            }
        }
        let autoincrement = ddl.to_ascii_uppercase().contains("AUTOINCREMENT");
        let columns: Vec<String> = columns
            .into_iter()
            .filter(|(name, _)| !autoincrement || !name.eq_ignore_ascii_case("id"))
            .map(|(name, _)| name)
            .collect();
        ensure!(
            !columns.is_empty(),
            "Original table {name} has no importable columns"
        );
        let names = columns
            .iter()
            .map(|name| identifier(name))
            .collect::<Vec<_>>()
            .join(",");
        let values = columns
            .iter()
            .map(|name| format!("incoming.{}", identifier(name)))
            .collect::<Vec<_>>()
            .join(",");
        let equal = columns
            .iter()
            .map(|name| format!("existing.{0} IS incoming.{0}", identifier(name)))
            .collect::<Vec<_>>()
            .join(" AND ");
        added+=transaction.execute(&format!("INSERT OR IGNORE INTO main.{table}({names}) SELECT {values} FROM legacy.{table} AS incoming WHERE NOT EXISTS(SELECT 1 FROM main.{table} AS existing WHERE {equal})"),[])?;
    }
    transaction.commit()?;
    db.execute_batch(
        "DETACH DATABASE legacy; PRAGMA wal_checkpoint(TRUNCATE); PRAGMA journal_mode=DELETE;",
    )?;
    Ok(added)
}
fn row_count(path: &Path) -> Result<usize> {
    let db = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let tables: Vec<String> = db
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")?
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut count = 0;
    for table in tables {
        count += db.query_row(
            &format!("SELECT COUNT(*) FROM {}", identifier(&table)),
            [],
            |row| row.get::<_, usize>(0),
        )?;
    }
    Ok(count)
}

struct Prepared {
    stage: PathBuf,
    destination: PathBuf,
    previous: Option<PathBuf>,
    source: PathBuf,
    rows: usize,
}
/// Caller imports settings first, applies receipt preferences, saves settings
/// atomically, and exits before starting any native workers.
pub fn import_data(source_settings_path: &Path, paths: &ConfigPaths) -> Result<Receipt> {
    let Some(root) = source_root(source_settings_path)? else {
        return Ok(Receipt {
            plugin_settings: json!({}),
            ..Default::default()
        });
    };
    ensure!(
        fs::canonicalize(&paths.data_dir).ok().as_ref() != Some(&root),
        "Source and native storage directories are identical"
    );
    validate_destination(&root, &paths.data_dir)?;
    fs::create_dir_all(&paths.data_dir)?;
    let staging = tempfile::Builder::new()
        .prefix("legacy-import-")
        .tempdir_in(&paths.data_dir)?;
    let directory = staging.path().to_owned();
    let mut receipt = Receipt {
        source_root: Some(root.clone()),
        plugin_settings: json!({}),
        ..Default::default()
    };
    let mut prepared = Vec::new();
    if let Some(folder) = child(&root, &[HECATE, "ObservatoryHecate"]) {
        for name in ["general.db", "journals.db", "spansh.db", "canonn.db"] {
            let source = folder.join(name);
            if !source.is_file() {
                continue;
            }
            let original = directory.join(format!("original-{name}"));
            snapshot(&source, &original)?;
            let destination = paths.data_dir.join("Hecate").join(name);
            validate_destination(&root, &destination)?;
            let stage = directory.join(format!("merged-{name}"));
            let previous = if destination.exists() {
                let previous = directory.join(format!("previous-{name}"));
                snapshot(&destination, &previous)?;
                fs::copy(&previous, &stage)?;
                Some(previous)
            } else {
                None
            };
            let rows = if previous.is_some() {
                merge_database(&stage, &original)?
            } else {
                fs::copy(&original, &stage)?;
                row_count(&stage)?
            };
            prepared.push(Prepared {
                stage,
                destination,
                previous,
                source,
                rows,
            });
        }
    }
    if let Some(folder) = child(&root, &[BIO, "ObservatoryBioInsights"]) {
        let destination = paths.data_dir.join("BioInsights/state.json");
        validate_destination(&root, &destination)?;
        let mut worker = BioInsights::new(json!({"GalacticCodexRemote":false}))?;
        let previous = if destination.is_file() {
            let bytes = read_limited(&destination, 64 * 1024 * 1024)?;
            let state: Value = serde_json::from_slice(&bytes)
                .context("Invalid existing native BioInsights state")?;
            ensure!(
                state.is_object(),
                "Native BioInsights state is not an object"
            );
            worker.restore_state(&state);
            let previous = directory.join("previous-bio-state.json");
            fs::write(&previous, bytes)?;
            Some(previous)
        } else {
            None
        };
        let mut commanders: Vec<_> = fs::read_dir(&folder)?
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .map(|entry| {
                (
                    entry.file_name().to_string_lossy().into_owned(),
                    entry.path(),
                    false,
                )
            })
            .filter(|(fid, _, _)| valid_fid(fid))
            .collect();
        commanders.sort_by(|a, b| a.0.cmp(&b.0));
        if ["CCSeen.bin", "CCMark.bin", "BISeen.bin", "dnotifier.json"]
            .iter()
            .any(|name| folder.join(format!("default.{name}")).is_file())
        {
            commanders.push(("F0".into(), folder.clone(), true));
        }
        let existing_state = worker.state();
        let mut changed = false;
        for (fid, path, default) in commanders {
            let mut payload = json!({"FID":fid});
            let mut found = false;
            for name in ["CCSeen.bin", "CCMark.bin", "BISeen.bin", "dnotifier.json"] {
                let source = path.join(if default {
                    format!("default.{name}")
                } else {
                    name.into()
                });
                if !source.is_file() {
                    continue;
                }
                let bytes = read_limited(&source, 32 * 1024 * 1024)?;
                let archive = directory.join("original-bio").join(&fid);
                fs::create_dir_all(&archive)?;
                fs::write(archive.join(name), &bytes)?;
                payload[if name == "dnotifier.json" {
                    "DistanceNotifier"
                } else {
                    name.trim_end_matches(".bin")
                }] = if name.ends_with(".bin") {
                    decode_legacy_codex(&bytes).with_context(|| {
                        format!("Invalid original BioInsights {}", source.display())
                    })?
                } else {
                    serde_json::from_slice(&bytes).with_context(|| {
                        format!("Invalid original BioInsights {}", source.display())
                    })?
                };
                found = true;
            }
            if found {
                if payload.get("DistanceNotifier").is_none() {
                    if let Some(cache) = existing_state["Commanders"][&fid].get("DistanceNotifier")
                    {
                        payload["DistanceNotifier"] = cache.clone();
                    }
                }
                receipt.codex_entries += worker.restore_legacy_state(&payload)?;
                receipt.commanders += 1;
                changed = true;
            }
        }
        if changed {
            let stage = directory.join("merged-bio-state.json");
            let imported_state = worker.state();
            let count = |state: &Value| {
                state["Commanders"]
                    .as_object()
                    .into_iter()
                    .flat_map(|commanders| commanders.values())
                    .map(|commander| {
                        ["PersonalCodex", "MarkedVariants"]
                            .iter()
                            .map(|key| commander[*key].as_array().map_or(0, Vec::len))
                            .sum::<usize>()
                    })
                    .sum::<usize>()
            };
            let added = count(&imported_state).saturating_sub(count(&existing_state));
            fs::write(&stage, serde_json::to_vec_pretty(&imported_state)?)?;
            prepared.push(Prepared {
                stage,
                destination,
                previous,
                source: folder.clone(),
                rows: added,
            });
        }
    }
    // The source custom palette is stored beside data, not in PluginSettings.
    let properties = crate::legacy_settings::source_properties(source_settings_path)?;
    let plugins = properties
        .get("PluginSettings")
        .map(|value| {
            if let Some(text) = value.as_str() {
                crate::core_theme::parse(text)
            } else {
                Ok(value.clone())
            }
        })
        .transpose()?
        .unwrap_or(json!({}));
    for (name, guid) in [("BioInsights", BIO), ("Evaluator", EVALUATOR)] {
        if let Some(data) = child(&root, &[guid, &format!("Observatory{name}")]) {
            let settings = plugins
                .as_object()
                .and_then(|values| {
                    values.iter().find(|(key, _)| {
                        key.eq_ignore_ascii_case(guid) || key.eq_ignore_ascii_case(name)
                    })
                })
                .map(|(_, value)| value);
            let index = settings
                .and_then(|value| value["ThemeSettings"]["ThemeIndex"].as_u64())
                .unwrap_or(1);
            let source_popout = data.join("popout.json");
            if source_popout.is_file() {
                let bytes = read_limited(&source_popout, 1024 * 1024)?;
                let mut source: Value =
                    serde_json::from_slice(&bytes).context("Invalid original popout.json")?;
                ensure!(source.is_object(), "Original popout.json must be an object");
                let destination = paths.data_dir.join(name).join("popout.json");
                validate_destination(&root, &destination)?;
                let previous = if destination.is_file() {
                    let bytes = read_limited(&destination, 1024 * 1024)?;
                    let native: Value =
                        serde_json::from_slice(&bytes).context("Invalid native popout.json")?;
                    ensure!(native.is_object(), "Native popout.json must be an object");
                    let mut combined = native;
                    merge(&mut combined, &source);
                    source = combined;
                    let previous = directory.join(format!("previous-{name}-popout.json"));
                    fs::write(&previous, bytes)?;
                    Some(previous)
                } else {
                    None
                };
                fs::write(
                    directory.join(format!("original-{name}-popout.json")),
                    bytes,
                )?;
                let stage = directory.join(format!("merged-{name}-popout.json"));
                fs::write(&stage, serde_json::to_vec_pretty(&source)?)?;
                prepared.push(Prepared {
                    stage,
                    destination,
                    previous,
                    source: source_popout,
                    rows: 0,
                });
            }
            let custom = data.join(format!("custom{index}.json"));
            if custom.is_file() {
                let theme: Value =
                    serde_json::from_slice(&read_limited(&custom, 4 * 1024 * 1024)?)?;
                ensure!(
                    theme.is_object(),
                    "Original custom palette must be an object"
                );
                receipt.plugin_settings[name]["ThemeSettings"]["Colors"] = theme
                    .get("Colors")
                    .filter(|value| value.is_object())
                    .unwrap_or(&theme)
                    .clone();
            }
        }
    }
    if prepared.is_empty() {
        return Ok(receipt);
    }
    for item in &prepared {
        if let Some(parent) = item.destination.parent() {
            fs::create_dir_all(parent)?;
        }
    }
    let mut committed = Vec::<usize>::new();
    for (index, item) in prepared.iter().enumerate() {
        if let Err(error) = fs::rename(&item.stage, &item.destination) {
            for index in committed.into_iter().rev() {
                let item = &prepared[index];
                if let Some(previous) = &item.previous {
                    let _ = fs::copy(previous, &item.destination);
                } else {
                    let _ = fs::remove_file(&item.destination);
                }
            }
            return Err(error)
                .with_context(|| format!("Cannot commit imported {}", item.destination.display()));
        }
        committed.push(index);
    }
    for item in prepared {
        receipt.files.push(FileReceipt {
            source: item.source,
            destination: item.destination,
            rows_added: item.rows,
            previous: item.previous,
        });
    }
    receipt.backup_directory = Some(staging.keep());
    let manifest = receipt
        .backup_directory
        .as_ref()
        .unwrap()
        .join("receipt.json");
    fs::write(manifest, serde_json::to_vec_pretty(&receipt)?)?;
    Ok(receipt)
}
fn valid_fid(fid: &str) -> bool {
    let value = fid.strip_prefix('F').unwrap_or(fid);
    !value.is_empty() && value.len() <= 20 && value.bytes().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    fn fixture() -> (tempfile::TempDir, PathBuf, ConfigPaths) {
        let root = tempfile::tempdir().unwrap();
        let original = root.path().join("original");
        fs::create_dir(&original).unwrap();
        let settings = original.join("settings.json");
        fs::write(&settings, b"{}").unwrap();
        let paths = ConfigPaths::from_roots(
            root.path().join("config"),
            root.path().join("data"),
            root.path().join("cache"),
        );
        (root, settings, paths)
    }
    fn original_folder(settings: &Path, guid: &str) -> PathBuf {
        let p = settings.parent().unwrap().join(guid);
        fs::create_dir_all(&p).unwrap();
        p
    }
    fn create_database(path: &Path, value: &str) -> Connection {
        let db = Connection::open(path).unwrap();
        db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=0; CREATE TABLE records (id INTEGER PRIMARY KEY AUTOINCREMENT, fid TEXT NOT NULL, body INTEGER NOT NULL, value TEXT, UNIQUE(fid,body));").unwrap();
        db.execute(
            "INSERT INTO records(fid,body,value) VALUES('F1',1,?1)",
            [value],
        )
        .unwrap();
        db
    }
    fn codex_bytes() -> Vec<u8> {
        let oracle: Value =
            serde_json::from_str(include_str!("../../fixtures/bioinsights-state-oracle.json"))
                .unwrap();
        base64::engine::general_purpose::STANDARD
            .decode(oracle["BytesBase64"].as_str().unwrap())
            .unwrap()
    }
    #[test]
    fn snapshots_all_four_live_wal_databases_and_keeps_source_bytes_unchanged() {
        let (_root, settings, paths) = fixture();
        let folder = original_folder(&settings, HECATE);
        let mut live = Vec::new();
        let mut before = Vec::new();
        for name in ["general.db", "journals.db", "spansh.db", "canonn.db"] {
            let path = folder.join(name);
            live.push(create_database(&path, name));
            before.push((
                path.clone(),
                fs::read(&path).unwrap(),
                fs::read(path.with_extension("db-wal")).unwrap(),
            ));
        }
        let receipt = import_data(&settings, &paths).unwrap();
        assert_eq!(receipt.files.len(), 4);
        assert!(receipt
            .backup_directory
            .as_ref()
            .unwrap()
            .join("receipt.json")
            .is_file());
        for item in receipt.files {
            assert_eq!(item.rows_added, 1);
            let db = Connection::open(item.destination).unwrap();
            let count: i64 = db
                .query_row("SELECT COUNT(*) FROM records", [], |r| r.get(0))
                .unwrap();
            assert_eq!(count, 1);
        }
        for (path, db, wal) in before {
            assert_eq!(fs::read(&path).unwrap(), db);
            assert_eq!(fs::read(path.with_extension("db-wal")).unwrap(), wal);
        }
        drop(live);
    }
    #[test]
    fn merges_without_overwriting_native_business_keys_and_is_idempotent() {
        let (_root, settings, paths) = fixture();
        let folder = original_folder(&settings, HECATE);
        let source = create_database(&folder.join("general.db"), "source older");
        source
            .execute(
                "INSERT INTO records(fid,body,value) VALUES('F2',2,'source unique')",
                [],
            )
            .unwrap();
        fs::create_dir_all(paths.data_dir.join("Hecate")).unwrap();
        let native = create_database(&paths.data_dir.join("Hecate/general.db"), "native newer");
        native
            .execute(
                "INSERT INTO records(fid,body,value) VALUES('F3',3,'native unique')",
                [],
            )
            .unwrap();
        drop(native);
        let receipt = import_data(&settings, &paths).unwrap();
        assert_eq!(receipt.files[0].rows_added, 1);
        assert!(receipt.files[0].previous.as_ref().unwrap().is_file());
        let db = Connection::open(paths.data_dir.join("Hecate/general.db")).unwrap();
        let rows: Vec<(String, String)> = db
            .prepare("SELECT fid,value FROM records ORDER BY fid")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        assert_eq!(
            rows,
            vec![
                ("F1".into(), "native newer".into()),
                ("F2".into(), "source unique".into()),
                ("F3".into(), "native unique".into())
            ]
        );
        drop(db);
        assert_eq!(
            import_data(&settings, &paths).unwrap().files[0].rows_added,
            0
        );
    }
    #[test]
    fn imports_commander_codex_marks_distance_cache_and_preserves_other_native_commander() {
        let (_root, settings, paths) = fixture();
        let folder = original_folder(&settings, BIO).join("F123");
        fs::create_dir(&folder).unwrap();
        let bytes = codex_bytes();
        fs::write(folder.join("CCSeen.bin"), &bytes).unwrap();
        fs::write(folder.join("CCMark.bin"), &bytes).unwrap();
        let cache = json!({"SystemID64":123,"MinimumDistance":500,"RadiusKM":6500000.0,"ScanPosition":[{"Latitude":1.5,"Longitude":-2.5}]});
        fs::write(folder.join("dnotifier.json"), cache.to_string()).unwrap();
        fs::create_dir_all(paths.data_dir.join("BioInsights")).unwrap();
        let mut worker = BioInsights::new(json!({"GalacticCodexRemote":false})).unwrap();
        worker
            .restore_legacy_state(
                &json!({"FID":"F456","CCSeen":decode_legacy_codex(&bytes).unwrap()}),
            )
            .unwrap();
        fs::write(
            paths.data_dir.join("BioInsights/state.json"),
            worker.state().to_string(),
        )
        .unwrap();
        let receipt = import_data(&settings, &paths).unwrap();
        assert_eq!(receipt.commanders, 1);
        assert!(receipt.codex_entries > 0);
        let state: Value = serde_json::from_slice(
            &fs::read(paths.data_dir.join("BioInsights/state.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(state["Commanders"]["F123"]["DistanceNotifier"], cache);
        assert_eq!(
            state["Commanders"]["F456"]["PersonalCodex"],
            worker.state()["Commanders"]["F456"]["PersonalCodex"]
        );
        assert_eq!(
            state["Commanders"]["F123"]["LegacySource"]["CCSeen"],
            decode_legacy_codex(&bytes).unwrap()
        );
        assert!(!state["Commanders"]["F123"]["MarkedVariants"]
            .as_array()
            .unwrap()
            .is_empty());
        assert_eq!(fs::read(folder.join("CCSeen.bin")).unwrap(), bytes);
        let repeated = import_data(&settings, &paths).unwrap();
        assert_eq!(
            repeated
                .files
                .iter()
                .find(|file| file.destination.ends_with("BioInsights/state.json"))
                .unwrap()
                .rows_added,
            0
        );
        let after: Value = serde_json::from_slice(
            &fs::read(paths.data_dir.join("BioInsights/state.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(after, state);
    }
    #[test]
    fn invalid_codex_prevents_commit_of_prepared_database_and_preserves_native_file() {
        let (_root, settings, paths) = fixture();
        let hecate = original_folder(&settings, HECATE);
        let db = create_database(&hecate.join("general.db"), "original");
        drop(db);
        let bio = original_folder(&settings, BIO).join("F123");
        fs::create_dir(&bio).unwrap();
        fs::write(bio.join("BISeen.bin"), b"invalid binary").unwrap();
        fs::create_dir_all(paths.data_dir.join("Hecate")).unwrap();
        let path = paths.data_dir.join("Hecate/general.db");
        let db = create_database(&path, "native");
        drop(db);
        let before = fs::read(&path).unwrap();
        assert!(import_data(&settings, &paths).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        assert!(!paths.data_dir.join("BioInsights/state.json").exists());
    }
    #[test]
    fn evaluator_only_palette_and_source_popout_import_without_opening_window() {
        let (_root, settings, paths) = fixture();
        let folder = original_folder(&settings, EVALUATOR);
        fs::write(
            &settings,
            json!({"PluginSettings":{EVALUATOR:{"ThemeSettings":{"ThemeIndex":3}}}}).to_string(),
        )
        .unwrap();
        let colors = json!({"DefaultText":{"R":1,"G":2,"B":3},"Alert":{"R":200,"G":0,"B":0}});
        fs::write(folder.join("custom3.json"), colors.to_string()).unwrap();
        let popout = json!({"Location":{"X":-10,"Y":20},"Size":{"Width":850,"Height":450},"IsOnTop":true,"ShowTitleBar":false,"HideButtons":true});
        fs::write(folder.join("popout.json"), popout.to_string()).unwrap();
        let receipt = import_data(&settings, &paths).unwrap();
        assert_eq!(receipt.files.len(), 1);
        let copied: Value = serde_json::from_slice(
            &fs::read(paths.data_dir.join("Evaluator/popout.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(copied, popout);
        let mut prefs = Settings::default();
        receipt.apply_to(&mut prefs);
        assert_eq!(
            prefs.plugins["Evaluator"]["ThemeSettings"]["Colors"],
            colors
        );
        assert!(prefs.extra.get("plugin_windows").is_none());
    }
    #[cfg(unix)]
    #[test]
    fn native_destination_symlink_into_original_is_rejected_before_commit() {
        let (_root, settings, paths) = fixture();
        let folder = original_folder(&settings, HECATE);
        let db = create_database(&folder.join("general.db"), "original");
        drop(db);
        let before = fs::read(folder.join("general.db")).unwrap();
        fs::create_dir_all(&paths.data_dir).unwrap();
        std::os::unix::fs::symlink(&folder, paths.data_dir.join("Hecate")).unwrap();
        assert!(import_data(&settings, &paths)
            .unwrap_err()
            .to_string()
            .contains("original storage"));
        assert_eq!(fs::read(folder.join("general.db")).unwrap(), before);
    }
    #[test]
    fn no_adjacent_original_storage_returns_settings_only_receipt() {
        let (_root, settings, paths) = fixture();
        let receipt = import_data(&settings, &paths).unwrap();
        assert!(receipt.source_root.is_none());
        assert!(receipt.files.is_empty());
        assert!(!paths.data_dir.exists());
    }
}
