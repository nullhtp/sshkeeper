## ADDED Requirements

### Requirement: Terminal setup and teardown
The system SHALL enter raw mode and enable alternate screen on startup, and SHALL restore the terminal to its original state on exit — including on panic.

#### Scenario: Normal startup
- **WHEN** the application starts
- **THEN** the terminal SHALL enter raw mode with alternate screen enabled

#### Scenario: Normal exit
- **WHEN** the user quits the application
- **THEN** the terminal SHALL be restored to its pre-launch state

#### Scenario: Panic recovery
- **WHEN** the application panics
- **THEN** a panic hook SHALL restore the terminal before printing the panic message

### Requirement: Event loop
The system SHALL run a single-threaded event loop that polls for keyboard events and renders the UI at a tick rate of no more than 250ms when idle.

#### Scenario: Keypress handling
- **WHEN** the user presses a key
- **THEN** the event loop SHALL deliver the key event to the current state handler within one tick cycle

#### Scenario: Idle rendering
- **WHEN** no input events occur
- **THEN** the UI SHALL still re-render at the tick rate to support future dynamic content

### Requirement: State machine routing
The system SHALL route input and rendering to the active screen based on an enum-based state machine with states: Browse, Detail, Search, Edit, Connecting.

#### Scenario: State transition from Browse to Detail
- **WHEN** the user selects a connection in Browse state and presses Enter
- **THEN** the app SHALL transition to Detail state showing that connection's information

#### Scenario: Global quit
- **WHEN** the user presses `q` or `Ctrl+C` from Browse state
- **THEN** the application SHALL exit cleanly
