# SSHKeeper

A cross-platform TUI for managing SSH connections.

## Features

- **Connection management** -- add, edit, delete, and organize SSH connections with groups and tags
- **SSH config import** -- import existing connections from `~/.ssh/config`
- **Quick actions** -- 20 built-in remote server commands (disk usage, restart service, view logs, etc.) with interactive parameter forms
- **File transfer** -- upload/download files via SCP with local and remote file tree browsers
- **Key auth setup** -- generate SSH keys and deploy them to servers in one step
- **Search and filter** -- fuzzy search across connections, searchable action menus

## Install

### Homebrew (macOS/Linux)

```sh
brew tap nullhtp/tap
brew install sshkeeper
```

### From source

```sh
cargo install --git https://github.com/nullhtp/sshkeeper.git
```

Or clone and build:

```sh
git clone https://github.com/nullhtp/sshkeeper.git
cd sshkeeper
cargo build --release
# Binary at target/release/sshkeeper
```

## Usage

```sh
sshkeeper
```

### Keybindings

**Browse screen:**

| Key | Action |
|-----|--------|
| `a` | Add connection |
| `i` | Import from ~/.ssh/config |
| `/` | Search |
| `Tab` | Toggle group view |
| `Enter` | View connection |
| `q` | Quit |

**Detail screen:**

| Key | Action |
|-----|--------|
| `Enter` | Connect via SSH |
| `e` | Edit connection |
| `d` | Delete connection |
| `t` | File transfer |
| `K` | Setup key auth |
| `a` | Quick actions |
| `Esc` | Back |

**Quick actions:**

| Key | Action |
|-----|--------|
| Type | Filter actions |
| `Enter` | Select action |
| `Tab`/arrows | Navigate form fields |
| `Enter` on select | Expand dropdown (type to search) |
| `Enter` on Execute | Run command |
| `Esc` | Back |

## Configuration

Connections are stored in `~/.sshkeeper/connections.toml`.

## License

MIT
