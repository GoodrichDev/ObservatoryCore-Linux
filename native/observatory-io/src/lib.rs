//! Journal ingestion is synchronous so callers can put it on their existing worker
//! thread. It never runs game or plugin code on a file watcher callback.

mod config;
mod discovery;
mod journal;

pub use config::{ConfigPaths, Settings};
pub use discovery::{discover_journal_dirs, resolve_journal_dir};
pub use journal::{
    journal_files, AuxiliaryUpdate, JournalTailer, ReadIssue, ReplayItem, ReplaySummary, ScanBatch,
};
