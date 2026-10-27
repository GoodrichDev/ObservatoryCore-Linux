use anyhow::{Context, Result};
use observatory_domain::Finding;
use rusqlite::{params, Connection};
use std::{fs::File, path::Path};

pub struct Store(Connection);

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let db = Connection::open(path).context("open findings database")?;
        db.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;
            CREATE TABLE IF NOT EXISTS findings (
                id INTEGER PRIMARY KEY, plugin TEXT NOT NULL, timestamp TEXT NOT NULL,
                system TEXT NOT NULL, body TEXT NOT NULL, title TEXT NOT NULL, detail TEXT NOT NULL,
                UNIQUE(plugin,timestamp,system,body,title,detail));",
        )?;
        Ok(Self(db))
    }

    pub fn insert(&self, f: &Finding) -> Result<bool> {
        Ok(self.0.execute("INSERT OR IGNORE INTO findings(plugin,timestamp,system,body,title,detail) VALUES(?1,?2,?3,?4,?5,?6)",
            params![f.plugin,f.timestamp,f.system,f.body,f.title,f.detail])? != 0)
    }

    #[cfg(test)]
    pub fn recent(&self, limit: usize) -> Result<Vec<Finding>> {
        let mut query = self.0.prepare("SELECT plugin,timestamp,system,body,title,detail FROM (SELECT * FROM findings ORDER BY id DESC LIMIT ?1) ORDER BY id")?;
        let rows = query.query_map([limit as i64], |row| {
            Ok(Finding {
                plugin: row.get(0)?,
                timestamp: row.get(1)?,
                system: row.get(2)?,
                body: row.get(3)?,
                title: row.get(4)?,
                detail: row.get(5)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn export(&self, path: &Path, delimiter: u8, plugin: Option<&str>) -> Result<usize> {
        let mut out = csv::WriterBuilder::new()
            .delimiter(delimiter)
            .from_writer(File::create(path)?);
        out.write_record(["Plugin", "Timestamp", "System", "Body", "Title", "Detail"])?;
        let mut query = self.0.prepare("SELECT plugin,timestamp,system,body,title,detail FROM findings WHERE (?1 IS NULL OR plugin=?1) ORDER BY id")?;
        let rows = query.query_map([plugin], |r| {
            Ok([
                r.get::<_, String>(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ])
        })?;
        let mut count = 0;
        for row in rows {
            out.write_record(row?)?;
            count += 1;
        }
        out.flush()?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn export_quotes_and_persistence_deduplicate() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("findings.sqlite3");
        let f = Finding {
            plugin: "Explorer".into(),
            timestamp: "now".into(),
            system: "Sol".into(),
            body: "Earth".into(),
            title: "Earth-like world".into(),
            detail: "one, two\n\"quoted\"".into(),
        };
        {
            let db = Store::open(&path).unwrap();
            assert!(db.insert(&f).unwrap());
            assert!(!db.insert(&f).unwrap());
        }
        let db = Store::open(&path).unwrap();
        assert_eq!(db.recent(100).unwrap(), vec![f.clone()]);
        let csv_path = temp.path().join("export.csv");
        assert_eq!(db.export(&csv_path, b',', None).unwrap(), 1);
        let record = csv::Reader::from_path(csv_path)
            .unwrap()
            .records()
            .next()
            .unwrap()
            .unwrap();
        assert_eq!(&record[5], f.detail);
    }
}
