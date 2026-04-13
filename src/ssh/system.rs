use super::SshBackend;
use crate::model::Connection;
use anyhow::{Context, Result, bail};
use std::process::Command;

pub struct SystemSshBackend;

impl SystemSshBackend {
    pub fn build_command(profile: &Connection) -> Command {
        let mut cmd = Command::new("ssh");
        if profile.port != 22 {
            cmd.arg("-p").arg(profile.port.to_string());
        }
        if let Some(ref user) = profile.user {
            cmd.arg("-l").arg(user);
        }
        if let Some(ref key) = profile.identity_file {
            cmd.arg("-i").arg(key);
        }
        if let Some(ref jump) = profile.proxy_jump {
            cmd.arg("-J").arg(jump);
        }
        for (key, val) in &profile.ssh_options {
            cmd.arg("-o").arg(format!("{key}={val}"));
        }
        cmd.arg(&profile.host);
        cmd
    }
}

impl SshBackend for SystemSshBackend {
    fn connect(&self, profile: &Connection) -> Result<()> {
        // Check if ssh binary exists
        #[cfg(not(target_os = "windows"))]
        let ssh_check = Command::new("which").arg("ssh").output();

        #[cfg(target_os = "windows")]
        let ssh_check = Command::new("where").arg("ssh").output();

        match ssh_check {
            Ok(output) if !output.status.success() => {
                bail!("SSH binary not found in PATH. Please install OpenSSH.");
            }
            Err(e) => {
                bail!("Failed to check for SSH binary: {e}");
            }
            _ => {}
        }

        let mut cmd = Self::build_command(profile);
        let status = cmd.status().context("Failed to launch SSH process")?;

        if !status.success() {
            if let Some(code) = status.code() {
                bail!("SSH exited with code {code}");
            }
        }
        Ok(())
    }
}
