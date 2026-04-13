## Context

SSHKeeper stores `connections.toml` and `transfer_history.toml` in the platform config directory via `dirs::config_dir()`. Both `TomlStorage` and `TransferHistory` independently resolve this path. The `dirs` crate is already a dependency.

## Goals / Non-Goals

**Goals:**
- Store all config in `~/.sshkeeper/` on all platforms
- Silently migrate existing files from the old location on first run
- Single shared function for path resolution

**Non-Goals:**
- Supporting XDG_CONFIG_HOME or other overrides (keep it simple)
- Backward compatibility mode (old location stops working after migration)

## Decisions

### 1. Centralized config directory function

Create a single `config_dir()` function in the storage module that returns `~/.sshkeeper/`, used by both `TomlStorage` and `TransferHistory`. This eliminates the duplicated `dirs::config_dir()` calls.

```rust
pub fn config_dir() -> Result<PathBuf> {
    let dir = dirs::home_dir()
        .context("Could not determine home directory")?
        .join(".sshkeeper");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}
```

### 2. Auto-migration on first run

When the new `~/.sshkeeper/` directory doesn't yet contain the expected files, check the old platform-specific location. If files exist there, copy them to the new location. Don't delete the old files — let the user clean up manually if they want.

Migration runs once per file, silently. No user prompt needed.

### 3. Drop `dirs::config_dir()` usage entirely

After this change, the only `dirs` function used is `home_dir()`. The `config_dir()` call is removed.

## Risks / Trade-offs

- **[Windows path]** → `~/.sshkeeper/` is a dotfolder, unusual on Windows but works fine. CLI tools like `.docker` do the same.
- **[Migration edge case]** → If user has files in both old and new locations, new location wins (no overwrite of existing files).
