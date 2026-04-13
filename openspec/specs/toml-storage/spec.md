## ADDED Requirements

### Requirement: Storage location
The system SHALL store connection profiles in a `connections.toml` file inside the platform-appropriate config directory (`~/.config/sshkeeper/` on Linux, `~/Library/Application Support/sshkeeper/` on macOS, `%APPDATA%\sshkeeper\` on Windows).

#### Scenario: First run on Linux
- **WHEN** the app runs for the first time on Linux with no existing config
- **THEN** the system SHALL create `~/.config/sshkeeper/connections.toml` with an empty connections table

#### Scenario: First run on macOS
- **WHEN** the app runs for the first time on macOS with no existing config
- **THEN** the system SHALL create `~/Library/Application Support/sshkeeper/connections.toml`

### Requirement: Load connections on startup
The system SHALL load all connection profiles from the TOML storage file on startup.

#### Scenario: Valid storage file
- **WHEN** the storage file exists and contains valid TOML
- **THEN** all connections SHALL be loaded into memory

#### Scenario: Corrupted storage file
- **WHEN** the storage file contains invalid TOML
- **THEN** the system SHALL display an error with the parse failure location and NOT overwrite the file

#### Scenario: Missing storage file
- **WHEN** the storage file does not exist
- **THEN** the system SHALL start with an empty connection list and create the file on first save

### Requirement: Save connections
The system SHALL write the full connection list to the TOML storage file after any mutation (add, edit, delete).

#### Scenario: Save after add
- **WHEN** a new connection is added
- **THEN** the storage file SHALL be updated to include the new connection

#### Scenario: Atomic write
- **WHEN** saving connections
- **THEN** the system SHALL write to a temporary file and rename it to the target path to prevent corruption on crash

### Requirement: Human-readable format
The TOML storage file SHALL be formatted for human readability — one table per connection, fields on separate lines, with blank lines between connections.

#### Scenario: Manual inspection
- **WHEN** a user opens `connections.toml` in a text editor
- **THEN** each connection SHALL be clearly readable as a separate TOML table with descriptive field names
