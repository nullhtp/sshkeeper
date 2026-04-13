use crate::model::Connection;
use anyhow::{bail, Result};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferDirection {
    Upload,
    Download,
}

pub fn validate_scp() -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    let check = Command::new("which").arg("scp").output();

    #[cfg(target_os = "windows")]
    let check = Command::new("where").arg("scp").output();

    match check {
        Ok(output) if !output.status.success() => {
            bail!("scp not found — install OpenSSH to use file transfer");
        }
        Err(e) => {
            bail!("Failed to check for scp binary: {}", e);
        }
        _ => Ok(()),
    }
}

pub fn build_scp_command(
    conn: &Connection,
    local_path: &str,
    remote_path: &str,
    direction: TransferDirection,
    recursive: bool,
) -> Command {
    let mut cmd = Command::new("scp");

    if recursive {
        cmd.arg("-r");
    }

    // SCP uses -P (uppercase) for port
    if conn.port != 22 {
        cmd.arg("-P").arg(conn.port.to_string());
    }
    if let Some(ref key) = conn.identity_file {
        cmd.arg("-i").arg(key);
    }
    if let Some(ref jump) = conn.proxy_jump {
        cmd.arg("-J").arg(jump);
    }
    for (key, val) in &conn.ssh_options {
        cmd.arg("-o").arg(format!("{}={}", key, val));
    }

    let remote = if let Some(ref user) = conn.user {
        format!("{}@{}:{}", user, conn.host, remote_path)
    } else {
        format!("{}:{}", conn.host, remote_path)
    };

    match direction {
        TransferDirection::Upload => {
            cmd.arg(local_path).arg(&remote);
        }
        TransferDirection::Download => {
            cmd.arg(&remote).arg(local_path);
        }
    }

    cmd
}
