## ADDED Requirements

### Requirement: Key auth setup action on Detail screen
The Detail screen SHALL provide a keybinding (`K`) to trigger the key auth setup flow for the currently viewed connection.

#### Scenario: Trigger from Detail screen
- **WHEN** the user presses `K` on the Detail screen
- **THEN** the system SHALL initiate the key auth setup flow for that connection

#### Scenario: Help text updated
- **WHEN** the Detail screen is displayed
- **THEN** the help bar SHALL include `K: setup key auth` alongside existing keybindings
