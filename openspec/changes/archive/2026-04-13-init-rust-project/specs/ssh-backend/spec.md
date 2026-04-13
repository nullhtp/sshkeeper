## ADDED Requirements

### Requirement: SSH backend trait
The system SHALL define a trait for SSH connection execution with a `connect` method that accepts a connection profile.

#### Scenario: Trait abstraction
- **WHEN** a new SSH backend implementation is added
- **THEN** it SHALL only need to implement the `connect` method to be usable by the application

### Requirement: System SSH backend
The system SHALL provide a backend implementation that spawns the user's system `ssh` binary to connect.

#### Scenario: Connect with minimal profile
- **WHEN** connecting to a profile with only host specified
- **THEN** the system SHALL execute `ssh <host>`

#### Scenario: Connect with full profile
- **WHEN** connecting to a profile with host, port, user, identity file, proxy jump, and SSH options
- **THEN** the system SHALL construct the full SSH command with `-p`, `-l`, `-i`, `-J`, and `-o` flags

#### Scenario: SSH binary not found
- **WHEN** the system `ssh` binary is not found in PATH
- **THEN** the system SHALL display an error message indicating SSH is not installed

### Requirement: Terminal handoff during SSH session
The system SHALL suspend the TUI (restore terminal to normal mode) before launching SSH and resume the TUI after the SSH session ends.

#### Scenario: Connect and return
- **WHEN** the user connects to a server and the SSH session ends
- **THEN** the TUI SHALL resume in the state it was in before connecting

### Requirement: Display SSH command
The system SHALL show the user the exact SSH command being executed before connecting.

#### Scenario: Command preview
- **WHEN** the user initiates a connection
- **THEN** the constructed SSH command SHALL be visible in the Detail or Connecting screen
