## Why

Users often connect to servers with password authentication, which is less secure and requires typing a password every time. SSHKeeper should help users upgrade to key-based auth with a single action — generate a key if needed and copy it to the server — so they enter their password one last time and never again.

## What Changes

- Add a "setup key auth" action on any connection, accessible from the Detail screen via a keybinding
- Check for an existing SSH key pair, offer to generate one (ed25519) if none exists
- Run `ssh-copy-id` to deploy the public key to the remote server (user enters password once)
- Update the connection profile with the identity file path after successful setup
- On Windows where `ssh-copy-id` is unavailable, fall back to manual key deployment via an SSH command

## Capabilities

### New Capabilities
- `key-auth-setup`: One-action setup of SSH key-based authentication for a connection — key generation, key deployment, and connection profile update

### Modified Capabilities
- `ssh-backend`: Adding a method to run arbitrary SSH commands (needed for Windows fallback key deployment)
- `connection-editor`: The Detail screen gains a new keybinding to trigger key auth setup

## Impact

- **Modified code**: `ssh/system.rs` (new command runner), `ui/detail.rs` (new keybinding), `ui/app.rs` (new action flow)
- **New code**: `ssh/key_setup.rs` (key generation + deployment logic)
- **External commands**: `ssh-keygen`, `ssh-copy-id` (Unix), `ssh` with remote append (Windows fallback)
- **No new dependencies**: Uses std::process::Command for all external calls
