//! Settings the Rust core needs before the interface is loaded: language, what
//! closing the window does, notifications. Kept as JSON in the app's config folder;
//! skins and colours stay with the interface.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Preferences {
    /// A locale such as "de"; `None` follows the system.
    pub language: Option<String>,
    /// Closing the window keeps the app running in the tray.
    pub close_to_tray: bool,
    /// Desktop notifications about the active NAS.
    pub notifications: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            language: None,
            // Off by default: some Linux desktops show no tray icons, and a hidden
            // window could not be brought back there.
            close_to_tray: false,
            notifications: true,
        }
    }
}

fn path(dir: &Path) -> PathBuf {
    dir.join("preferences.json")
}

pub fn load(dir: &Path) -> Preferences {
    fs::read(path(dir))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

pub fn save(dir: &Path, preferences: &Preferences) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let temporary = path(dir).with_extension("json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(preferences)?)?;
    fs::rename(temporary, path(dir))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_gives_defaults_and_saved_values_come_back() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load(dir.path()), Preferences::default());
        let changed = Preferences {
            language: Some("de".into()),
            close_to_tray: true,
            notifications: false,
        };
        save(dir.path(), &changed).unwrap();
        assert_eq!(load(dir.path()), changed);
    }
}
