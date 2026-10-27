use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use rusqlite::{params, types::ValueRef, Connection};
use serde_json::{json, Value};
use std::{io::Read, path::Path};

pub const MAX_DOCUMENT: usize = 32 * 1024 * 1024;

/// An original-format database of compressed journal visits or remote documents.
pub struct SystemStore {
    db: Connection,
    table: &'static str,
}
impl SystemStore {
    pub fn open(path: Option<&Path>, table: &'static str) -> Result<Self> {
        anyhow::ensure!(
            matches!(table, "journals" | "spansh" | "canonn"),
            "unknown system store"
        );
        let db = match path {
            Some(path) => Connection::open(path)?,
            None => Connection::open_in_memory()?,
        };
        db.execute_batch(&format!("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; CREATE TABLE IF NOT EXISTS {table}(id INTEGER PRIMARY KEY AUTOINCREMENT,id64 INTEGER NOT NULL,fid INTEGER NOT NULL,data BLOB,inserted DATETIME NOT NULL); CREATE INDEX IF NOT EXISTS {table}_id64_idx ON {table}(id64);"))?;
        Ok(Self { db, table })
    }
    pub fn add(&self, id: u64, fid: i64, text: &str, timestamp: &str, replace: bool) -> Result<()> {
        if id == 0 || text.is_empty() {
            return Ok(());
        }
        anyhow::ensure!(text.len() <= MAX_DOCUMENT, "Hecate document exceeds 32 MiB");
        let blob = zstd::stream::encode_all(text.as_bytes(), 3)?;
        if replace {
            self.db.execute(
                &format!("DELETE FROM {} WHERE id64=?1", self.table),
                [id as i64],
            )?;
        }
        let exists = self.db.query_row(
            &format!(
                "SELECT COUNT(*) FROM {} WHERE id64=?1 AND fid=?2 AND inserted=?3 AND data=?4",
                self.table
            ),
            params![id as i64, fid, timestamp, &blob],
            |r| r.get::<_, i64>(0),
        )?;
        if exists == 0 {
            self.db.execute(
                &format!(
                    "INSERT INTO {}(id64,fid,data,inserted) VALUES(?1,?2,?3,?4)",
                    self.table
                ),
                params![id as i64, fid, blob, timestamp],
            )?;
        }
        Ok(())
    }
    pub fn latest(&self, id: u64) -> Result<Option<(String, String)>> {
        let mut statement = self.db.prepare(&format!(
            "SELECT data,inserted FROM {} WHERE id64=?1 ORDER BY id DESC LIMIT 1",
            self.table
        ))?;
        let mut rows = statement.query([id as i64])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        Ok(Some((read_document(row.get_ref(0)?)?, row.get(1)?)))
    }
    pub fn visits(&self, id: u64, before: &str) -> Result<(Vec<Value>, Vec<i64>)> {
        let mut statement = self.db.prepare(&format!(
            "SELECT data,fid,inserted FROM {} WHERE id64=?1 ORDER BY inserted,id",
            self.table
        ))?;
        let mut rows = statement.query([id as i64])?;
        let mut events = Vec::new();
        let mut fids = Vec::new();
        let cutoff = epoch(before)?;
        while let Some(row) = rows.next()? {
            let inserted: String = row.get(2)?;
            if epoch(&inserted)? >= cutoff {
                continue;
            }
            let fid: i64 = row.get(1)?;
            let text = read_document(row.get_ref(0)?)?;
            for line in text.lines().filter(|l| !l.trim().is_empty()) {
                events.push(serde_json::from_str(line).context("invalid stored Hecate journal")?);
                fids.push(fid);
            }
        }
        Ok((events, fids))
    }
    pub fn visited(&self, ids: &[u64]) -> Result<Vec<u64>> {
        let mut statement = self.db.prepare(&format!(
            "SELECT 1 FROM {} WHERE id64=?1 LIMIT 1",
            self.table
        ))?;
        let mut out = Vec::new();
        for id in ids {
            if statement.exists([*id as i64])? {
                out.push(*id);
            }
        }
        Ok(out)
    }
    pub fn clear(&self) -> Result<()> {
        self.db
            .execute(&format!("DELETE FROM {}", self.table), [])?;
        Ok(())
    }
    pub fn count(&self) -> Result<i64> {
        Ok(self
            .db
            .query_row(&format!("SELECT COUNT(*) FROM {}", self.table), [], |r| {
                r.get(0)
            })?)
    }
    pub fn details(&self) -> Result<Value> {
        Ok(json!({"Store":self.table,"Records":self.count()?}))
    }
}

fn read_document(value: ValueRef<'_>) -> Result<String> {
    match value {
        ValueRef::Text(text) => Ok(std::str::from_utf8(text)?.to_owned()),
        ValueRef::Blob(blob) => {
            if blob.starts_with(&[0x28, 0xb5, 0x2f, 0xfd]) {
                let decoder = zstd::stream::read::Decoder::new(blob)?;
                let mut bytes = Vec::new();
                decoder
                    .take((MAX_DOCUMENT + 1) as u64)
                    .read_to_end(&mut bytes)?;
                anyhow::ensure!(
                    bytes.len() <= MAX_DOCUMENT,
                    "stored Hecate document exceeds 32 MiB"
                );
                Ok(String::from_utf8(bytes)?)
            } else {
                Ok(std::str::from_utf8(blob)?.to_owned())
            }
        }
        _ => anyhow::bail!("stored Hecate document is not text or a blob"),
    }
}
pub fn epoch(timestamp: &str) -> Result<i64> {
    if timestamp.is_empty() {
        return Ok(0);
    }
    if let Ok(value) = timestamp.parse::<i64>() {
        return Ok(value);
    }
    if let Ok(value) = DateTime::parse_from_rfc3339(timestamp) {
        return Ok(value.timestamp());
    }
    for format in ["%Y-%m-%d %H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S%.f"] {
        if let Ok(value) = NaiveDateTime::parse_from_str(timestamp, format) {
            return Ok(value.and_utc().timestamp());
        }
    }
    anyhow::bail!("invalid Hecate timestamp")
}
pub fn timestamp(epoch: i64) -> String {
    DateTime::from_timestamp(epoch, 0)
        .unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}
pub fn now() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}
