## Why

Server management often requires running the same commands repeatedly — restarting services, checking disk space, viewing logs. Currently users must open a full SSH session and type commands manually. Quick actions provide a curated menu of common remote commands with best-in-class UX: when an action has parameters (e.g., which service to restart), the app fetches live data from the server and presents smart form controls — select lists populated from the server, text inputs with defaults, confirmations for dangerous operations.

## What Changes

- Add a quick actions system with a two-phase UX: action selection → parameter form → execution
- Actions can define parameters, each with a type: `select` (populated by a remote command), `text` (free input with optional default), or `confirm` (y/n for dangerous actions)
- Ship 20 built-in actions across 5 categories, several with dynamic parameters
- Execute the final command remotely via SSH, suspending TUI to show output
- Data-driven design: adding a new action is defining one struct with its parameters — no code changes elsewhere

## Capabilities

### New Capabilities
- `quick-actions`: Extensible quick action system with dynamic parameter forms — action registry, parameter types with remote data fetching, form UI, 20 built-in actions

### Modified Capabilities

## Impact

- `src/ui/detail.rs` — new hotkey to open quick actions
- `src/ui/app.rs` — new `DetailAction` variants, remote execution handler
- New `src/ui/quick_actions.rs` — action list popup and parameter form UI
- New `src/ssh/actions.rs` — action definitions, parameter types, registry
