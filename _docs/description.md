# SSHKeeper

## Overview

SSHKeeper is a cross-platform, terminal-based application for managing and organizing SSH connections. It provides a TUI (Terminal User Interface) that makes it easy to store, browse, and quickly connect to remote servers without remembering hostnames, ports, or credentials. It runs on macOS, Linux, and Windows.

## Goals

- **Manage SSH connections** — Store and organize SSH connection profiles (host, port, user, key, etc.) in one place.
- **TUI-first experience** — Provide an intuitive terminal interface for browsing, searching, and selecting connections.
- **Simplify SSH workflows** — Connect to any saved server with minimal keystrokes. No need to remember or type full SSH commands.
- **Cross-platform** — Works on macOS, Linux, and Windows with a consistent experience across all platforms.
- **Easy to install** — Single binary with no dependencies. Install via package managers (Homebrew, apt, scoop) or download a prebuilt binary.

## Core Concepts

- **Connection profile** — A saved SSH connection with all parameters needed to connect (host, port, user, identity file, options).
- **Quick connect** — Select a profile from the list and connect instantly.
- **Groups/Tags** — Organize connections by project, environment, or any custom category.

## Key Features

1. **Connection storage** — Add, edit, and delete SSH connection profiles.
2. **Interactive TUI** — Navigate connections with keyboard shortcuts, search/filter, and connect in one step.
3. **One-action connect** — Select a connection and press Enter to open an SSH session.
4. **Import/Export** — Read from `~/.ssh/config` or export profiles for backup and sharing.
5. **Grouping** — Organize connections into logical groups for easy navigation.

## Design Principles

- **Simplicity** — Minimal configuration, sensible defaults, no unnecessary complexity.
- **Speed** — Fast startup, instant search, no lag in the TUI.
- **Transparency** — The user always sees the actual SSH command being run. No hidden magic.
- **Cross-platform** — Single binary that works the same on macOS, Linux, and Windows.
- **Easy installation** — Zero dependencies, single binary distribution. Installable via Homebrew, apt, scoop, or direct download.
