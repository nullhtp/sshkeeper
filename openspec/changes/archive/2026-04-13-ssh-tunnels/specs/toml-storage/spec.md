## MODIFIED Requirements

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
