use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use crate::ssh::transfer::TransferDirection;

const MAX_ENTRIES_PER_CONNECTION: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferEntry {
    pub direction: String, // "upload" or "download"
    pub local_path: String,
    pub remote_path: String,
    pub recursive: bool,
    pub timestamp: DateTime<Utc>,
}

impl TransferEntry {
    pub fn new(
        direction: TransferDirection,
        local_path: String,
        remote_path: String,
        recursive: bool,
    ) -> Self {
        Self {
            direction: match direction {
                TransferDirection::Upload => "upload".into(),
                TransferDirection::Download => "download".into(),
            },
            local_path,
            remote_path,
            recursive,
            timestamp: Utc::now(),
        }
    }

    pub fn transfer_direction(&self) -> TransferDirection {
        if self.direction == "download" {
            TransferDirection::Download
        } else {
            TransferDirection::Upload
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct HistoryFile {
    #[serde(default)]
    connections: BTreeMap<String, Vec<TransferEntry>>,
}

pub struct TransferHistory {
    path: PathBuf,
    data: HistoryFile,
}

impl TransferHistory {
    pub fn load() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("sshkeeper");
        fs::create_dir_all(&config_dir)?;
        let path = config_dir.join("transfer_history.toml");

        let data = if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            toml::from_str(&content).unwrap_or_default()
        } else {
            HistoryFile::default()
        };

        Ok(Self { path, data })
    }

    pub fn entries_for(&self, connection_id: &str) -> &[TransferEntry] {
        self.data
            .connections
            .get(connection_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn push(&mut self, connection_id: &str, entry: TransferEntry) {
        let entries = self
            .data
            .connections
            .entry(connection_id.to_string())
            .or_default();
        entries.push(entry);
        if entries.len() > MAX_ENTRIES_PER_CONNECTION {
            entries.remove(0);
        }
    }

    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(&self.data)
            .context("Failed to serialize transfer history")?;
        let tmp_path = self.path.with_extension("toml.tmp");
        fs::write(&tmp_path, &content)
            .with_context(|| format!("Failed to write {}", tmp_path.display()))?;
        fs::rename(&tmp_path, &self.path)
            .with_context(|| format!("Failed to rename {} to {}", tmp_path.display(), self.path.display()))?;
        Ok(())
    }
}
