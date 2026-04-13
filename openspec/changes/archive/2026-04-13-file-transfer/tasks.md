## 1. SCP Command Builder

- [x] 1.1 Create `src/ssh/transfer.rs` module with `build_scp_command()` that takes a `Connection`, local path, remote path, direction (upload/download), and recursive flag — returns a `std::process::Command` with port (`-P`), identity file (`-i`), proxy jump (`-J`), and SSH options (`-o`)
- [x] 1.2 Add SCP binary validation function (reuse pattern from `SystemSshBackend::validate()` but for `scp`)
- [x] 1.3 Register `transfer` module in `src/ssh/mod.rs`

## 2. Transfer History Storage

- [x] 2.1 Define `TransferEntry` struct (direction, local_path, remote_path, recursive, timestamp) and `TransferHistory` struct (map of connection ID to Vec<TransferEntry>) in a new `src/storage/transfer_history.rs`
- [x] 2.2 Implement TOML load/save for transfer history at `~/.config/sshkeeper/transfer_history.toml` (follow `TomlStorage` patterns)
- [x] 2.3 Add method to push a new entry (capping at 10 per connection, removing oldest)
- [x] 2.4 Register module in `src/storage/mod.rs`

## 3. File Tree Browser Component

- [x] 3.1 Create `src/ui/file_tree.rs` with `FileTree` struct: root path, flat list of visible `TreeNode` entries (path, name, is_dir, depth, expanded), cursor index, show_hidden flag
- [x] 3.2 Implement lazy directory loading: `expand()` reads `std::fs::read_dir` on demand, sorts dirs-first then alphabetical, caps at 200 entries per directory
- [x] 3.3 Implement collapse: hides all children and nested descendants from the visible list
- [x] 3.4 Implement keyboard navigation: j/k or Up/Down to move cursor, Enter/Right to expand dir, Enter/Left to collapse dir, Left on collapsed dir to go to parent, Space to select, `h` to toggle hidden files, `/` to jump-to-path
- [x] 3.5 Implement rendering: indented lines with `▸`/`▾` for dirs, file icon distinction, highlighted cursor line, selected item indicator
- [x] 3.6 Register module in `src/ui/mod.rs`

## 4. Transfer Screen UI

- [x] 4.1 Add `Screen::Transfer(String)` variant (holding connection ID) to the screen enum in `src/ui/app.rs`
- [x] 4.2 Create `src/ui/transfer.rs` with `TransferScreen` struct holding: `FileTree`, direction (Upload/Download), selected local path, remote_path text input, recursive flag, focused pane (Tree/Form), history entries, selected history index
- [x] 4.3 Implement split-pane rendering: left pane shows FileTree widget, right pane shows transfer form (direction toggle, local path display, remote path input, recursive indicator, history list, help line)
- [x] 4.4 Implement keyboard routing: Tab switches focus between tree pane and form pane; when tree focused, keys go to FileTree; when form focused, arrow keys navigate fields, `d` toggles direction, `r` toggles recursive, Enter executes
- [x] 4.5 Wire Space-select in file tree to auto-populate local path and auto-set recursive flag (on for dirs, off for files)
- [x] 4.6 Implement history quick-fill: selecting a history entry populates direction, local path, remote path, and recursive flag
- [x] 4.7 Register module in `src/ui/mod.rs`

## 5. App Integration

- [x] 5.1 Add `TransferAction` enum (Execute, Cancel, Back) and wire `Screen::Transfer` into the main event loop in `src/ui/app.rs`
- [x] 5.2 Add `t` keybinding to the detail screen (`src/ui/detail.rs`) that emits a new `DetailAction::Transfer` variant
- [x] 5.3 Handle `DetailAction::Transfer`: validate SCP binary, show error if missing, otherwise transition to `Screen::Transfer`
- [x] 5.4 Handle `TransferAction::Execute`: suspend TUI, run SCP command, restore TUI, record success in transfer history, show status message, return to detail screen on success (stay on transfer screen on failure)
- [x] 5.5 Load transfer history at app startup (in `main.rs`) and pass to app state; save after each successful transfer

## 6. Theme and Polish

- [x] 6.1 Add transfer-related styles to `src/ui/theme.rs` (pane borders for focused/unfocused, direction highlight, tree indentation guides, recursive indicator, selected path style)
- [x] 6.2 Add `t Transfer` to the detail screen help line
