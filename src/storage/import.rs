use crate::model::Connection;
use anyhow::{Context, Result};
use ssh2_config::{ParseRule, SshConfig};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub struct ImportResult {
    pub imported: Vec<Connection>,
    pub skipped_duplicates: Vec<String>,
    pub skipped_wildcards: Vec<String>,
}

pub fn ssh_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".ssh")
        .join("config")
}

pub fn import_ssh_config(
    existing: &[Connection],
) -> Result<ImportResult> {
    let path = ssh_config_path();
    if !path.exists() {
        anyhow::bail!("No SSH config found at {}", path.display());
    }

    let file = File::open(&path)
        .with_context(|| format!("Failed to open {}", path.display()))?;
    let mut reader = BufReader::new(file);
    let config = SshConfig::default()
        .parse(&mut reader, ParseRule::ALLOW_UNKNOWN_FIELDS)
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    let mut result = ImportResult {
        imported: Vec::new(),
        skipped_duplicates: Vec::new(),
        skipped_wildcards: Vec::new(),
    };

    for host in config.get_hosts() {
        // Get the first pattern clause as the name
        let first_clause = match host.pattern.first() {
            Some(c) => c,
            None => continue,
        };
        let pattern = &first_clause.pattern;

        // Skip wildcard entries
        if pattern.contains('*') || pattern.contains('?') {
            result.skipped_wildcards.push(pattern.clone());
            continue;
        }

        let params = config.query(pattern);
        let hostname = params
            .host_name
            .clone()
            .unwrap_or_else(|| pattern.clone());
        let port = params.port.unwrap_or(22);
        let user = params.user.clone();

        // Check for duplicates
        let is_dup = existing.iter().any(|c| {
            c.host == hostname && c.user == user && c.port == port
        });
        if is_dup {
            result.skipped_duplicates.push(pattern.clone());
            continue;
        }

        let mut conn = Connection::new(pattern.clone(), hostname);
        conn.port = port;
        conn.user = user;
        conn.identity_file = params.identity_file.and_then(|files| {
            files.into_iter().next().map(|p| p.to_string_lossy().to_string())
        });

        result.imported.push(conn);
    }

    Ok(result)
}
