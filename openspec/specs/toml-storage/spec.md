## ADDED Requirements

### Requirement: Storage location
The system SHALL store all config files (`connections.toml`, `transfer_history.toml`) in `~/.sshkeeper/` on all platforms, using `dirs::home_dir()` to resolve the home directory.

#### Scenario: First run with no existing config
- **WHEN** the app runs for the first time with no existing config anywhere
- **THEN** the system SHALL create `~/.sshkeeper/connections.toml` with an empty connections table

#### Scenario: Migration from old location
- **WHEN** the app runs and `~/.sshkeeper/` does not contain a config file but the old platform-specific config directory does
- **THEN** the system SHALL copy the file from the old location to `~/.sshkeeper/` without deleting the original

#### Scenario: Both locations have files
- **WHEN** config files exist in both `~/.sshkeeper/` and the old platform-specific location
- **THEN** the system SHALL use the `~/.sshkeeper/` files and ignore the old location

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
The TOML storage file SHALL be formatted for human readability — one table per connection, fields on separate lines, with blank lines between connections. Tunnel configurations SHALL be serialized as TOML array-of-tables (`[[connection-id.tunnels]]`) nested within their parent connection table.

#### Scenario: Manual inspection
- **WHEN** a user opens `connections.toml` in a text editor
- **THEN** each connection SHALL be clearly readable as a separate TOML table with descriptive field names

#### Scenario: Connection with tunnels
- **WHEN** a connection has tunnel configurations
- **THEN** each tunnel SHALL appear as a `[[connection-id.tunnels]]` section with its fields on separate lines, following the connection's other fields

#### Scenario: Connection without tunnels
- **WHEN** a connection has no tunnel configurations
- **THEN** the `tunnels` field SHALL be omitted from the TOML output (not written as an empty array)
