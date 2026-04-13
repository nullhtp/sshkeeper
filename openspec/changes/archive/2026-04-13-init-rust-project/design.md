## Context

SSHKeeper is a greenfield Rust project — no existing code. The project needs a complete foundation: build system, TUI framework, data model, storage layer, SSH execution, and config import. All future features will build on the patterns and module boundaries established here.

The target is a single static binary with zero runtime dependencies, working identically on macOS, Linux, and Windows. Users range from developers managing a handful of servers to ops engineers with hundreds of connections.

## Goals / Non-Goals

**Goals:**
- Establish a clean, modular Rust project that compiles on all three platforms
- Create a TUI shell with Ratatui that handles terminal lifecycle and state routing
- Define a connection profile model that covers all common SSH parameters
- Implement TOML-based persistent storage in platform-appropriate config directories
- Build an SSH backend trait with a system-SSH implementation
- Import existing connections from `~/.ssh/config`
- Set patterns (error handling, module organization, testing) that scale as features grow

**Non-Goals:**
- Native SSH library integration (russh) — future work behind the trait
- Clipboard, mouse support, or theming — not in initial scope
- Connection health checks or background monitoring
- Multi-user or networked sync of connection profiles
- Package manager distribution (Homebrew, apt, scoop) — post-MVP

## Decisions

### 1. Ratatui + crossterm for TUI

**Choice**: Ratatui with crossterm backend

**Alternatives considered**:
- **cursive**: Higher-level API but less flexible layout, smaller ecosystem
- **termion backend**: Unix-only, no Windows support
- **Bubble Tea (Go)**: Would require switching language entirely

**Rationale**: Ratatui is the most actively maintained Rust TUI framework. crossterm provides cross-platform terminal I/O including Windows. Immediate-mode rendering gives precise control over what renders each frame. The widget ecosystem (lists, tables, paragraphs, input) covers our needs.

### 2. Event loop with crossterm polling

**Choice**: Single-threaded event loop with `crossterm::event::poll()` and 250ms tick rate

**Rationale**: Simple, no async runtime needed. The app is I/O-idle most of the time (waiting for keypresses). A tick-based loop lets us add periodic updates later (connection status) without restructuring. No need for tokio — this app doesn't do concurrent I/O.

### 3. App state machine

**Choice**: Enum-based state machine for screen routing

```
enum AppState {
    Browse,      // Main connection list
    Detail(id),  // View connection details
    Search,      // Active search/filter
    Edit(id),    // Edit existing / add new
    Connecting,  // SSH session active (TUI suspended)
}
```

**Rationale**: Explicit states prevent impossible state combinations. Each state owns its key bindings and render logic. Transitions are a single match expression in the update function.

### 4. TOML for connection storage

**Choice**: Single `connections.toml` file in platform config directory

**Alternatives considered**:
- **JSON**: Less human-readable, no comments
- **SQLite**: Overkill for <1000 records, adds a C dependency (or pure-Rust alternative)
- **YAML**: Whitespace-sensitive, common source of bugs

**Rationale**: TOML is human-editable, supports comments, maps cleanly to Rust structs via serde. Users can hand-edit their connections file. A single file avoids directory-per-connection complexity. At the expected scale (dozens to hundreds of connections), file I/O is instantaneous.

**Storage path**: `dirs::config_dir()` → `~/.config/sshkeeper/` (Linux), `~/Library/Application Support/sshkeeper/` (macOS), `%APPDATA%\sshkeeper\` (Windows).

### 5. SSH backend trait

**Choice**: Trait-based abstraction over SSH execution

```rust
trait SshBackend {
    fn connect(&self, profile: &Connection) -> Result<()>;
}
```

System implementation spawns `ssh` via `std::process::Command`, replaces the current process on Unix (`exec`), or spawns a child on Windows.

**Rationale**: Spawning the system SSH binary gives users their existing agent, config, and known_hosts for free. The trait boundary allows adding a native Rust SSH backend (russh) later without changing calling code. The system backend is the pragmatic default — maximum compatibility, minimum code.

### 6. SSH config import via ssh2-config

**Choice**: `ssh2-config` crate for parsing `~/.ssh/config`

**Rationale**: Handles Host patterns, ProxyJump, Include directives, and most OpenSSH options. Read-only — we never write to the user's SSH config. Imported connections are copied into SSHKeeper's own storage as independent profiles.

### 7. Error handling with anyhow

**Choice**: `anyhow::Result` for application-level errors, `thiserror` if we need typed errors at module boundaries later

**Rationale**: For an application (not a library), anyhow provides ergonomic error chaining with context. No need for a custom error enum at this stage. We can introduce thiserror at specific boundaries if error recovery logic demands it.

## Risks / Trade-offs

- **[Risk] Ratatui boilerplate** — More setup code than higher-level frameworks → Mitigated by establishing clear patterns in the initial scaffold that all future screens follow
- **[Risk] crossterm Windows quirks** — Windows terminal support has edge cases (legacy conhost) → Mitigated by targeting Windows Terminal / modern terminals; fallback behavior acceptable on legacy
- **[Risk] ssh2-config parsing gaps** — Some exotic SSH config directives may not parse → Mitigated by graceful skip of unparseable hosts with warnings; users can manually add those connections
- **[Risk] Single TOML file scaling** — Very large connection lists may make the file unwieldy to hand-edit → Acceptable for MVP; can split by group into multiple files later
- **[Trade-off] System SSH vs native** — Depends on user having `ssh` installed → Acceptable because target users are SSH-heavy and always have it; native backend is a future option via the trait
