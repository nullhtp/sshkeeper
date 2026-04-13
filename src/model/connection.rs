use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub name: String,
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub ssh_options: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy_jump: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_port() -> u16 {
    22
}

impl Connection {
    pub fn new(name: String, host: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            host,
            port: 22,
            user: None,
            identity_file: None,
            group: None,
            tags: Vec::new(),
            ssh_options: BTreeMap::new(),
            proxy_jump: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn ssh_command(&self) -> String {
        let mut parts = vec!["ssh".to_string()];
        if self.port != 22 {
            parts.push(format!("-p {}", self.port));
        }
        if let Some(ref user) = self.user {
            parts.push(format!("-l {user}"));
        }
        if let Some(ref key) = self.identity_file {
            parts.push(format!("-i {key}"));
        }
        if let Some(ref jump) = self.proxy_jump {
            parts.push(format!("-J {jump}"));
        }
        for (key, val) in &self.ssh_options {
            parts.push(format!("-o {key}={val}"));
        }
        parts.push(self.host.clone());
        parts.join(" ")
    }

    pub fn matches_query(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        self.name.to_lowercase().contains(&q)
            || self.host.to_lowercase().contains(&q)
            || self
                .group
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(&q)
            || self.tags.iter().any(|t| t.to_lowercase().contains(&q))
    }
}
