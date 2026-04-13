## Why

SSHKeeper manages SSH connections effectively, but users frequently need to transfer files to and from their servers — uploading configs, downloading logs, deploying scripts. Currently they must leave the TUI and manually run `scp` or `rsync` commands. Adding integrated file transfer with a polished UX eliminates context-switching and makes SSHKeeper a complete SSH workflow tool.

## What Changes

- Add a new **Transfer** screen accessible from the connection detail view, providing upload and download modes
- Implement SCP-based file transfer using the system `scp` binary (reusing existing SSH connection parameters: host, port, user, identity file, proxy jump, custom options)
- Show a local file tree browser (split-pane layout) for visually navigating and selecting source/destination paths with keyboard navigation
- Support transferring individual files and entire directories (recursive mode)
- Display transfer progress and status feedback in the TUI
- Add transfer history to quickly repeat recent transfers

## Capabilities

### New Capabilities
- `file-transfer`: SCP-based upload and download of files and directories through the TUI, with local file browsing, path input, recursive transfers, and transfer history

### Modified Capabilities
- `connection-model`: Add transfer history tracking (recent source/destination paths per connection)

## Impact

- **Code**: New `transfer` UI screen module, new SCP command builder in `ssh/` module, extended connection model with transfer history
- **Dependencies**: No new crate dependencies required — uses system `scp` binary similar to existing SSH execution
- **UX**: New keybinding from detail screen (e.g., `t` for Transfer), new screen with upload/download mode toggle and file browser
