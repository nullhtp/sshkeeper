## 1. Tunnel Data Model

- [x] 1.1 Create `src/model/tunnel.rs` with `Tunnel` struct (id, name, tunnel_type enum, bind_address, bind_port, remote_host, remote_port) and serde serialization
- [x] 1.2 Implement `TunnelType` enum (Local, Remote, Dynamic) with lowercase serde serialization
- [x] 1.3 Implement `Tunnel::ssh_flag()` method that generates the `-L`/`-R`/`-D` flag string
- [x] 1.4 Implement validation: require remote_host/remote_port for Local/Remote types, forbid for Dynamic
- [x] 1.5 Add `tunnels: Vec<Tunnel>` field to `Connection` struct with `#[serde(default, skip_serializing_if = "Vec::is_empty")]`
- [x] 1.6 Update `Connection::new()` to initialize `tunnels` to empty vec
- [x] 1.7 Export `tunnel` module from `src/model/mod.rs`

## 2. Tunnel Backend

- [x] 2.1 Create `src/ssh/tunnel.rs` with `TunnelManager` struct holding `HashMap<String, Child>` (tunnel_id → process)
- [x] 2.2 Implement `TunnelManager::start()` — build SSH command using `SystemSshBackend::build_command()` + `-N` + tunnel flag, spawn with stderr piped
- [x] 2.3 Implement `TunnelManager::stop()` — kill process and reap with wait()
- [x] 2.4 Implement `TunnelManager::is_running()` — non-blocking `try_wait()` check, clean up dead processes
- [x] 2.5 Implement `TunnelManager::stop_all()` — kill all tracked processes (used on app exit)
- [x] 2.6 Implement `TunnelManager::active_count()` — return number of running tunnels
- [x] 2.7 Implement `TunnelManager::get_error()` — read stderr from a failed tunnel process for error reporting
- [x] 2.8 Export `tunnel` module from `src/ssh/mod.rs`

## 3. Tunnel Manager UI

- [x] 3.1 Create `src/ui/tunnels.rs` with `TunnelScreenState` struct (selected index, mode enum for list/add/edit)
- [x] 3.2 Implement tunnel list rendering — table with columns: name, type (L/R/D), bind addr:port, remote target, status indicator
- [x] 3.3 Implement status indicator rendering — green "ON" / red "OFF" based on `TunnelManager::is_running()`
- [x] 3.4 Implement add tunnel form — fields for name, type (dropdown), bind_address, bind_port, remote_host, remote_port
- [x] 3.5 Implement edit tunnel form — pre-populated with selected tunnel's values
- [x] 3.6 Implement key handling: Enter (toggle start/stop), `a` (add), `e` (edit), `d` (delete), Escape (back)
- [x] 3.7 Implement delete — stop tunnel if running, remove from connection's tunnels vec, save
- [x] 3.8 Define `TunnelAction` enum for screen actions (None, Back, Start, Stop, Add, Edit, Delete)
- [x] 3.9 Implement empty state message when no tunnels configured
- [x] 3.10 Implement help bar at bottom showing available keybindings

## 4. App Integration

- [x] 4.1 Add `Screen::Tunnels { conn_id, state }` variant to the Screen enum
- [x] 4.2 Add `TunnelManager` field to `App` struct, initialized as empty
- [x] 4.3 Add `DetailAction::ManageTunnels(String)` variant — triggered by `u` key on detail screen
- [x] 4.4 Handle `u` keypress in `detail.rs` to return `ManageTunnels` action
- [x] 4.5 Handle `Screen::Tunnels` in `App::render()` and `App::handle_key()`
- [x] 4.6 Implement tunnel start/stop/add/edit/delete handlers in `App` that delegate to `TunnelManager` and persist to storage
- [x] 4.7 Call `TunnelManager::stop_all()` before app exit in the main loop
- [x] 4.8 Show active tunnel count in browse screen status bar when tunnels are running
- [x] 4.9 Export `tunnels` module from `src/ui/mod.rs`

## 5. Testing & Verification

- [x] 5.1 Add unit tests for `Tunnel::ssh_flag()` — local, remote, and dynamic variants
- [x] 5.2 Add unit tests for tunnel validation (missing remote_host for local type, etc.)
- [x] 5.3 Add unit test for `Connection` serialization round-trip with tunnels
- [x] 5.4 Add unit test for `Connection` deserialization without tunnels field (backwards compat)
- [x] 5.5 Verify `cargo clippy` passes with no warnings
- [x] 5.6 Verify `cargo build` succeeds on all targets
