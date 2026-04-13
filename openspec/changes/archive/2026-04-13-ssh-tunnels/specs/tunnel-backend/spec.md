## ADDED Requirements

### Requirement: Spawn tunnel process
The system SHALL spawn an SSH tunnel as a background child process using `ssh -N` with the connection's authentication parameters (port, user, identity file, proxy jump, SSH options) plus the tunnel's forwarding flag (`-L`, `-R`, or `-D`).

#### Scenario: Spawn a local forwarding tunnel
- **WHEN** starting a local tunnel for a connection to `bastion.example.com` with user `deploy` and key `~/.ssh/id_ed25519`
- **THEN** the system SHALL spawn `ssh -N -p 22 -l deploy -i ~/.ssh/id_ed25519 -L 127.0.0.1:5432:db.internal:5432 bastion.example.com` as a background process

#### Scenario: Spawn a dynamic tunnel with proxy jump
- **WHEN** starting a dynamic tunnel for a connection that uses a jump host
- **THEN** the system SHALL include `-J <jump_host>` in the spawned SSH command

### Requirement: Track tunnel processes
The system SHALL maintain a mapping of tunnel IDs to their `Child` process handles. This mapping SHALL be stored in the application state and accessible from the UI layer.

#### Scenario: Multiple tunnels active
- **WHEN** 3 tunnels are started across different connections
- **THEN** the system SHALL track all 3 process handles independently

### Requirement: Check tunnel liveness
The system SHALL check whether a tunnel process is still running using a non-blocking `try_wait()` call. This check SHALL occur on each UI render cycle for visible tunnels.

#### Scenario: Tunnel still running
- **WHEN** `try_wait()` returns `None` for a tunnel process
- **THEN** the system SHALL report the tunnel as running

#### Scenario: Tunnel exited unexpectedly
- **WHEN** `try_wait()` returns `Some(status)` for a tunnel process
- **THEN** the system SHALL report the tunnel as stopped and remove the process handle

### Requirement: Stop tunnel process
The system SHALL stop a running tunnel by calling `kill()` on the child process followed by `wait()` to reap the process.

#### Scenario: Graceful stop
- **WHEN** the user requests to stop a running tunnel
- **THEN** the system SHALL kill the process and remove it from the active tunnels map

### Requirement: Cleanup on exit
The system SHALL kill all active tunnel processes when the application exits, whether via normal quit or Ctrl+C.

#### Scenario: App quit with active tunnels
- **WHEN** the user quits the app while 3 tunnels are running
- **THEN** the system SHALL kill all 3 tunnel processes before exiting

### Requirement: Stderr capture for error reporting
The system SHALL capture stderr from tunnel processes to report connection errors (e.g., authentication failure, port in use) to the user.

#### Scenario: Tunnel fails to start
- **WHEN** a tunnel process exits immediately after spawn with a non-zero exit code
- **THEN** the system SHALL read stderr output and display the error message in the UI
