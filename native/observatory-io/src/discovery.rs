use crate::config::{env_path, home_dir};
use anyhow::{Context, Result};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

const JOURNAL_SUFFIX: &str = "Saved Games/Frontier Developments/Elite Dangerous";

/// Resolve an explicit choice first. An invalid explicit path is reported rather
/// than falling back to an unrelated directory and pretending monitoring works.
pub fn resolve_journal_dir(explicit: Option<&Path>) -> Result<PathBuf> {
    let environment = env_path("ELITE_JOURNAL_DIR");
    if let Some(path) = explicit.or(environment.as_deref()) {
        fs::read_dir(path)
            .with_context(|| format!("Journal directory is not readable: {}", path.display()))?;
        return Ok(path.to_owned());
    }
    discover_journal_dirs().into_iter().next().context(
        "Elite Dangerous journals were not found. Select the journal directory or set ELITE_JOURNAL_DIR.",
    )
}

/// Discover existing native, Proton, Steam Flatpak, Wine, and Lutris locations.
/// The game remains under Proton; Observatory itself uses ordinary Linux I/O.
pub fn discover_journal_dirs() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = env_path("ELITE_JOURNAL_DIR") {
        candidates.push(path);
    }
    if let Some(prefix) = env_path("WINEPREFIX") {
        add_prefix(&prefix, &mut candidates);
    }
    if let Some(prefix) = env_path("STEAM_COMPAT_DATA_PATH") {
        add_prefix(&prefix, &mut candidates);
        add_prefix(&prefix.join("pfx"), &mut candidates);
    }
    let mut steam_roots = Vec::new();
    if let Some(path) = env_path("STEAM_COMPAT_CLIENT_INSTALL_PATH") {
        steam_roots.push(path);
    }
    if let Some(home) = home_dir() {
        candidates.push(home.join(JOURNAL_SUFFIX));
        candidates.push(home.join(".local/share/Frontier Developments/Elite Dangerous"));
        for relative in [
            ".steam/steam",
            ".steam/root",
            ".steam/debian-installation",
            ".local/share/Steam",
            ".var/app/com.valvesoftware.Steam/data/Steam",
        ] {
            steam_roots.push(home.join(relative));
        }
        add_prefix(&home.join(".wine"), &mut candidates);
        for prefix in ["Games/elite-dangerous", "Games/elite-dangerous-odyssey"] {
            add_prefix(&home.join(prefix), &mut candidates);
        }
        let lutris = env_path("XDG_CONFIG_HOME")
            .unwrap_or_else(|| home.join(".config"))
            .join("lutris/games");
        add_lutris_prefixes(&lutris, &home, &mut candidates);
        add_lutris_prefixes(
            &home.join(".var/app/net.lutris.Lutris/config/lutris/games"),
            &home,
            &mut candidates,
        );
    }
    let mut libraries = steam_roots.clone();
    for root in &steam_roots {
        for relative in ["steamapps/libraryfolders.vdf", "config/libraryfolders.vdf"] {
            if let Ok(content) = fs::read_to_string(root.join(relative)) {
                libraries.extend(library_paths(&content));
            }
        }
    }
    for library in unique_paths(libraries) {
        add_prefix(
            &library.join("steamapps/compatdata/359320/pfx"),
            &mut candidates,
        );
    }
    unique_paths(candidates.into_iter().filter(|p| p.is_dir()).collect())
}

fn add_prefix(prefix: &Path, candidates: &mut Vec<PathBuf>) {
    let users = prefix.join("drive_c/users");
    if let Ok(entries) = fs::read_dir(&users) {
        let mut entries: Vec<_> = entries.flatten().map(|entry| entry.path()).collect();
        entries.sort();
        for user in entries {
            candidates.push(user.join(JOURNAL_SUFFIX));
        }
    }
}

fn unique_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    paths
        .into_iter()
        .filter(|path| seen.insert(fs::canonicalize(path).unwrap_or_else(|_| path.to_owned())))
        .collect()
}

fn add_lutris_prefixes(directory: &Path, home: &Path, candidates: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    // Only read individual launcher definitions, never recursively walk game data.
    let mut files: Vec<_> = entries.flatten().map(|entry| entry.path()).collect();
    files.sort();
    for path in files {
        if !matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("yml" | "yaml")
        ) {
            continue;
        }
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        for line in content.lines() {
            let Some(value) = line.trim().strip_prefix("prefix:") else {
                continue;
            };
            let value = value.trim().trim_matches(['\'', '"']);
            let prefix = if let Some(relative) = value.strip_prefix("~/") {
                home.join(relative)
            } else {
                PathBuf::from(value)
            };
            if prefix.is_absolute() {
                add_prefix(&prefix, candidates);
            }
        }
    }
}

/// Steam VDF files have quoted key/value tokens. This accepts both the current
/// numbered objects with a `path` field and the original numbered path entries.
fn library_paths(vdf: &str) -> Vec<PathBuf> {
    let tokens = quoted_tokens(vdf);
    tokens
        .windows(2)
        .filter_map(|pair| {
            if pair[0] == "path" || pair[0].bytes().all(|c| c.is_ascii_digit()) {
                let path = PathBuf::from(&pair[1]);
                path.is_absolute().then_some(path)
            } else {
                None
            }
        })
        .collect()
}

fn quoted_tokens(input: &str) -> Vec<String> {
    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();
    while let Some(character) = chars.next() {
        if character != '"' {
            continue;
        }
        let mut token = String::new();
        while let Some(character) = chars.next() {
            match character {
                '"' => break,
                '\\' => match chars.next() {
                    Some('\\') => token.push('\\'),
                    Some('"') => token.push('"'),
                    Some(other) => {
                        token.push('\\');
                        token.push(other);
                    }
                    None => break,
                },
                other => token.push(other),
            }
        }
        tokens.push(token);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_current_and_legacy_library_files() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("Steam Library");
        let escaped = path.display().to_string().replace('\\', "\\\\");
        let current =
            format!("\"libraryfolders\" {{ \"0\" {{ \"path\" \"{escaped}\" \"label\" \"\" }} }}");
        let old = format!("\"LibraryFolders\" {{ \"1\" \"{escaped}\" }}");
        assert_eq!(
            library_paths(&current).as_slice(),
            std::slice::from_ref(&path)
        );
        assert_eq!(library_paths(&old), [path]);
    }

    #[test]
    fn prefix_uses_actual_wine_user_names() {
        let dir = tempfile::tempdir().unwrap();
        let journals = dir.path().join("drive_c/users/cmdr").join(JOURNAL_SUFFIX);
        fs::create_dir_all(&journals).unwrap();
        let mut found = Vec::new();
        add_prefix(dir.path(), &mut found);
        assert_eq!(found, [journals]);
    }

    #[test]
    fn invalid_explicit_directory_does_not_fall_back() {
        let dir = tempfile::tempdir().unwrap();
        assert!(resolve_journal_dir(Some(&dir.path().join("absent"))).is_err());
    }
}
