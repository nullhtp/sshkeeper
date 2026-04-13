## ADDED Requirements

### Requirement: Import from SSH config
The system SHALL import connection profiles from `~/.ssh/config` by parsing Host entries and mapping their directives to connection profile fields.

#### Scenario: Import standard hosts
- **WHEN** the user triggers import and `~/.ssh/config` contains Host entries with HostName, Port, User, and IdentityFile
- **THEN** each Host entry SHALL be imported as a connection profile with fields mapped accordingly

#### Scenario: Skip wildcard hosts
- **WHEN** `~/.ssh/config` contains a `Host *` entry
- **THEN** that entry SHALL be skipped (not imported as a connection)

#### Scenario: SSH config not found
- **WHEN** `~/.ssh/config` does not exist
- **THEN** the system SHALL display a message indicating no SSH config was found

### Requirement: Import is non-destructive
The system SHALL NOT modify or write to `~/.ssh/config`. Import is read-only.

#### Scenario: Original file unchanged
- **WHEN** import completes
- **THEN** `~/.ssh/config` SHALL be byte-identical to before the import

### Requirement: Duplicate handling on import
The system SHALL detect connections that match an already-stored profile (by host + user + port) and skip them during import.

#### Scenario: Duplicate detected
- **WHEN** importing a host that matches an existing connection's host, user, and port
- **THEN** that host SHALL be skipped and the user SHALL be informed of the skip

#### Scenario: No duplicates
- **WHEN** all imported hosts are new
- **THEN** all hosts SHALL be imported as new connection profiles
