## Context

SSHKeeper manages SSH connections via a TUI built with Ratatui. Users can connect, transfer files, run quick actions, and set up key auth. The app stores connections as TOML with serde serialization and spawns SSH/SCP as child processes, suspending the TUI during execution.

Tunnels differ from existing SSH operations: they are long-running background processes that must coexist with the TUI. This is the first feature requiring background process management within the app.

## Goals / Non-Goals

**Goals:**
- Let users define, persist, and manage SSH tunnel configurations per connection
- Start/stop tunnels from the TUI without leaving the interface
- Show live tunnel status (running/stopped) in the UI
- Clean up all tunnel processes on app exit
- Support local (`-L`), remote (`-R`), and dynamic SOCKS (`-D`) forwarding

**Non-Goals:**
- Auto-reconnect on tunnel failure (future enhancement)
- Multiplexing tunnels over a single SSH connection (use separate `ssh -N` per tunnel for simplicity)
- Tunnel health checks via actual traffic probing (just check if process is alive)
- Exposing tunnels as a CLI subcommand (TUI-only for now)

## Decisions

### 1. One `ssh -N` process per tunnel (not multiplexed)

Each tunnel starts its own `ssh -N -L/-R/-D ...` process. This is simpler than multiplexing multiple forwards over a single SSH connection via ControlMaster.

**Alternative considered**: SSH multiplexing with `-o ControlMaster=auto`. More efficient but adds complexity around socket management, platform differences, and partial failure handling. Not worth it for v1.

**Rationale**: Independent processes are easier to start/stop individually, have clear status (process alive = tunnel up), and avoid shared-state bugs. The overhead of multiple SSH connections is negligible for typical tunnel counts (<10).

### 2. Background `Child` process tracking in `App` state

Store a `HashMap<String, Child>` in `App` (tunnel_id → Child process). Check liveness by calling `child.try_wait()` which is non-blocking.

**Alternative considered**: Daemonizing tunnel processes or using a separate manager thread. Over-engineered for a TUI app — the tunnels should live and die with the app session.

**Rationale**: `std::process::Child` is the simplest approach. Processes are cleaned up when the app drops them (or explicitly via `child.kill()` in a cleanup handler).

### 3. Tunnel config embedded in Connection struct

Add `tunnels: Vec<Tunnel>` to `Connection`. Tunnels serialize as nested TOML arrays within each connection's table.

```toml
[connection-uuid]
name = "prod-db"
host = "bastion.example.com"
# ... other fields ...

[[connection-uuid.tunnels]]
id = "tunnel-uuid"
name = "PostgreSQL"
tunnel_type = "local"
bind_address = "127.0.0.1"
bind_port = 5432
remote_host = "db.internal"
remote_port = 5432
```

**Alternative considered**: Separate `tunnels.toml` file. Would require cross-referencing connection IDs and complicate the storage layer for little benefit.

**Rationale**: Tunnels are inherently tied to connections (they use the connection's host, port, user, key). Co-locating them keeps the data model cohesive and storage simple.

### 4. Tunnel TUI as a new Screen variant

Add `Screen::Tunnels { conn_id, state }` similar to the existing Transfer screen. Accessible from the Detail screen via a keybinding.

The tunnel list screen shows all tunnels for a connection with status indicators. Inline add/edit form (similar to connection editor) for creating tunnels.

**Rationale**: Follows established patterns in the codebase. Each feature gets its own screen with its own state struct and action enum.

### 5. SSH command construction

Build tunnel SSH commands by reusing `SystemSshBackend::build_command()` for common connection args (port, user, key, jump host, options), then appending `-N` (no remote command) and the appropriate forwarding flag:
- Local: `-L [bind_addr:]bind_port:remote_host:remote_port`
- Remote: `-R [bind_addr:]bind_port:remote_host:remote_port`
- Dynamic: `-D [bind_addr:]bind_port`

### 6. Process cleanup strategy

- On individual tunnel stop: `child.kill()` + `child.wait()`
- On app exit: iterate all active tunnels and kill them in the `Drop` impl or explicit cleanup before exit
- On tunnel screen refresh: `try_wait()` to detect processes that died unexpectedly, update status accordingly

## Risks / Trade-offs

- **[Zombie processes on crash]** → If the app panics or is killed with SIGKILL, tunnel processes become orphans. Mitigation: tunnels use `ssh -N` which will eventually die when the terminal session ends. This is acceptable for v1.
- **[Port conflicts]** → User tries to bind a port already in use. Mitigation: the SSH process will fail immediately with an error — detect this via `try_wait()` shortly after spawn and show the error in the UI.
- **[No auto-reconnect]** → If the network drops, the tunnel dies silently. Mitigation: status check via `try_wait()` on each render cycle shows "stopped" status. User can manually restart. Auto-reconnect is a future enhancement.
- **[TOML format change]** → Adding nested arrays to `connections.toml`. Mitigation: `tunnels` defaults to empty vec with `skip_serializing_if`, so existing files load unchanged. Fully backwards-compatible.
