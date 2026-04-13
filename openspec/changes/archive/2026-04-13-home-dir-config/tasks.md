## 1. Centralized config directory

- [x] 1.1 Add `config_dir()` function to `src/storage/mod.rs` that returns `~/.sshkeeper/` via `dirs::home_dir()` with auto-create
- [x] 1.2 Add `migrate_file()` helper that copies a file from old platform config dir to new dir if new doesn't exist

## 2. Update storage modules

- [x] 2.1 Update `TomlStorage::new()` to use `config_dir()` instead of `dirs::config_dir()`, call `migrate_file` for `connections.toml`
- [x] 2.2 Update `TransferHistory::load()` to use `config_dir()` instead of `dirs::config_dir()`, call `migrate_file` for `transfer_history.toml`

## 3. Docs

- [x] 3.1 Update README.md config location from `~/.config/sshkeeper/` to `~/.sshkeeper/`

## 4. Verification

- [x] 4.1 Build compiles with `cargo clippy`
