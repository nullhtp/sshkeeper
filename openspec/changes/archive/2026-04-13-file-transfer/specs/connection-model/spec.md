## MODIFIED Requirements

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

Note: Transfer history is stored in a separate file (`~/.config/sshkeeper/transfer_history.toml`), not in the connection model itself. The connection `id` is used as the key to associate history with connections.

#### Scenario: Create a minimal connection
- **WHEN** a connection is created with only `name` and `host`
- **THEN** the system SHALL assign a unique `id`, set `port` to 22, set `created_at` and `updated_at` to the current time, and leave optional fields empty

#### Scenario: Create a fully specified connection
- **WHEN** a connection is created with all fields populated
- **THEN** the system SHALL store all provided values and assign a unique `id`
