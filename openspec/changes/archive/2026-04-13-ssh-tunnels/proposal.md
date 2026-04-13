## Why

SSH tunnels (port forwarding) are the second most common SSH workflow after interactive sessions, yet managing them requires remembering cryptic `-L`/`-R`/`-D` flags and keeping terminal windows open. SSHKeeper already manages connections — extending it to manage tunnels per connection turns it into a daily-use tool for developers who depend on remote databases, staging APIs, and internal services.

## What Changes

- New `Tunnel` data model for local (`-L`), remote (`-R`), and dynamic SOCKS (`-D`) forwarding rules
- Tunnels are stored per-connection in `connections.toml` alongside existing connection data
- New TUI screen for browsing, adding, editing, and deleting tunnel configurations
- Start/stop tunnels from the TUI with live status indicators (running/stopped)
- Tunnel processes run as background child processes managed by the app
- New keybinding on the Detail screen (`T` or `u`) to open the tunnel manager
- Status bar shows count of active tunnels across all connections

## Capabilities

### New Capabilities
- `tunnel-model`: Data model for tunnel configurations (local/remote/dynamic forwarding types, bind addresses, ports, remote targets) and persistence in TOML storage
- `tunnel-manager-ui`: TUI screen for managing tunnels — list, add, edit, delete, start/stop with status indicators
- `tunnel-backend`: Background SSH tunnel process lifecycle — spawning `ssh -N -L/-R/-D` processes, health monitoring, and cleanup on app exit

### Modified Capabilities
- `connection-model`: Add `tunnels` field (Vec of tunnel configs) to the Connection struct
- `toml-storage`: Serialize/deserialize nested tunnel configs within connection entries

## Impact

- **Model layer**: `Connection` struct gains a `tunnels: Vec<Tunnel>` field (backwards-compatible — defaults to empty vec)
- **Storage**: `connections.toml` format extended with nested tunnel tables; existing files load without migration
- **SSH module**: New `tunnel.rs` for building and managing `ssh -N` processes
- **UI module**: New `tunnel.rs` screen; `detail.rs` gains tunnel action keybinding; `app.rs` gains `Screen::Tunnels` variant and background process tracking
- **Dependencies**: No new crates needed — `std::process::Child` handles background processes
- **Platforms**: Works on macOS, Linux, Windows (all have `ssh -L/-R/-D` support)
