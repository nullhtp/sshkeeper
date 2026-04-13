## ADDED Requirements

### Requirement: Tunnel data model
The system SHALL represent each tunnel configuration with the following fields:
- `id` (string, unique, auto-generated UUID)
- `name` (string, required, user-facing label e.g. "PostgreSQL", "SOCKS proxy")
- `tunnel_type` (enum: `local`, `remote`, `dynamic`)
- `bind_address` (string, optional, defaults to "127.0.0.1")
- `bind_port` (u16, required)
- `remote_host` (string, required for local/remote types, absent for dynamic)
- `remote_port` (u16, required for local/remote types, absent for dynamic)

#### Scenario: Create a local forwarding tunnel
- **WHEN** a tunnel is created with `tunnel_type = "local"`, `bind_port = 5432`, `remote_host = "db.internal"`, `remote_port = 5432`
- **THEN** the system SHALL store all fields and assign a unique `id`, defaulting `bind_address` to "127.0.0.1"

#### Scenario: Create a dynamic SOCKS tunnel
- **WHEN** a tunnel is created with `tunnel_type = "dynamic"` and `bind_port = 1080`
- **THEN** the system SHALL store the tunnel without `remote_host` or `remote_port` fields

#### Scenario: Create a remote forwarding tunnel
- **WHEN** a tunnel is created with `tunnel_type = "remote"`, `bind_port = 8080`, `remote_host = "localhost"`, `remote_port = 3000`
- **THEN** the system SHALL store all fields with `tunnel_type` set to "remote"

### Requirement: Tunnel type validation
The system SHALL enforce that `remote_host` and `remote_port` are present for `local` and `remote` tunnel types and absent for `dynamic` type.

#### Scenario: Missing remote_host for local tunnel
- **WHEN** a user attempts to create a local tunnel without specifying `remote_host`
- **THEN** the system SHALL reject the configuration with a validation error

#### Scenario: Dynamic tunnel ignores remote fields
- **WHEN** a user creates a dynamic tunnel
- **THEN** the system SHALL NOT require or store `remote_host` or `remote_port`

### Requirement: Tunnel serialization
The system SHALL serialize and deserialize tunnel configurations to/from TOML format using serde. The `tunnel_type` field SHALL serialize as a lowercase string ("local", "remote", "dynamic").

#### Scenario: Round-trip serialization
- **WHEN** a tunnel is serialized to TOML and then deserialized
- **THEN** all fields SHALL match the original values exactly

### Requirement: Tunnel SSH command generation
The system SHALL generate the correct SSH forwarding flag from a tunnel configuration:
- Local: `-L [bind_address:]bind_port:remote_host:remote_port`
- Remote: `-R [bind_address:]bind_port:remote_host:remote_port`
- Dynamic: `-D [bind_address:]bind_port`

#### Scenario: Local tunnel command flag
- **WHEN** generating an SSH flag for a local tunnel with `bind_address = "127.0.0.1"`, `bind_port = 5432`, `remote_host = "db.internal"`, `remote_port = 5432`
- **THEN** the system SHALL produce `-L 127.0.0.1:5432:db.internal:5432`

#### Scenario: Dynamic tunnel command flag
- **WHEN** generating an SSH flag for a dynamic tunnel with `bind_address = "127.0.0.1"`, `bind_port = 1080`
- **THEN** the system SHALL produce `-D 127.0.0.1:1080`

#### Scenario: Remote tunnel command flag
- **WHEN** generating an SSH flag for a remote tunnel with `bind_port = 8080`, `remote_host = "localhost"`, `remote_port = 3000`
- **THEN** the system SHALL produce `-R 127.0.0.1:8080:localhost:3000`
