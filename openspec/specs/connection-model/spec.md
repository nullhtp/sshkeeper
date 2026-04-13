## ADDED Requirements

### Requirement: Connection profile fields
The system SHALL represent each connection profile with the following fields:
- `id` (string, unique, auto-generated UUID)
- `name` (string, required, user-facing display name)
- `host` (string, required)
- `port` (u16, optional, defaults to 22)
- `user` (string, optional)
- `identity_file` (string, optional, path to private key)
- `group` (string, optional)
- `tags` (list of strings, optional)
- `ssh_options` (key-value map, optional, arbitrary SSH options like `-o StrictHostKeyChecking=no`)
- `proxy_jump` (string, optional, jump host)
- `created_at` (datetime)
- `updated_at` (datetime)

#### Scenario: Create a minimal connection
- **WHEN** a connection is created with only `name` and `host`
- **THEN** the system SHALL assign a unique `id`, set `port` to 22, set `created_at` and `updated_at` to the current time, and leave optional fields empty

#### Scenario: Create a fully specified connection
- **WHEN** a connection is created with all fields populated
- **THEN** the system SHALL store all provided values and assign a unique `id`

### Requirement: Connection profile serialization
The system SHALL serialize and deserialize connection profiles to/from TOML format using serde.

#### Scenario: Round-trip serialization
- **WHEN** a connection profile is serialized to TOML and then deserialized
- **THEN** all fields SHALL match the original values exactly

### Requirement: Connection grouping
The system SHALL support organizing connections by a `group` field. Connections with the same `group` value SHALL be logically grouped together.

#### Scenario: Connections in the same group
- **WHEN** two connections have `group` set to "production"
- **THEN** both connections SHALL appear under the "production" group when grouped

#### Scenario: Ungrouped connection
- **WHEN** a connection has no `group` value
- **THEN** it SHALL appear in a default ungrouped category
