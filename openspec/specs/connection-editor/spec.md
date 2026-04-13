## ADDED Requirements

### Requirement: Add new connection
The system SHALL provide a form to create a new connection profile with fields for name, host, port, user, identity file, group, tags, and SSH options.

#### Scenario: Add with required fields only
- **WHEN** the user presses `a` in Browse state and fills in name and host, then confirms
- **THEN** a new connection SHALL be created with defaults for optional fields and saved to storage

#### Scenario: Add with all fields
- **WHEN** the user fills in all form fields and confirms
- **THEN** a new connection SHALL be created with all provided values and saved to storage

#### Scenario: Cancel add
- **WHEN** the user presses `Escape` while in the add form
- **THEN** no connection SHALL be created and the app SHALL return to Browse state

### Requirement: Edit existing connection
The system SHALL allow editing any field of an existing connection profile.

#### Scenario: Edit a connection
- **WHEN** the user presses `e` on a selected connection in Detail state
- **THEN** the edit form SHALL open pre-filled with the connection's current values

#### Scenario: Save edits
- **WHEN** the user modifies fields and confirms
- **THEN** the connection SHALL be updated in storage with the new values and `updated_at` SHALL be refreshed

### Requirement: Delete connection
The system SHALL allow deleting a connection with confirmation.

#### Scenario: Delete with confirmation
- **WHEN** the user presses `d` on a selected connection and confirms the prompt
- **THEN** the connection SHALL be removed from storage

#### Scenario: Cancel delete
- **WHEN** the user presses `d` and then cancels the confirmation
- **THEN** the connection SHALL remain unchanged

### Requirement: Key auth setup action on Detail screen
The Detail screen SHALL provide a keybinding (`K`) to trigger the key auth setup flow for the currently viewed connection.

#### Scenario: Trigger from Detail screen
- **WHEN** the user presses `K` on the Detail screen
- **THEN** the system SHALL initiate the key auth setup flow for that connection

#### Scenario: Help text updated
- **WHEN** the Detail screen is displayed
- **THEN** the help bar SHALL include `K: setup key auth` alongside existing keybindings
