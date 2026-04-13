## Context

SSHKeeper is a Rust TUI (ratatui + crossterm) that manages SSH connections — browse, connect, edit, import, and set up key auth. It already shells out to the system `ssh` binary for connections and `ssh-keygen`/`ssh-copy-id` for key setup. File transfer is the most common missing workflow: users must leave the TUI, remember connection params, and manually run `scp`.

The app uses an enum-based screen state machine (`Screen::Browse`, `Screen::Detail`, `Screen::Editor`) with action enums driving transitions. SSH commands are built from `Connection` fields and executed by suspending the TUI.

## Goals / Non-Goals

**Goals:**
- Let users upload local files/directories to a remote server and download remote files/directories to a local path
- Reuse all existing connection parameters (host, port, user, identity file, proxy jump, SSH options) for SCP commands
- Provide a simple, keyboard-driven UX: pick direction (upload/download), browse local filesystem via a tree navigator to select paths, enter remote path, confirm
- Support recursive directory transfers
- Remember recent transfer paths per connection for quick re-use

**Non-Goals:**
- Real-time progress bars with percentage (SCP progress requires PTY allocation and parsing — out of scope for v1; we show a "transferring..." state and success/failure)
- SFTP interactive browsing of remote filesystem (would require libssh2 or similar — too complex for this change)
- Drag-and-drop or mouse interaction
- Resume/retry of partial transfers
- Bulk multi-file selection from different directories

## Decisions

### 1. Use system `scp` binary (same pattern as SSH execution)

**Rationale:** The app already validates and uses the system `ssh` binary. SCP accepts identical auth parameters (`-P` for port, `-i` for identity, `-o` for options, `-J` for proxy jump). This avoids adding a Rust SSH library dependency and keeps the architecture consistent.

**Alternative considered:** Using `libssh2` crate for native SFTP — rejected because it adds a heavy native dependency, complicates cross-platform builds, and the existing pattern works well.

### 2. New `Screen::Transfer` state with split layout

**Rationale:** A dedicated screen (like Detail and Editor) keeps the transfer workflow focused. The screen uses a split layout:
- **Left pane**: Local file tree browser — a collapsible tree view of the local filesystem with keyboard navigation (j/k or arrows to move, Enter to expand/collapse directories, Space to select a file or directory)
- **Right pane**: Transfer form with direction toggle (Upload/Download), selected local path (populated from tree), remote path text input, recursive auto-detection (on when a directory is selected), and recent history list
- Navigation: Tab switches focus between left tree pane and right form pane. Esc cancels. Enter on the form executes.

The tree starts at the user's home directory and allows navigating up to parent with `..` or typing a path to jump directly. Selected item is highlighted and its full path auto-fills the local path field.

**Alternative considered:** Tab-completion on a text input — rejected because navigating unfamiliar directory structures by typing is slow and error-prone. A visual tree lets users discover and confirm paths without memorizing them.

### 3. Transfer history stored in a separate TOML file

**Rationale:** Transfer history is auxiliary data that shouldn't bloat the connection model. Store in `~/.config/sshkeeper/transfer_history.toml` keyed by connection ID. Each entry stores direction, local path, remote path, recursive flag, and timestamp. Keep last 10 entries per connection.

**Alternative considered:** Adding history fields directly to `Connection` struct — rejected to keep the core model clean and serialization simple. The connection model only gets a lightweight reference (no structural changes needed after reconsidering).

### 4. SCP command builder in `ssh/` module

**Rationale:** New `ssh/transfer.rs` module with a `build_scp_command()` function that takes a `Connection` + transfer params and returns a `Command`. Mirrors `ssh/system.rs` pattern. The TUI suspends (like SSH connect and key setup) to let SCP run interactively in the terminal, showing its native progress output.

### 5. Local file tree browser component

**Rationale:** A tree view is the most intuitive way to pick files. Implementation uses `std::fs::read_dir` to lazily load directory contents on expand (not upfront). Each node tracks: path, name, is_directory, expanded (bool), and children (Vec). Directories sort before files, both alphabetical. Hidden files (dotfiles) are hidden by default with `h` to toggle visibility. The tree is rendered as indented lines with `▸`/`▾` arrows for directories. This component lives in `src/ui/file_tree.rs` and is reusable.

## Risks / Trade-offs

- **[SCP not installed]** → Mitigation: validate `scp` binary at startup (same as `ssh` validation). Show clear error if missing.
- **[Large file transfers block the TUI]** → Mitigation: TUI is suspended during transfer (same as SSH sessions), so the user sees SCP's native output. This is acceptable and consistent with the existing connect flow.
- **[Remote path typos]** → Mitigation: SCP will error on invalid remote paths; we capture exit code and show error in status bar. Recent paths history reduces re-typing.
- **[Large directories in file tree]** → Mitigation: lazy loading (only read contents on expand), cap visible entries per directory at 200 with a "(N more...)" indicator. Collapse reclaims memory.
