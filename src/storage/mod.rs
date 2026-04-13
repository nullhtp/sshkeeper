mod import;
mod toml_storage;
pub mod transfer_history;

pub use import::import_ssh_config;
pub use toml_storage::TomlStorage;
pub use transfer_history::TransferHistory;

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Returns `~/.sshkeeper/`, creating it if needed.
pub fn config_dir() -> Result<PathBuf> {
    let dir = dirs::home_dir()
        .context("Could not determine home directory")?
        .join(".sshkeeper");
    fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create config directory: {}", dir.display()))?;
    Ok(dir)
}

/// If `filename` doesn't exist in the new config dir but does in the old
/// platform-specific config dir, copy it over.
pub fn migrate_file(filename: &str) {
    let new_dir = match dirs::home_dir() {
        Some(h) => h.join(".sshkeeper"),
        None => return,
    };
    let new_path = new_dir.join(filename);
    if new_path.exists() {
        return;
    }
    let old_dir = match dirs::config_dir() {
        Some(d) => d.join("sshkeeper"),
        None => return,
    };
    let old_path = old_dir.join(filename);
    if old_path.exists() {
        let _ = fs::copy(&old_path, &new_path);
    }
}
