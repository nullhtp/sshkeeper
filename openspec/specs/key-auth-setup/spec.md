## ADDED Requirements

### Requirement: Detect existing SSH key
The system SHALL check for existing SSH key pairs at standard paths (`~/.ssh/id_ed25519`, `~/.ssh/id_rsa`, `~/.ssh/id_ecdsa`) before attempting to generate a new one.

#### Scenario: Key exists
- **WHEN** the user triggers key auth setup and `~/.ssh/id_ed25519` exists
- **THEN** the system SHALL use the existing key and skip generation

#### Scenario: Multiple keys exist
- **WHEN** multiple key types exist (e.g., both id_rsa and id_ed25519)
- **THEN** the system SHALL prefer ed25519, then ecdsa, then rsa

#### Scenario: No key exists
- **WHEN** no SSH key pair is found at any standard path
- **THEN** the system SHALL proceed to key generation

### Requirement: Generate SSH key pair
The system SHALL generate a new ed25519 SSH key pair by running `ssh-keygen -t ed25519` when no existing key is found. The TUI SHALL be suspended so the user can interact with ssh-keygen prompts directly.

#### Scenario: Successful generation
- **WHEN** ssh-keygen completes successfully
- **THEN** the system SHALL proceed to key deployment using the newly generated key

#### Scenario: User cancels generation
- **WHEN** the user cancels or ssh-keygen exits with a non-zero status
- **THEN** the system SHALL abort the setup and return to the Detail screen with an error message

#### Scenario: ssh-keygen not found
- **WHEN** `ssh-keygen` is not found in PATH
- **THEN** the system SHALL display an error message and abort

### Requirement: Deploy public key to server
The system SHALL deploy the public key to the remote server. On Unix, it SHALL use `ssh-copy-id`. On Windows, it SHALL use an SSH command to append the key to `~/.ssh/authorized_keys` on the remote.

#### Scenario: Successful deployment on Unix
- **WHEN** `ssh-copy-id -i <key_path> user@host` completes successfully
- **THEN** the system SHALL report success and proceed to update the connection profile

#### Scenario: Successful deployment on Windows
- **WHEN** the SSH-based key append command completes successfully on Windows
- **THEN** the system SHALL report success and proceed to update the connection profile

#### Scenario: Deployment fails (wrong password)
- **WHEN** the user enters an incorrect password during key deployment
- **THEN** the system SHALL show the error from ssh-copy-id and abort without modifying the connection profile

#### Scenario: Connection with non-standard port
- **WHEN** the connection uses a non-standard port
- **THEN** the system SHALL pass `-p <port>` to ssh-copy-id

### Requirement: Update connection profile after setup
The system SHALL update the connection's `identity_file` field with the path to the private key used and save the profile to storage after successful key deployment.

#### Scenario: Profile updated
- **WHEN** key deployment succeeds
- **THEN** the connection's `identity_file` SHALL be set to the private key path and `updated_at` SHALL be refreshed

#### Scenario: Profile unchanged on failure
- **WHEN** key deployment fails
- **THEN** the connection profile SHALL remain unmodified
