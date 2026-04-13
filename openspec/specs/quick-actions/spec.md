### Requirement: Quick action data model
The system SHALL define a `QuickAction` struct with: `name`, `category`, `description`, `command_template` (with `{key}` placeholders), `params` (list of parameters), and optional `confirm_message`. The system SHALL define `ActionParam` with `key`, `label`, and `ParamType` (Text, Select, Confirm). Adding a new action MUST require only adding one struct instance to the registry.

#### Scenario: Action without parameters
- **WHEN** an action has an empty `params` list
- **THEN** it executes immediately on selection (or after confirm if `confirm_message` is set)

#### Scenario: Action with parameters
- **WHEN** an action has one or more `params`
- **THEN** a parameter form is shown before execution

### Requirement: Parameter types
The system SHALL support three parameter types:

- **Text**: Free text input with an optional default value
- **Select**: A list of choices fetched from the server via SSH command, one option per line
- **Confirm**: A yes/no gate for dangerous operations

#### Scenario: Text parameter
- **WHEN** a form field is of type `Text`
- **THEN** a text input field is rendered with the default value pre-filled

#### Scenario: Select parameter with remote fetch
- **WHEN** a form field is of type `Select`
- **THEN** the system runs `ssh <connection> <fetch_command>` in a background thread, shows "Loading..." until results arrive, then displays a selectable list of options parsed from stdout lines

#### Scenario: Select field interaction
- **WHEN** user focuses a Select field and presses Enter or Space
- **THEN** the field expands to show all options; arrow keys navigate, Enter confirms selection, Esc collapses

#### Scenario: Confirm parameter
- **WHEN** a form contains a Confirm field
- **THEN** a "y/n" prompt is displayed and the form cannot submit unless the user confirms with `y`

### Requirement: Quick actions popup menu
The system SHALL display a popup overlay on the detail screen when the user presses `a`. Actions SHALL be grouped by category with non-selectable category headers. The user SHALL navigate with arrow keys or `j`/`k`, execute/open form with `Enter`, and dismiss with `Esc`.

#### Scenario: Open and navigate
- **WHEN** user presses `a` on the detail screen
- **THEN** a centered popup lists all quick actions grouped by category; j/k/arrows navigate, Enter selects, Esc dismisses

### Requirement: Parameter form screen
The system SHALL display a form screen when an action with parameters is selected. Each parameter renders as its corresponding control. Tab or arrow keys move between fields. Enter on the last field or Ctrl+Enter submits the form.

#### Scenario: Fill and submit form
- **WHEN** user fills in all parameters and submits
- **THEN** the system replaces `{key}` placeholders in the command template with the provided values and executes the command

#### Scenario: Cancel form
- **WHEN** user presses Esc on the parameter form
- **THEN** the form closes and returns to the action list

### Requirement: Shell-safe parameter substitution
The system SHALL shell-escape all parameter values before substituting them into the command template to prevent shell injection.

#### Scenario: Special characters in input
- **WHEN** a user enters a value containing shell metacharacters (e.g., `; rm -rf /`)
- **THEN** the value is escaped so it is treated as a literal string in the SSH command

### Requirement: Remote command execution
The system SHALL execute the final command by running `ssh [connection-params] <command>`. The TUI SHALL suspend before execution, print the command being run, show command output, wait for "Press Enter to continue", then resume.

#### Scenario: Execute and return
- **WHEN** a quick action command is executed
- **THEN** the TUI suspends, command runs with visible output, user presses Enter, TUI resumes to the detail screen

### Requirement: 20 built-in actions
The system SHALL ship 20 built-in quick actions:

**System Info (no params, immediate execution):**
1. Disk Usage — `df -h`
2. Memory Usage — `free -h`
3. System Uptime — `uptime`
4. Top Processes by CPU — `ps aux --sort=-%cpu | head -20`
5. Top Processes by Memory — `ps aux --sort=-%mem | head -20`
6. OS & Kernel Info — `uname -a && cat /etc/os-release`

**Service Management (with Select params):**
7. Restart Service — param: select service from `systemctl list-units --type=service --state=running --no-legend | awk '{print $1}'` → `sudo systemctl restart {service}`
8. Stop Service — param: select from running services → `sudo systemctl stop {service}`
9. Start Service — param: select from stopped/inactive services via `systemctl list-units --type=service --state=inactive --no-legend | awk '{print $1}'` → `sudo systemctl start {service}`
10. Service Status — param: select from all services via `systemctl list-units --type=service --no-legend | awk '{print $1}'` → `systemctl status {service}`
11. View Service Logs — param: select from all services, text param for line count (default "50") → `journalctl -u {service} -n {lines} --no-pager`

**Network:**
12. Listening Ports — `ss -tlnp`
13. Active Connections — `ss -tnp`
14. Firewall Rules — `sudo iptables -L -n --line-numbers 2>/dev/null || sudo ufw status verbose`
15. Public IP — `curl -s ifconfig.me && echo`

**Logs (some with params):**
16. System Logs — text param for lines (default "50") → `journalctl -n {lines} --no-pager`
17. Auth Logs — `journalctl -u sshd -n 30 --no-pager`
18. Search Logs — text param for grep pattern → `journalctl --no-pager -n 200 | grep -i {pattern}`

**Maintenance:**
19. Check Updates — `apt list --upgradable 2>/dev/null || yum check-update 2>/dev/null || dnf check-update 2>/dev/null`
20. Reboot Server — confirm_message: "This will REBOOT the server. Are you sure?" → `sudo reboot`

#### Scenario: All 20 actions available
- **WHEN** user opens the quick actions menu
- **THEN** all 20 actions are listed grouped by their category
