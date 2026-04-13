use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TunnelType {
    Local,
    Remote,
    Dynamic,
}

impl TunnelType {
    pub fn label(self) -> &'static str {
        match self {
            Self::Local => "L",
            Self::Remote => "R",
            Self::Dynamic => "D",
        }
    }
}

impl std::fmt::Display for TunnelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "Local"),
            Self::Remote => write!(f, "Remote"),
            Self::Dynamic => write!(f, "Dynamic"),
        }
    }
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tunnel {
    pub id: String,
    pub name: String,
    pub tunnel_type: TunnelType,
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
    pub bind_port: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_port: Option<u16>,
}

fn default_bind_address() -> String {
    "127.0.0.1".to_string()
}

impl Tunnel {
    pub fn new(name: String, tunnel_type: TunnelType, bind_port: u16) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            tunnel_type,
            bind_address: default_bind_address(),
            bind_port,
            remote_host: None,
            remote_port: None,
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self.tunnel_type {
            TunnelType::Local | TunnelType::Remote => {
                if self.remote_host.is_none() {
                    bail!(
                        "{} tunnel requires a remote host",
                        self.tunnel_type.label()
                    );
                }
                if self.remote_port.is_none() {
                    bail!(
                        "{} tunnel requires a remote port",
                        self.tunnel_type.label()
                    );
                }
            }
            TunnelType::Dynamic => {}
        }
        Ok(())
    }

    pub fn ssh_flag(&self) -> String {
        match self.tunnel_type {
            TunnelType::Local => {
                format!(
                    "-L {}:{}:{}:{}",
                    self.bind_address,
                    self.bind_port,
                    self.remote_host.as_deref().unwrap_or("localhost"),
                    self.remote_port.unwrap_or(0)
                )
            }
            TunnelType::Remote => {
                format!(
                    "-R {}:{}:{}:{}",
                    self.bind_address,
                    self.bind_port,
                    self.remote_host.as_deref().unwrap_or("localhost"),
                    self.remote_port.unwrap_or(0)
                )
            }
            TunnelType::Dynamic => {
                format!("-D {}:{}", self.bind_address, self.bind_port)
            }
        }
    }

    pub fn remote_target(&self) -> String {
        match self.tunnel_type {
            TunnelType::Local | TunnelType::Remote => {
                format!(
                    "{}:{}",
                    self.remote_host.as_deref().unwrap_or("?"),
                    self.remote_port.map_or("?".to_string(), |p| p.to_string())
                )
            }
            TunnelType::Dynamic => "-".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_tunnel_ssh_flag() {
        let mut t = Tunnel::new("pg".into(), TunnelType::Local, 5432);
        t.remote_host = Some("db.internal".into());
        t.remote_port = Some(5432);
        assert_eq!(t.ssh_flag(), "-L 127.0.0.1:5432:db.internal:5432");
    }

    #[test]
    fn test_remote_tunnel_ssh_flag() {
        let mut t = Tunnel::new("webhook".into(), TunnelType::Remote, 8080);
        t.remote_host = Some("localhost".into());
        t.remote_port = Some(3000);
        assert_eq!(t.ssh_flag(), "-R 127.0.0.1:8080:localhost:3000");
    }

    #[test]
    fn test_dynamic_tunnel_ssh_flag() {
        let t = Tunnel::new("socks".into(), TunnelType::Dynamic, 1080);
        assert_eq!(t.ssh_flag(), "-D 127.0.0.1:1080");
    }

    #[test]
    fn test_validate_local_missing_remote_host() {
        let t = Tunnel::new("test".into(), TunnelType::Local, 5432);
        assert!(t.validate().is_err());
    }

    #[test]
    fn test_validate_local_missing_remote_port() {
        let mut t = Tunnel::new("test".into(), TunnelType::Local, 5432);
        t.remote_host = Some("db".into());
        assert!(t.validate().is_err());
    }

    #[test]
    fn test_validate_local_ok() {
        let mut t = Tunnel::new("test".into(), TunnelType::Local, 5432);
        t.remote_host = Some("db".into());
        t.remote_port = Some(5432);
        assert!(t.validate().is_ok());
    }

    #[test]
    fn test_validate_dynamic_ok() {
        let t = Tunnel::new("socks".into(), TunnelType::Dynamic, 1080);
        assert!(t.validate().is_ok());
    }

    #[test]
    fn test_tunnel_serialization_roundtrip() {
        let mut t = Tunnel::new("pg".into(), TunnelType::Local, 5432);
        t.remote_host = Some("db.internal".into());
        t.remote_port = Some(5432);
        let toml_str = toml::to_string(&t).unwrap();
        let deserialized: Tunnel = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.name, t.name);
        assert_eq!(deserialized.tunnel_type, TunnelType::Local);
        assert_eq!(deserialized.bind_port, 5432);
        assert_eq!(deserialized.remote_host.as_deref(), Some("db.internal"));
        assert_eq!(deserialized.remote_port, Some(5432));
    }
}
