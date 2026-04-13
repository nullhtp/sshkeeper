## ADDED Requirements

### Requirement: Run arbitrary SSH command
The system SSH backend SHALL support running an arbitrary command on a remote host via `ssh user@host <command>`, returning the exit status and output. This is needed for the Windows key deployment fallback.

#### Scenario: Run remote command
- **WHEN** a command is executed on a remote host
- **THEN** the system SHALL spawn `ssh` with the connection's parameters and the command appended, and return the exit status
