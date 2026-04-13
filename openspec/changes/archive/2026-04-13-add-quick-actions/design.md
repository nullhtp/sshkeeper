## Context

SSHKeeper is a Rust TUI (ratatui) SSH connection manager. The detail screen shows connection info and offers keybind actions. The editor screen demonstrates the existing form pattern: `tui_input::Input` fields with Tab/arrow navigation. Quick actions will extend this with a richer form system that includes select controls populated by live server data.

## Goals / Non-Goals

**Goals:**
- Two-phase UX: browse actions → fill parameter form → execute
- Parameter types: `Select` (choices fetched from server), `Text` (free input), `Confirm` (y/n gate)
- Ship 20 useful actions, many with dynamic parameters (e.g., pick a service from the running list)
- Adding a new action = one struct definition, no other code changes

**Non-Goals:**
- Custom/user-defined actions (future)
- Multi-step wizards or chained actions
- Background/async execution — commands run synchronously with TUI suspended
- Output capture back into TUI — raw terminal output is shown

## Decisions

### 1. Action and parameter model

```rust
pub struct QuickAction {
    pub name: &'static str,
    pub category: ActionCategory,
    pub description: &'static str,
    pub command_template: &'static str,  // e.g. "systemctl restart {service}"
    pub params: &'static [ActionParam],
    pub confirm_message: Option<&'static str>,  // shown before dangerous actions
}

pub struct ActionParam {
    pub key: &'static str,           // placeholder name in template
    pub label: &'static str,         // display label
    pub param_type: ParamType,
}

pub enum ParamType {
    Text { default: &'static str },
    Select { fetch_command: &'static str },  // SSH command to get options, one per line
    Confirm,
}
```

`command_template` uses `{key}` placeholders replaced with user-supplied values. Actions without params execute immediately (after optional confirm). Actions with params show a form first.

**Why templates over closures**: Pure data — serializable, inspectable, and trivial to add new ones. The `{key}` substitution is simple string replacement.

### 2. Two-phase UX flow

```
Detail screen → [a] → Action List popup
                         ↓ Enter
              (no params?) → execute immediately
              (has params?) → Parameter Form
                                ↓ Enter
                              Execute (suspend TUI, ssh, resume)
```

**Action List**: Category-grouped popup overlay. j/k or arrows navigate, Enter selects, Esc back.

**Parameter Form**: Renders each param as a control:
- `Text` → tui_input field (reuses existing `tui_input` crate)
- `Select` → Shows "Loading..." while fetching via SSH, then a selectable list. The fetch runs `ssh <conn> <fetch_command>` and splits output by newline.
- `Confirm` → Simple y/n prompt

Tab/arrows move between fields. Enter on last field (or explicit Ctrl+Enter) submits.

### 3. Select fetching with loading state

When a form with `Select` params opens, each select triggers an SSH command to fetch options. This runs in a child process:

1. Form renders with "Loading..." for selects
2. Spawn `ssh <conn> <fetch_command>` via `std::process::Command` with output captured
3. Parse stdout lines into options
4. Re-render form with populated select

Since ratatui is single-threaded and we can't do true async, the fetch blocks briefly. For most commands (list services, list containers) this is <1 second. The TUI event loop continues polling so it stays responsive — the fetch runs in a separate thread and the form polls for completion.

### 4. Select control UX

A select field shows the current value. When focused and user presses Enter or Space, it expands to show all options as a dropdown-style list. Arrow keys pick an option, Enter confirms, Esc collapses back. This is a common TUI pattern for select controls.

### 5. Command execution

After form submission, build the final command by replacing `{key}` placeholders in the template. Then run exactly like existing `do_connect`:

1. Suspend TUI (`ratatui::restore()`)
2. Print the command being run
3. `ssh [connection-params] <final-command>` via `std::process::Command`
4. "Press Enter to continue..."
5. Resume TUI (`ratatui::init()`)

## Risks / Trade-offs

- **[Fetch latency]** → Select population requires SSH roundtrip. Mitigation: show loading indicator, use threaded fetch so TUI stays responsive, most list commands complete in <1s.
- **[Command compatibility]** → Actions assume Linux with systemd. Mitigation: this covers the vast majority of servers; can add OS-specific action sets later.
- **[Shell injection in params]** → User-entered text goes into SSH command. Mitigation: shell-escape parameter values before substitution.
- **[Thread complexity]** → Threaded fetch adds complexity. Mitigation: contained to one component (select field), uses simple channel pattern.
