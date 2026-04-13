use crate::model::Connection;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use super::{config_dir, migrate_file};

#[derive(Debug, Serialize, Deserialize)]
struct StorageFile {
    #[serde(default)]
    connections: BTreeMap<String, Connection>,
}

pub struct TomlStorage {
    path: PathBuf,
}

impl TomlStorage {
    pub fn new() -> Result<Self> {
        migrate_file("connections.toml");
        let dir = config_dir()?;
        Ok(Self {
            path: dir.join("connections.toml"),
        })
    }

    pub fn load(&self) -> Result<Vec<Connection>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&self.path)
            .with_context(|| format!("Failed to read {}", self.path.display()))?;
        let storage: StorageFile = toml::from_str(&content).with_context(|| {
            format!(
                "Failed to parse {}. The file may be corrupted — it has not been modified.",
                self.path.display()
            )
        })?;
        let mut connections: Vec<Connection> = storage.connections.into_values().collect();
        connections.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(connections)
    }

    pub fn save(&self, connections: &[Connection]) -> Result<()> {
        let mut map = BTreeMap::new();
        for conn in connections {
            map.insert(conn.id.clone(), conn.clone());
        }
        let storage = StorageFile { connections: map };
        let content =
            toml::to_string_pretty(&storage).context("Failed to serialize connections")?;

        // Atomic write: write to temp file, then rename
        let tmp_path = self.path.with_extension("toml.tmp");
        fs::write(&tmp_path, &content)
            .with_context(|| format!("Failed to write {}", tmp_path.display()))?;
        fs::rename(&tmp_path, &self.path).with_context(|| {
            format!(
                "Failed to rename {} to {}",
                tmp_path.display(),
                self.path.display()
            )
        })?;
        Ok(())
    }
}
