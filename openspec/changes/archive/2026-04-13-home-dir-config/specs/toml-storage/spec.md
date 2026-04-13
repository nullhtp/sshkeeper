## MODIFIED Requirements

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
