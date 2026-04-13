## Why

The app currently uses platform-specific config directories (`~/Library/Application Support/sshkeeper/` on macOS, `~/.config/sshkeeper/` on Linux) via `dirs::config_dir()`. This makes config files hard to find and doesn't follow the convention of similar CLI tools. Tools like SSH itself (`~/.ssh/`), Docker (`~/.docker/`), Kubernetes (`~/.kube/`), and AWS CLI (`~/.aws/`) all use a dotfolder in the user's home directory. SSHKeeper should do the same with `~/.sshkeeper/`.

## What Changes

- Change config directory from `dirs::config_dir()/sshkeeper/` to `~/.sshkeeper/`
- Auto-migrate existing config files from the old location on first run
- Update all references (README, specs)

## Capabilities

### New Capabilities

### Modified Capabilities
- `toml-storage`: Config directory changes from platform config dir to `~/.sshkeeper/`

## Impact

- `src/storage/toml_storage.rs` — change path resolution from `dirs::config_dir()` to `dirs::home_dir()/.sshkeeper/`
- `src/storage/transfer_history.rs` — same path change
- Both modules: add one-time migration from old path
- `README.md` — update config location docs
