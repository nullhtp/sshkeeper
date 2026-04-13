## ADDED Requirements

### Requirement: Display connection list
The system SHALL display all stored connections in a scrollable list showing at minimum the connection name, host, and group.

#### Scenario: List with connections
- **WHEN** the user opens the app with stored connections
- **THEN** the list SHALL display all connections sorted alphabetically by name

#### Scenario: Empty state
- **WHEN** no connections exist
- **THEN** the system SHALL display a message indicating no connections and how to add one or import from SSH config

### Requirement: Keyboard navigation
The system SHALL support navigating the connection list with `j`/`k` or arrow keys (up/down), and `g`/`G` for jump to top/bottom.

#### Scenario: Move down
- **WHEN** the user presses `j` or Down arrow
- **THEN** the selection SHALL move to the next connection in the list

#### Scenario: Move up
- **WHEN** the user presses `k` or Up arrow
- **THEN** the selection SHALL move to the previous connection in the list

#### Scenario: Wrap behavior
- **WHEN** the selection is at the last item and the user presses `j`
- **THEN** the selection SHALL remain on the last item (no wrap)

### Requirement: Search and filter
The system SHALL support fuzzy filtering of the connection list by typing `/` to enter search mode and typing a query.

#### Scenario: Enter search mode
- **WHEN** the user presses `/`
- **THEN** a search input bar SHALL appear and the list SHALL filter as the user types

#### Scenario: Filter matches
- **WHEN** the user types "prod" in search
- **THEN** only connections whose name, host, group, or tags contain "prod" SHALL be shown

#### Scenario: Exit search
- **WHEN** the user presses `Escape` while in search mode
- **THEN** the search bar SHALL close and the full list SHALL be restored

### Requirement: Group display
The system SHALL support toggling between a flat list and a grouped view (connections organized under group headers).

#### Scenario: Toggle grouped view
- **WHEN** the user presses `Tab`
- **THEN** the list SHALL toggle between flat and grouped display modes
