## Why

SSHKeeper has no codebase yet — it's a greenfield project. We need to establish the Rust project structure, core dependencies, and foundational modules so that all subsequent features build on a solid, consistent base. Starting with the right architecture avoids costly rewrites later and sets conventions for the entire project lifecycle.

## What Changes

- Initialize a Rust project with Cargo workspace and cross-platform build targets
- Set up Ratatui + crossterm as the TUI framework with an event-loop architecture
- Implement TOML-based connection profile storage with serde serialization
- Create an SSH backend trait with a system-SSH implementation (spawns the user's `ssh` binary)
- Build `~/.ssh/config` import to bootstrap connections from existing SSH configurations
- Establish module structure: `ui/`, `model/`, `storage/`, `ssh/`
- Implement core TUI states: Browse (list), Detail (view), Search (filter), Edit (form), Connect (session)

## Capabilities

### New Capabilities
- `connection-model`: Connection profile data model — struct, serialization, grouping, tags
- `tui-shell`: TUI application shell — event loop, state machine, screen routing, terminal setup/teardown
- `connection-list`: Interactive connection list — browsing, keyboard navigation, search/filter
- `connection-editor`: Add/edit connection profiles via TUI forms
- `ssh-backend`: SSH connection execution — trait-based backend with system SSH spawner
- `ssh-config-import`: Import connections from `~/.ssh/config`
- `toml-storage`: Persistent storage — read/write connection profiles to TOML files

### Modified Capabilities

## Impact

- **New files**: Entire Rust project scaffold (`Cargo.toml`, `src/**/*.rs`)
- **Dependencies**: ratatui, crossterm, serde, toml, ssh2-config, dirs, anyhow
- **Systems**: Reads `~/.ssh/config` (read-only), writes to `~/.config/sshkeeper/` (or platform equivalent)
- **No breaking changes**: Greenfield project, nothing to break
