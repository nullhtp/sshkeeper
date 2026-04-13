## ADDED Requirements

### Requirement: Tunnel list screen
The system SHALL display a screen listing all tunnels configured for a connection, showing for each: name, type (L/R/D), bind address:port, remote target (if applicable), and status (running/stopped).

#### Scenario: View tunnels for a connection with tunnels
- **WHEN** the user opens the tunnel manager for a connection that has 3 configured tunnels
- **THEN** the system SHALL display all 3 tunnels in a scrollable list with their details and current status

#### Scenario: View tunnels for a connection with no tunnels
- **WHEN** the user opens the tunnel manager for a connection with no configured tunnels
- **THEN** the system SHALL display an empty state message prompting the user to add a tunnel

### Requirement: Tunnel status indicators
The system SHALL display a visual status indicator for each tunnel:
- Green circle or "ON" label when the tunnel process is running
- Red circle or "OFF" label when the tunnel is stopped or has exited

#### Scenario: Running tunnel indicator
- **WHEN** a tunnel's background SSH process is alive
- **THEN** the system SHALL show a green/active status indicator next to that tunnel

#### Scenario: Stopped tunnel indicator
- **WHEN** a tunnel's background SSH process is not running
- **THEN** the system SHALL show a red/inactive status indicator next to that tunnel

### Requirement: Start tunnel
The user SHALL be able to start a stopped tunnel by selecting it and pressing Enter or a designated start key.

#### Scenario: Start a stopped tunnel
- **WHEN** the user selects a stopped tunnel and presses Enter
- **THEN** the system SHALL spawn a background SSH process for that tunnel and update the status to running

#### Scenario: Start fails due to port conflict
- **WHEN** the user starts a tunnel but the bind port is already in use
- **THEN** the system SHALL detect the process exit and display an error message

### Requirement: Stop tunnel
The user SHALL be able to stop a running tunnel by selecting it and pressing Enter or a designated stop key.

#### Scenario: Stop a running tunnel
- **WHEN** the user selects a running tunnel and presses Enter
- **THEN** the system SHALL kill the background SSH process and update the status to stopped

### Requirement: Add tunnel
The user SHALL be able to add a new tunnel configuration via an inline form with fields for name, type, bind address, bind port, remote host, and remote port.

#### Scenario: Add a new local tunnel
- **WHEN** the user presses `a` on the tunnel list screen, fills in the form, and confirms
- **THEN** the system SHALL create a new tunnel configuration and save it to storage

#### Scenario: Cancel adding a tunnel
- **WHEN** the user presses Escape during tunnel creation
- **THEN** the system SHALL discard the form and return to the tunnel list

### Requirement: Edit tunnel
The user SHALL be able to edit an existing tunnel configuration by selecting it and pressing `e`.

#### Scenario: Edit tunnel name and port
- **WHEN** the user selects a tunnel, presses `e`, modifies fields, and confirms
- **THEN** the system SHALL update the tunnel configuration and save to storage

### Requirement: Delete tunnel
The user SHALL be able to delete a tunnel configuration by selecting it and pressing `d`.

#### Scenario: Delete a tunnel
- **WHEN** the user selects a tunnel and presses `d`
- **THEN** the system SHALL remove the tunnel from the connection, stop it if running, and save to storage

### Requirement: Access from detail screen
The tunnel manager SHALL be accessible from the connection detail screen via a keybinding (`u` for tunnels).

#### Scenario: Open tunnel manager
- **WHEN** the user presses `u` on the connection detail screen
- **THEN** the system SHALL navigate to the tunnel manager screen for that connection

### Requirement: Back navigation
The user SHALL be able to return to the connection detail screen by pressing Escape from the tunnel list.

#### Scenario: Return to detail
- **WHEN** the user presses Escape on the tunnel list screen
- **THEN** the system SHALL navigate back to the connection detail screen

### Requirement: Active tunnel count in status
The system SHALL display the count of currently active tunnels in the app status area when any tunnels are running.

#### Scenario: Active tunnels shown
- **WHEN** 2 tunnels are running across all connections
- **THEN** the status area SHALL display "2 tunnels active" or similar indicator
