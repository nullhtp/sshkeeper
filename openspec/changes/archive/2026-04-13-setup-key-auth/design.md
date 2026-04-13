## Context

SSHKeeper currently stores connection profiles and spawns system SSH to connect. Authentication is whatever the user has configured externally. There's no way to help users transition from password auth to key-based auth from within the tool.

The SSH ecosystem already has the right tools: `ssh-keygen` for key generation and `ssh-copy-id` for key deployment. This feature wraps them into a guided flow triggered from the TUI.

## Goals / Non-Goals

**Goals:**
- One-action key auth setup from the Detail screen
- Generate ed25519 key pair if none exists
- Deploy public key to the remote server via `ssh-copy-id` (Unix) or SSH command fallback (Windows)
- Update the connection profile's `identity_file` after successful setup
- Suspend the TUI during the process (user needs to see prompts and enter password)

**Non-Goals:**
- Managing multiple key pairs per connection
- Revoking or rotating keys
- Custom key types or sizes (ed25519 is the sensible default)
- Passphrase management for the key itself (user decides during `ssh-keygen`)
- SSH agent configuration

## Decisions

### 1. Use ssh-keygen and ssh-copy-id directly

**Choice**: Shell out to `ssh-keygen` and `ssh-copy-id` rather than implementing key generation or deployment in Rust.

**Alternatives considered**:
- **Native Rust key generation** (via `ssh-key` crate): More code, more dependencies, no benefit — `ssh-keygen` is always available where SSH is
- **Manual authorized_keys append on all platforms**: More fragile, harder to get right (permissions, file creation)

**Rationale**: These tools exist, are battle-tested, and handle edge cases (file permissions, authorized_keys format, duplicate detection). We just orchestrate them.

### 2. Default key path: ~/.ssh/id_ed25519

**Choice**: Check for `~/.ssh/id_ed25519`. If it doesn't exist, run `ssh-keygen -t ed25519` and let the user interact with the prompts (passphrase choice, etc.).

**Rationale**: ed25519 is the modern default — smaller keys, faster, and recommended by OpenSSH. We don't override the user's choices during keygen (passphrase, comment) — the TUI is suspended and they interact with ssh-keygen directly.

### 3. Windows fallback via SSH command

**Choice**: On Windows, where `ssh-copy-id` doesn't exist, deploy the key by running:
```
type %USERPROFILE%\.ssh\id_ed25519.pub | ssh user@host "mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys && chmod 700 ~/.ssh"
```

**Rationale**: This replicates what `ssh-copy-id` does. It's a well-known pattern. The user still enters their password once for this SSH session.

### 4. TUI suspension during the entire flow

**Choice**: Suspend the TUI (restore normal terminal) for the full key setup process, similar to how we handle SSH connections.

**Rationale**: Both `ssh-keygen` and `ssh-copy-id` are interactive — they prompt for passphrases and passwords. The user needs to see and respond to these prompts directly. Trying to capture and relay their output would be fragile.

### 5. New module: ssh/key_setup.rs

**Choice**: Add a `key_setup` module in `src/ssh/` containing the orchestration logic.

**Rationale**: Keeps key setup logic separate from the SSH backend trait. The key setup is a one-time operation, not part of the connect flow.

## Risks / Trade-offs

- **[Risk] ssh-keygen not in PATH** → Check before running, show clear error. Unlikely since SSH is a prerequisite.
- **[Risk] ssh-copy-id fails (wrong password, network issue, server rejects)** → Show the error output to the user. The connection profile is only updated on success.
- **[Risk] User already has a key but it's not ed25519** → Check for any existing key (`id_ed25519`, `id_rsa`, `id_ecdsa`). Use whichever exists rather than forcing ed25519 generation.
- **[Risk] Remote server has password auth disabled** → `ssh-copy-id` will fail with a clear error. Not our problem to solve.
- **[Trade-off] Windows fallback is less robust** → `ssh-copy-id` handles more edge cases. Acceptable since Windows SSH usage is less common and the fallback covers the standard case.
