use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    env,
    fs::{self, File},
    io::{BufReader, Write},
    path::{Path, PathBuf},
};

/// Unknown fields survive a load/save cycle, allowing newer plugins and older
/// application versions to share a settings file without silently discarding data.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub journal_dir: Option<PathBuf>,
    pub poll_interval_ms: u64,
    pub popup: bool,
    pub speech: bool,
    pub theme: String,
    pub auto_start: bool,
    pub start_read_all: bool,
    pub plugins: Value,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            journal_dir: None,
            poll_interval_ms: 250,
            popup: true,
            speech: false,
            theme: "dark".into(),
            auto_start: false,
            start_read_all: false,
            plugins: Value::Object(Default::default()),
            extra: Default::default(),
        }
    }
}

impl Settings {
    /// A missing file is a first launch; a damaged file is an error and is never
    /// silently replaced with defaults.
    pub fn load(path: &Path) -> Result<Self> {
        match File::open(path) {
            Ok(file) => serde_json::from_reader(BufReader::new(file))
                .with_context(|| format!("Cannot read settings {}", path.display())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => {
                Err(error).with_context(|| format!("Cannot open settings {}", path.display()))
            }
        }
    }

    /// Replace the whole file atomically. A crash before rename leaves the old
    /// settings intact; the temporary file lives on the same filesystem.
    pub fn save(&self, path: &Path) -> Result<()> {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        fs::create_dir_all(parent)
            .with_context(|| format!("Cannot create settings directory {}", parent.display()))?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        serde_json::to_writer_pretty(&mut temporary, self)?;
        temporary.write_all(b"\n")?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(path)
            .map_err(|error| error.error)
            .with_context(|| format!("Cannot save settings {}", path.display()))?;
        #[cfg(unix)]
        File::open(parent)?.sync_all()?;
        Ok(())
    }

    pub fn poll_interval(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.poll_interval_ms.clamp(50, 60_000))
    }
}

#[derive(Clone, Debug)]
pub struct ConfigPaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub settings_file: PathBuf,
    pub plugin_dir: PathBuf,
}

impl ConfigPaths {
    pub fn discover() -> Result<Self> {
        let home = home_dir().context("Cannot determine the home directory")?;
        let config = env_path("XDG_CONFIG_HOME").unwrap_or_else(|| {
            if cfg!(windows) {
                env_path("APPDATA").unwrap_or_else(|| home.join("AppData/Roaming"))
            } else {
                home.join(".config")
            }
        });
        let data = env_path("XDG_DATA_HOME").unwrap_or_else(|| {
            if cfg!(windows) {
                env_path("LOCALAPPDATA").unwrap_or_else(|| home.join("AppData/Local"))
            } else {
                home.join(".local/share")
            }
        });
        let cache = env_path("XDG_CACHE_HOME").unwrap_or_else(|| {
            if cfg!(windows) {
                data.join("Cache")
            } else {
                home.join(".cache")
            }
        });
        Ok(Self::from_roots(config, data, cache))
    }

    pub fn from_roots(config: PathBuf, data: PathBuf, cache: PathBuf) -> Self {
        let config_dir = config.join("observatory-core");
        let data_dir = data.join("observatory-core");
        Self {
            settings_file: config_dir.join("settings.json"),
            plugin_dir: data_dir.join("plugins"),
            config_dir,
            data_dir,
            cache_dir: cache.join("observatory-core"),
        }
    }

    pub fn create_dirs(&self) -> Result<()> {
        for path in [
            &self.config_dir,
            &self.data_dir,
            &self.cache_dir,
            &self.plugin_dir,
        ] {
            fs::create_dir_all(path)
                .with_context(|| format!("Cannot create {}", path.display()))?;
        }
        Ok(())
    }
}

pub(crate) fn home_dir() -> Option<PathBuf> {
    env_path("HOME").or_else(|| env_path("USERPROFILE"))
}

pub(crate) fn env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_atomically_and_keeps_unknown_fields() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/settings.json");
        let settings = Settings {
            journal_dir: Some(dir.path().join("ゲーム journals")),
            ..Settings::default()
        };
        settings.save(&path).unwrap();
        let mut loaded = Settings::load(&path).unwrap();
        assert_eq!(loaded.journal_dir, settings.journal_dir);
        loaded
            .extra
            .insert("future_setting".into(), serde_json::json!({"key": 42}));
        loaded.popup = false;
        loaded.save(&path).unwrap();
        let reloaded = Settings::load(&path).unwrap();
        assert!(!reloaded.popup);
        assert_eq!(reloaded.extra["future_setting"]["key"], 42);
        assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
    }

    #[test]
    fn malformed_settings_are_not_treated_as_first_launch() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        assert!(Settings::load(&path).is_ok());
        fs::write(&path, b"{ broken").unwrap();
        assert!(Settings::load(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"{ broken");
    }
}
