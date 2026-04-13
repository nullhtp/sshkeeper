## ADDED Requirements

### Requirement: Transfer screen accessible from connection detail
The system SHALL provide a Transfer action from the connection detail screen, activated by pressing `t`. This SHALL navigate to a new Transfer screen scoped to the selected connection.

#### Scenario: Open transfer screen
- **WHEN** the user presses `t` on the connection detail screen
- **THEN** the system SHALL display the Transfer screen with the connection name in the title and default to Upload mode

### Requirement: Transfer screen layout
The Transfer screen SHALL use a split layout with a local file tree browser on the left pane and a transfer form on the right pane. The user SHALL switch focus between panes using Tab.

#### Scenario: Initial layout
- **WHEN** the Transfer screen opens
- **THEN** the left pane SHALL show a file tree rooted at the user's home directory, and the right pane SHALL show the transfer form with Upload mode selected

#### Scenario: Switch focus between panes
- **WHEN** the user presses Tab
- **THEN** focus SHALL alternate between the file tree pane and the transfer form pane, with the focused pane visually highlighted

### Requirement: Local file tree browser
The system SHALL display a navigable tree view of the local filesystem in the left pane. Directories SHALL be expandable/collapsible. Directories SHALL sort before files, both sorted alphabetically.

#### Scenario: Navigate tree with keyboard
- **WHEN** the file tree pane is focused
- **THEN** the user SHALL navigate with j/k or Up/Down arrows to move the cursor, and the selected entry SHALL be highlighted

#### Scenario: Expand a directory
- **WHEN** the user presses Enter or Right arrow on a collapsed directory
- **THEN** the directory's contents SHALL be loaded and displayed as indented children with a `▾` indicator

#### Scenario: Collapse a directory
- **WHEN** the user presses Enter or Left arrow on an expanded directory
- **THEN** the directory's children SHALL be hidden and the indicator SHALL change to `▸`

#### Scenario: Select a file or directory
- **WHEN** the user presses Space on a file or directory in the tree
- **THEN** the full path SHALL be set as the local path in the transfer form, and if a directory is selected, recursive mode SHALL be automatically enabled

#### Scenario: Navigate to parent directory
- **WHEN** the user presses Left arrow on a collapsed directory or Backspace at the tree root
- **THEN** the tree SHALL navigate up to the parent directory

#### Scenario: Toggle hidden files
- **WHEN** the user presses `h` while the file tree is focused
- **THEN** dotfiles and hidden directories SHALL toggle between visible and hidden (hidden by default)

#### Scenario: Lazy loading of directory contents
- **WHEN** a directory is expanded for the first time
- **THEN** the system SHALL read its contents from the filesystem at that moment (not preloaded)

#### Scenario: Large directory handling
- **WHEN** a directory contains more than 200 entries
- **THEN** only the first 200 entries SHALL be displayed, with a "(N more...)" indicator at the end

#### Scenario: Jump to path
- **WHEN** the user presses `/` in the file tree pane and types a path
- **THEN** the tree SHALL navigate to and expand that path, scrolling it into view

### Requirement: Transfer direction selection
The transfer form SHALL allow the user to toggle between Upload and Download modes. The current mode SHALL be visually highlighted.

#### Scenario: Toggle transfer direction
- **WHEN** the user presses `d` on the transfer form pane
- **THEN** the mode SHALL switch between Upload and Download, and the labels SHALL update accordingly (Upload: local path is source; Download: local path is destination)

### Requirement: Remote path input
The transfer form SHALL provide a text input field for the remote file path. This field accepts free-text input.

#### Scenario: Enter remote path
- **WHEN** the user focuses the remote path field and types a path like `/var/log/app.log`
- **THEN** the system SHALL store it as the remote path for the transfer command

### Requirement: Automatic recursive mode for directories
When a directory is selected in the file tree, recursive mode SHALL be automatically enabled. When a file is selected, recursive mode SHALL be automatically disabled. The user MAY manually override the recursive flag.

#### Scenario: Directory selected sets recursive on
- **WHEN** the user selects a directory in the file tree
- **THEN** recursive mode SHALL be enabled and the SCP command SHALL include the `-r` flag

#### Scenario: File selected sets recursive off
- **WHEN** the user selects a file in the file tree
- **THEN** recursive mode SHALL be disabled and the SCP command SHALL NOT include the `-r` flag

#### Scenario: Manual recursive override
- **WHEN** the user presses `r` on the transfer form
- **THEN** the recursive flag SHALL toggle regardless of the selected path type

### Requirement: Execute file transfer via SCP
The system SHALL build and execute an SCP command using the connection's parameters (host, port, user, identity file, proxy jump, SSH options). The TUI SHALL suspend during transfer to allow SCP to run interactively.

#### Scenario: Upload a file
- **WHEN** the user confirms an upload with local path `/tmp/config.yaml` and remote path `/etc/app/config.yaml`
- **THEN** the system SHALL execute `scp` with the local path as source and `[user@]host:remote_path` as destination, using the connection's port, identity file, and SSH options

#### Scenario: Download a file
- **WHEN** the user confirms a download with remote path `/var/log/app.log` and local path `/tmp/app.log`
- **THEN** the system SHALL execute `scp` with `[user@]host:remote_path` as source and the local path as destination

#### Scenario: Transfer with proxy jump
- **WHEN** the connection has a proxy_jump configured
- **THEN** the SCP command SHALL include `-J <proxy_jump>`

#### Scenario: Transfer succeeds
- **WHEN** the SCP command exits with code 0
- **THEN** the system SHALL display a success message in the status bar and return to the detail screen

#### Scenario: Transfer fails
- **WHEN** the SCP command exits with a non-zero code
- **THEN** the system SHALL display an error message in the status bar and remain on the transfer screen

### Requirement: Transfer history per connection
The system SHALL maintain a history of recent transfers per connection, storing the last 10 entries. Each entry SHALL record: direction (upload/download), local path, remote path, recursive flag, and timestamp.

#### Scenario: Transfer added to history
- **WHEN** a transfer completes successfully
- **THEN** the system SHALL add the transfer details to the connection's history

#### Scenario: History exceeds limit
- **WHEN** a connection has 10 history entries and a new transfer completes
- **THEN** the oldest entry SHALL be removed

### Requirement: Quick-fill from transfer history
The Transfer screen SHALL display the connection's recent transfer history. The user SHALL be able to select a history entry to populate the path fields and direction.

#### Scenario: Select history entry
- **WHEN** the user selects a previous transfer from the history list
- **THEN** the local path, remote path, direction, and recursive flag SHALL be populated from that entry

#### Scenario: No transfer history
- **WHEN** the connection has no previous transfers
- **THEN** the history section SHALL display "No recent transfers"

### Requirement: SCP binary validation
The system SHALL validate that the `scp` binary is available on the system. If not found, the Transfer action SHALL display an error message.

#### Scenario: SCP binary missing
- **WHEN** the user attempts to open the Transfer screen and `scp` is not found in PATH
- **THEN** the system SHALL display "scp not found — install OpenSSH to use file transfer" in the status bar and remain on the detail screen

### Requirement: Cancel transfer setup
The user SHALL be able to cancel the transfer and return to the detail screen by pressing Esc.

#### Scenario: Cancel transfer
- **WHEN** the user presses Esc on the Transfer screen
- **THEN** the system SHALL return to the connection detail screen without executing any transfer
