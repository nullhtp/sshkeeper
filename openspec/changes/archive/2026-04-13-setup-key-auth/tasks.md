## 1. Key Detection & Generation

- [x] 1.1 Create `src/ssh/key_setup.rs` module with function to find existing SSH key (check id_ed25519, id_ecdsa, id_rsa in order of preference)
- [x] 1.2 Implement key generation — run `ssh-keygen -t ed25519` with TUI suspended, handle success/failure/cancellation
- [x] 1.3 Handle `ssh-keygen` not found in PATH with clear error message

## 2. Key Deployment

- [x] 2.1 Implement Unix key deployment via `ssh-copy-id -i <key>.pub [-p port] [user@]host` with TUI suspended
- [x] 2.2 Implement Windows fallback — read public key file and deploy via `ssh user@host "mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys && chmod 700 ~/.ssh"`
- [x] 2.3 Handle deployment failure (wrong password, network error) — return error without modifying connection profile

## 3. SSH Backend Extension

- [x] 3.1 Add `run_command` method to `SystemSshBackend` — spawn SSH with connection params plus a remote command, return exit status

## 4. TUI Integration

- [x] 4.1 Add `SetupKeyAuth(String)` variant to `DetailAction` and handle `K` keybinding in Detail screen
- [x] 4.2 Update Detail screen help bar to include `K: setup key auth`
- [x] 4.3 Implement `do_setup_key_auth` in `App` — orchestrate the full flow: suspend TUI, detect/generate key, deploy, update profile, resume TUI
- [x] 4.4 Update connection's `identity_file` and save to storage on successful deployment
