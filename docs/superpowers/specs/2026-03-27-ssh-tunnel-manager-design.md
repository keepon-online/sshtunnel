# SSH Tunnel Manager Design

## Context

- Workspace: `/home/top/project/sshtunnel`
- Current state: empty directory, not a Git repository
- Referenced instruction file `RTK.md` was not present in the workspace or under `/home/top/project`
- Target platforms: Linux and Windows
- Product direction approved during brainstorming: lightweight desktop tray utility

## Goal

Build a cross-platform desktop application for managing SSH local port forwarding tunnels (`ssh -L`) with a tray-first workflow, configuration management, credential storage in the system keychain, and clear runtime status.

## Confirmed Scope

### In Scope

- Desktop GUI
- System tray entry point
- Linux and Windows support
- Local port forwarding only (`-L`)
- Key-based authentication
- Password-based authentication
- Password storage in platform credential stores
- Per-tunnel connect, disconnect, reconnect
- Basic status display and recent error output
- Optional auto-connect on app launch
- Optional auto-reconnect
- Optional app auto-start at OS login
- Prefer system `ssh` binary

### Out of Scope

- Remote forwarding (`-R`)
- Dynamic SOCKS forwarding (`-D`)
- Jump host / ProxyJump
- Multi-user sync
- Fleet monitoring
- Traffic charts
- Embedded SSH implementation in v1

## User Experience

### Primary Flow

1. User opens the app and creates a tunnel profile.
2. User selects authentication mode: private key or password.
3. Passwords are saved in the system credential store instead of the regular config file.
4. User starts the tunnel from the main window or tray menu.
5. The app validates config, locates `ssh`, and launches a dedicated child process.
6. Tray state changes to connected, disconnected, or error.
7. If the tunnel exits, the app records the reason and exposes reconnect actions.

### Interaction Model

- Tray is the high-frequency control surface.
- Main window is a single-window workspace for editing profiles and reading status.
- The product remains intentionally small and operationally focused.

## Architecture

### Recommended Stack

- `Tauri` for desktop shell
- `Rust` for process management, config logic, and platform integration
- Web frontend for the app window

### Modules

#### 1. Tauri Shell

Responsibilities:

- App lifecycle
- System tray integration
- Window management
- Auto-start integration
- Frontend/backend command bridge

#### 2. Tunnel Core

Responsibilities:

- Validate tunnel definitions
- Build `ssh` arguments for `-L`
- Launch and supervise one child process per tunnel
- Stop or restart tunnels
- Track runtime state
- Record exit status and recent error text
- Detect whether system `ssh` exists

#### 3. Config Store

Responsibilities:

- Persist tunnel metadata in an app-local JSON file
- Exclude secrets from the regular config file
- Store flags such as auto-connect and auto-reconnect

Stored fields:

- tunnel name
- SSH host
- SSH port
- username
- local bind address
- local bind port
- remote target host
- remote target port
- auth mode
- private key path
- auto-connect flag
- auto-reconnect flag
- UI metadata

#### 4. Credential Adapter

Responsibilities:

- Save password to system credential storage
- Read password on demand
- Delete password when auth mode changes or profile is removed
- Report whether the credential exists

Platform targets:

- Windows: Credential Manager
- Linux: Secret Service

## Security Constraints

- Passwords never enter the plain config file.
- Passwords are never rendered in status logs.
- Diagnostic exports must stay redacted.
- Missing credentials are surfaced as a first-class error state.
- Password-based login must be encapsulated behind a backend-only interaction layer.

## Runtime Strategy

### Preferred Execution Path

- Detect system `ssh`
- Use one child process per tunnel
- Maintain in-memory state plus recent events
- Terminate gracefully first, then force-kill if needed

### Fallback Boundary

The approved product direction was "prefer system `ssh`, consider embedded fallback later". For v1, the codebase should keep the abstraction boundary, but the embedded fallback should not be implemented unless required later.

## UI Structure

### Tray

- Overall app state
- Recent tunnel shortcuts
- Connect / disconnect / reconnect actions
- Open main window
- Disconnect all
- Exit

### Main Window

- Left panel: tunnel list
- Right upper panel: selected tunnel form
- Right lower panel: runtime status and recent log

## Risks And Open Technical Constraints

### Rust Toolchain

At design time, the current environment did not have `rustc` or `cargo` installed, which blocks compiling a Tauri app in-place unless the toolchain is installed.

### Password-Based SSH

Because system `ssh` does not accept a normal password flag, password auth needs a controlled PTY/expect-style interaction layer. Platform differences may require a degraded message if a secure implementation is not available everywhere in v1.

### Linux Credential Storage

Secret Service availability depends on the desktop environment and installed services. The adapter layer should isolate this variability.

## Accepted Design Summary

The approved v1 is a tray-first desktop utility for Linux and Windows that manages local SSH port-forwarding profiles, stores passwords in system credential stores, and supervises one system `ssh` process per tunnel through a Rust/Tauri backend.
