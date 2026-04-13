## 1. Action Data Model

- [x] 1.1 Create `src/ssh/actions.rs` with `QuickAction`, `ActionParam`, `ParamType` (Text/Select/Confirm), `ActionCategory` enum, and shell-escape utility for parameter values
- [x] 1.2 Implement `build_actions()` returning all 20 built-in actions with their parameter definitions
- [x] 1.3 Export the module from `src/ssh/mod.rs`

## 2. Action List Popup

- [x] 2.1 Create `src/ui/quick_actions.rs` with `ActionListState` — holds actions grouped by category, selected index, navigation logic skipping category headers
- [x] 2.2 Implement `render()` for action list — centered popup overlay with category headers and action items, highlight on selected
- [x] 2.3 Implement `handle_key()` — j/k/arrows navigate, Enter selects action, Esc dismisses

## 3. Parameter Form

- [x] 3.1 Add `ParamFormState` to `quick_actions.rs` — holds current action, parameter values, focused field index, and per-select loading/options state
- [x] 3.2 Implement Text field control — tui_input field with default value, renders label + input
- [x] 3.3 Implement Select field control — shows "Loading..." during fetch, current value when collapsed, expandable option list on Enter/Space with arrow navigation
- [x] 3.4 Implement Confirm field control — y/n toggle display
- [x] 3.5 Implement threaded SSH fetch for Select params — spawn `ssh <conn> <fetch_command>` in background thread, communicate results via `mpsc` channel, poll in render loop
- [x] 3.6 Implement form submission — replace `{key}` placeholders in command template with shell-escaped values, return final command string
- [x] 3.7 Implement form `render()` — full-screen form with action name as title, param fields, help line
- [x] 3.8 Implement form `handle_key()` — Tab/arrows between fields, delegate to focused field, Esc cancels, Ctrl+Enter or Enter on last field submits

## 4. Integration

- [x] 4.1 Add `QuickActions` and `RunRemoteAction(String)` variants to `DetailAction` enum
- [x] 4.2 Add quick action overlay state to `DetailState` (None / ActionList / ParamForm), wire `a` key to open, delegate keys when overlay active
- [x] 4.3 Render action list / param form as overlay or screen on top of detail view
- [x] 4.4 Implement `do_run_quick_action()` in `App` — build `ssh [connection-params] <command>`, suspend TUI, print and run command, "Press Enter to continue", resume TUI
- [x] 4.5 Update detail screen help line to include `a: actions`
- [x] 4.6 Export quick_actions module from `src/ui/mod.rs`

## 5. Verification

- [x] 5.1 Build compiles with `cargo build`
- [ ] 5.2 Manual test: open detail → press `a` → navigate → select "Restart Service" → verify service list loads → pick one → confirm command runs
