## 1. Project Scaffold

- [x] 1.1 Initialize Cargo project with `cargo init`, configure `Cargo.toml` with all dependencies (ratatui, crossterm, serde, toml, ssh2-config, dirs, anyhow, uuid)
- [x] 1.2 Create module directory structure: `src/{ui, model, storage, ssh}/mod.rs`
- [x] 1.3 Set up `main.rs` with terminal setup/teardown (raw mode, alternate screen) and panic hook for terminal restoration

## 2. Connection Model

- [x] 2.1 Define `Connection` struct with all fields (id, name, host, port, user, identity_file, group, tags, ssh_options, proxy_jump, created_at, updated_at) and serde derives
- [x] 2.2 Implement `Connection::new()` with defaults (port 22, auto-generated UUID, timestamps)
- [x] 2.3 Define `ConnectionStore` struct to hold a collection of connections with lookup by id, filter by group, and search

## 3. TOML Storage

- [x] 3.1 Implement storage path resolution using `dirs::config_dir()` with directory auto-creation
- [x] 3.2 Implement `load()` — read and deserialize `connections.toml`, handle missing file (empty list) and corrupted file (error message, no overwrite)
- [x] 3.3 Implement `save()` — serialize and write via atomic temp-file-and-rename
- [x] 3.4 Ensure human-readable TOML output formatting (one table per connection, blank lines between)

## 4. SSH Backend

- [x] 4.1 Define `SshBackend` trait with `connect(&self, profile: &Connection) -> Result<()>`
- [x] 4.2 Implement `SystemSshBackend` — build SSH command from profile fields (-p, -l, -i, -J, -o flags)
- [x] 4.3 Implement terminal suspend/resume around SSH session (leave alternate screen, restore after session ends)
- [x] 4.4 Handle SSH binary not found with clear error message

## 5. SSH Config Import

- [x] 5.1 Implement `~/.ssh/config` parsing using ssh2-config crate — map Host entries to Connection profiles
- [x] 5.2 Skip wildcard `Host *` entries and handle missing config file gracefully
- [x] 5.3 Implement duplicate detection (match on host + user + port) to skip already-stored connections
- [x] 5.4 Wire import into TUI as a command accessible from Browse state

## 6. TUI Shell & Event Loop

- [x] 6.1 Implement event loop with crossterm polling (250ms tick rate) and key event dispatch
- [x] 6.2 Define `AppState` enum (Browse, Detail, Search, Edit, Connecting) and state transition logic
- [x] 6.3 Implement screen routing — delegate render and input handling to the active state's module

## 7. Connection List Screen (Browse)

- [x] 7.1 Implement connection list widget with ratatui — display name, host, group columns
- [x] 7.2 Implement keyboard navigation (j/k, arrows, g/G) with selection tracking
- [x] 7.3 Implement search/filter mode (/ to activate, Escape to cancel, fuzzy match on name/host/group/tags)
- [x] 7.4 Implement grouped view toggle (Tab to switch between flat and grouped display)
- [x] 7.5 Implement empty state message when no connections exist

## 8. Detail & Editor Screens

- [x] 8.1 Implement Detail screen — display all fields of the selected connection and the constructed SSH command
- [x] 8.2 Implement Edit/Add form with text input fields for each connection property
- [x] 8.3 Implement delete with confirmation prompt
- [x] 8.4 Wire Enter from Detail to trigger SSH connect, `e` to edit, `d` to delete

## 9. Integration & Polish

- [x] 9.1 Wire all modules together in main.rs — load storage, initialize app state, run event loop
- [x] 9.2 Add `q` / `Ctrl+C` global quit from Browse state
- [x] 9.3 Test cross-platform build (macOS, Linux, Windows targets)
