use crate::model::Connection;
use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Command;

const KEY_TYPES: &[&str] = &["id_ed25519", "id_ecdsa", "id_rsa"];

pub struct KeySetupResult {
    pub private_key_path: String,
}

/// Find an existing SSH key, preferring ed25519 > ecdsa > rsa.
pub fn find_existing_key() -> Option<PathBuf> {
    let ssh_dir = dirs::home_dir()?.join(".ssh");
    for key_name in KEY_TYPES {
        let path = ssh_dir.join(key_name);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

/// Generate a new ed25519 key pair. TUI must be suspended before calling this.
pub fn generate_key() -> Result<PathBuf> {
    // Check ssh-keygen exists
    #[cfg(not(target_os = "windows"))]
    let check = Command::new("which").arg("ssh-keygen").output();
    #[cfg(target_os = "windows")]
    let check = Command::new("where").arg("ssh-keygen").output();

    match check {
        Ok(output) if !output.status.success() => {
            bail!("ssh-keygen not found in PATH. Please install OpenSSH.");
        }
        Err(e) => bail!("Failed to check for ssh-keygen: {}", e),
        _ => {}
    }

    let key_path = dirs::home_dir()
        .context("Could not determine home directory")?
        .join(".ssh")
        .join("id_ed25519");

    println!("Generating new ed25519 SSH key pair...\n");

    let status = Command::new("ssh-keygen")
        .arg("-t")
        .arg("ed25519")
        .arg("-f")
        .arg(&key_path)
        .status()
        .context("Failed to run ssh-keygen")?;

    if !status.success() {
        bail!("ssh-keygen exited with non-zero status. Key generation cancelled or failed.");
    }

    Ok(key_path)
}

/// Deploy the public key to a remote server. TUI must be suspended before calling this.
pub fn deploy_key(conn: &Connection, private_key_path: &PathBuf) -> Result<()> {
    let pub_key_path = private_key_path.with_extension("pub");
    if !pub_key_path.exists() {
        bail!(
            "Public key not found at {}",
            pub_key_path.display()
        );
    }

    #[cfg(not(target_os = "windows"))]
    {
        deploy_unix(conn, &pub_key_path)?;
    }

    #[cfg(target_os = "windows")]
    {
        deploy_windows(conn, &pub_key_path)?;
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn deploy_unix(conn: &Connection, pub_key_path: &PathBuf) -> Result<()> {
    // Check ssh-copy-id exists
    let check = Command::new("which").arg("ssh-copy-id").output();
    match check {
        Ok(output) if !output.status.success() => {
            bail!("ssh-copy-id not found in PATH. Please install it (usually part of openssh-client).");
        }
        Err(e) => bail!("Failed to check for ssh-copy-id: {}", e),
        _ => {}
    }

    let mut cmd = Command::new("ssh-copy-id");
    cmd.arg("-i").arg(pub_key_path);

    if conn.port != 22 {
        cmd.arg("-p").arg(conn.port.to_string());
    }

    let target = match &conn.user {
        Some(user) => format!("{}@{}", user, conn.host),
        None => conn.host.clone(),
    };
    cmd.arg(&target);

    println!("Deploying public key to {}...\n", target);

    let status = cmd.status().context("Failed to run ssh-copy-id")?;

    if !status.success() {
        bail!("ssh-copy-id failed. Check your password and try again.");
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn deploy_windows(conn: &Connection, pub_key_path: &PathBuf) -> Result<()> {
    let pub_key = std::fs::read_to_string(pub_key_path)
        .with_context(|| format!("Failed to read {}", pub_key_path.display()))?;
    let pub_key = pub_key.trim();

    let remote_cmd = format!(
        "mkdir -p ~/.ssh && echo '{}' >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys && chmod 700 ~/.ssh",
        pub_key
    );

    let target = match &conn.user {
        Some(user) => format!("{}@{}", user, conn.host),
        None => conn.host.clone(),
    };

    println!("Deploying public key to {}...\n", target);

    let mut cmd = Command::new("ssh");
    if conn.port != 22 {
        cmd.arg("-p").arg(conn.port.to_string());
    }
    cmd.arg(&target).arg(&remote_cmd);

    let status = cmd.status().context("Failed to run SSH for key deployment")?;

    if !status.success() {
        bail!("Key deployment failed. Check your password and try again.");
    }

    Ok(())
}

/// Full key auth setup flow. TUI must be suspended before calling this.
/// Returns the private key path on success.
pub fn setup_key_auth(conn: &Connection) -> Result<KeySetupResult> {
    // Step 1: Find or generate key
    let key_path = match find_existing_key() {
        Some(path) => {
            println!("Found existing SSH key: {}\n", path.display());
            path
        }
        None => {
            println!("No SSH key found. Let's create one.\n");
            generate_key()?
        }
    };

    // Step 2: Deploy to server
    deploy_key(conn, &key_path)?;

    println!("\nKey auth setup complete! Future connections will use key-based authentication.");

    Ok(KeySetupResult {
        private_key_path: key_path.to_string_lossy().to_string(),
    })
}
