# SSH Tunnel Manager - Project Documentation Index

> Auto-generated project documentation and knowledge base.
> Generated: 2026-04-01

## Architecture Overview

SSH Tunnel Manager is a cross-platform desktop application for managing SSH local port forwarding tunnels. Built with Tauri 2 (Rust backend + static web frontend), it provides a command-center style UI with system tray integration for managing persistent SSH tunnels on Linux and Windows.

**Stack**: Tauri 2 | Rust | Vanilla JavaScript (no framework) | CSS custom properties

**Platforms**: Linux (`.deb`), Windows (NSIS `.exe`)

## Documentation Sections

- [Project Structure](#project-structure)
- [Backend Architecture (Rust)](#backend-architecture-rust)
- [Frontend Architecture (JavaScript)](#frontend-architecture-javascript)
- [Tauri Commands API](#tauri-commands-api)
- [Data Models](#data-models)
- [Process Management](#process-management)
- [State Management](#state-management)
- [Connectivity Testing](#connectivity-testing)
- [System Tray](#system-tray)
- [Configuration & Security](#configuration--security)
- [Theming](#theming)
- [Testing](#testing)
- [CI/CD Pipeline](#cicd-pipeline)

---

## Project Structure

```
sshtunnel/
├── crates/core/              # Core library (sshtunnel-core)
│   ├── src/
│   │   ├── lib.rs            # Module re-exports
│   │   ├── models.rs         # TunnelDefinition, AuthKind, validation
│   │   ├── ssh_args.rs       # SSH argument builders (tunnel & probe)
│   │   └── ssh_launch.rs     # LaunchPlan, CommandSpec, build_launch_plan
│   └── tests/
│       ├── ssh_launch.rs     # Integration tests for launch plans
│       └── tunnel_definition.rs  # Integration tests for models & args
├── src-tauri/                # Tauri desktop shell (sshtunnel-app)
│   ├── src/
│   │   ├── main.rs           # Entry point (GUI subsystem on Windows)
│   │   ├── lib.rs            # App setup, commands, state, runtime management
│   │   ├── managed_process.rs # Process lifecycle (native + PTY)
│   │   ├── tray_model.rs     # Tray menu data models & rendering
│   │   └── tray_model_tests.rs # Tray model unit tests
│   ├── capabilities/default.json  # Tauri permissions
│   └── tauri.conf.json       # Tauri app configuration
├── web/                      # Static frontend (no build step)
│   ├── index.html            # Main window layout
│   ├── app.js                # UI controller, bridge, rendering logic
│   ├── view-model.js         # Pure view model functions (display copy)
│   ├── styles.css            # Design tokens, layout, components
│   └── tests/
│       ├── app.test.js       # Controller logic tests
│       └── view-model.test.js # View model tests
├── vendor/portable-pty/      # Patched PTY library (vendored dependency)
├── .github/workflows/ci.yml  # CI pipeline
└── Cargo.toml                # Workspace root
```

---

## Backend Architecture (Rust)

The Rust backend is split into two crates:

### `sshtunnel-core` (crates/core)

Platform-independent library providing data models, validation, and SSH command construction.

| Module | Responsibility |
|--------|---------------|
| `models.rs` | `TunnelDefinition` struct, `AuthKind` enum, `validate()` method |
| `ssh_args.rs` | `build_ssh_args()` for tunnel mode, `build_ssh_probe_args()` for test mode |
| `ssh_launch.rs` | `LaunchPlan` enum (Native vs PromptedPassword), `build_launch_plan()` |

**Key Design**: `LaunchPlan` is a two-variant enum that abstracts over SSH authentication:
- `Native(CommandSpec)` - key-based auth, runs SSH as a standard child process
- `PromptedPassword { command, password, prompt }` - password auth, drives SSH via PTY

### `sshtunnel-app` (src-tauri)

Tauri desktop shell with runtime management, system tray, and all Tauri commands.

| Module | Responsibility |
|--------|---------------|
| `main.rs` | Entry point, `windows_subsystem = "windows"` for GUI mode |
| `lib.rs` | App setup, Tauri commands, state management, runtime lifecycle |
| `managed_process.rs` | `ManagedProcess` wrapping native and PTY-based SSH sessions |
| `tray_model.rs` | Tray menu item model, ordering, and label generation |

---

## Frontend Architecture (JavaScript)

No build step, no framework. Uses vanilla JS with a desktop bridge pattern.

### File Responsibilities

| File | Role |
|------|------|
| `index.html` | Semantic HTML layout, theme initialization, element references |
| `app.js` | **Controller**: desktop bridge, DOM manipulation, event handlers, polling |
| `view-model.js` | **View Model**: pure functions producing display-ready copy objects |
| `styles.css` | CSS custom properties, dark/light themes, responsive layout |

### Architecture Pattern

```
Tauri Backend  ←→  Desktop Bridge (app.js)  →  View Model (view-model.js)
                        ↓                              ↓
                   DOM Rendering              Pure display functions
```

- **Desktop Bridge** (`createDesktopBridge`) - abstracts `__TAURI__.core.invoke` and `__TAURI__.dialog.open`, returns graceful errors outside desktop
- **View Model** (`SshTunnelViewModel`) - pure functions that transform tunnel data into display objects (Chinese localized copy)
- **Controller** (`bootstrap()`) - wires events, manages state, calls view model, renders DOM

### Key Frontend Patterns

- **Signature-based diffing**: Tunnel list only rebuilds when serialized signature changes (`buildTunnelListSignature`)
- **Auto-follow logs**: `shouldAutoFollowLogPanel()` detects scroll position; only auto-scrolls when user is near bottom
- **Polling refresh**: `setInterval` polls `load_state` every 4 seconds when page is visible and editor is closed
- **IP masking**: IPs are masked by default (e.g., `10.***.***.12`), with per-tunnel eye-toggle buttons
- **Text node rendering**: All tunnel data rendered via `textContent` to prevent XSS

---

## Tauri Commands API

All commands return `Result<AppSnapshot, String>` (except as noted).

| Command | Parameters | Returns | Purpose |
|---------|-----------|---------|---------|
| `load_state` | - | `AppSnapshot` | Full application state snapshot |
| `save_tunnel` | `SaveTunnelPayload { tunnel, password }` | `AppSnapshot` | Create or update tunnel config |
| `delete_tunnel` | `id: String` | `AppSnapshot` | Remove tunnel and its credentials |
| `connect_tunnel` | `id: String` | `AppSnapshot` | Start SSH session for tunnel |
| `disconnect_tunnel` | `id: String` | `AppSnapshot` | Stop SSH session for tunnel |
| `set_autostart` | `enabled: bool` | `AppSnapshot` | Toggle OS auto-start |
| `reveal_config_path` | - | `String` | Return config file path |
| `test_tunnel_connectivity` | `SaveTunnelPayload { tunnel, password }` | `ConnectivityTestResponse { snapshot, result }` | Test SSH login + target reachability |

---

## Data Models

### TunnelDefinition

Core configuration entity. Serialized to `config.json`.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Unique identifier (slug-timestamp pattern) |
| `name` | `String` | Display name |
| `ssh_host` | `String` | SSH server hostname |
| `ssh_port` | `u16` | SSH server port (default: 22) |
| `username` | `String` | SSH username |
| `local_bind_address` | `String` | Local bind address (default: `127.0.0.1`) |
| `local_bind_port` | `u16` | Local listening port |
| `remote_host` | `String` | Remote target hostname |
| `remote_port` | `u16` | Remote target port |
| `auth_kind` | `AuthKind` | `PrivateKey` or `Password` |
| `private_key_path` | `Option<String>` | Path to SSH private key |
| `auto_connect` | `bool` | Connect on app startup |
| `auto_reconnect` | `bool` | Reconnect after unexpected disconnect |
| `password_entry` | `Option<String>` | Keyring entry name (`profile:{id}`) |

### AuthKind

```rust
enum AuthKind { PrivateKey, Password }
```

- **PrivateKey**: SSH `-i` flag with `IdentitiesOnly=yes`
- **Password**: PTY-driven interactive session, `PreferredAuthentications=password`, `PubkeyAuthentication=no`

### AppSnapshot

Full state sent to frontend on every command response:

| Field | Type | Description |
|-------|------|-------------|
| `tunnels` | `Vec<TunnelView>` | All tunnels with runtime status |
| `ssh_available` | `bool` | System SSH binary found |
| `autostart_enabled` | `bool` | OS auto-start state |
| `config_path` | `String` | Config file path |
| `test_recent_log` | `Vec<String>` | Latest connectivity test logs |

### ConnectivityTestResult

Returned by `test_tunnel_connectivity` command:

| Field | Type | Description |
|-------|------|-------------|
| `ssh_ok` | `bool` | SSH login succeeded |
| `target_ok` | `bool` | Remote target port reachable |
| `summary` | `String` | Human-readable overall result |
| `ssh_summary` | `String` | SSH login phase summary |
| `target_summary` | `Option<String>` | Target reachability phase summary (None if SSH failed first) |

### TunnelView

Per-tunnel state sent to frontend:

| Field | Type | Description |
|-------|------|-------------|
| `definition` | `TunnelDefinition` | Tunnel configuration |
| `status` | `TunnelStatus` | `Idle`, `Connected`, or `Error` |
| `last_error` | `Option<String>` | Most recent error message |
| `recent_log` | `Vec<String>` | Last 12 log entries |

---

## Process Management

### ManagedProcess

Dual-mode process wrapper handling both native and PTY-based SSH sessions.

```
ManagedProcess
├── NativeProcess       (key auth)
│   ├── child: Child
│   └── reader_thread   (stderr reader)
└── PromptedProcess     (password auth)
    ├── child: Box<dyn Child>  (PTY)
    ├── reader_thread    (PTY output reader + prompt detector)
    ├── _pty_master      (kept alive for stdin)
    └── _pty_writer      (password writer)
```

**Native flow** (key auth):
1. Spawn `ssh` with stderr pipe
2. Background thread reads stderr lines into shared log buffer

**Prompted flow** (password auth):
1. Create PTY pair, spawn SSH on slave
2. Reader thread monitors PTY output for `"assword:"` prompt
3. Writer thread waits for prompt signal (5s timeout), then sends password
4. ANSI escape sequences stripped before prompt matching (Windows ConPTY compatibility)

**Log buffer**: Shared `Arc<Mutex<Vec<String>>>`, capped at 24 entries per process.

---

## State Management

### AppState (Rust)

```
AppState
├── config_path: PathBuf
├── inner: Mutex<InnerState>
│   ├── tunnels: Vec<TunnelDefinition>
│   ├── runtimes: HashMap<String, TunnelRuntime>
│   └── recent_tunnel_ids: Vec<String>
├── test_recent_log: Mutex<Vec<String>>
└── tray_menu_signature: Mutex<Option<String>>
```

### TunnelRuntime

Per-tunnel runtime state, created lazily:

| Field | Type | Description |
|-------|------|-------------|
| `process` | `Option<ManagedProcess>` | Active SSH process |
| `last_error` | `Option<String>` | Last error message |
| `recent_log` | `Vec<String>` | Last 12 log entries |
| `disconnect_requested` | `bool` | User-initiated disconnect (suppresses reconnect) |
| `reconnect_pending` | `bool` | Auto-reconnect is pending |

### Runtime Maintenance Loop

Background thread runs every 1 second:
1. Polls all running processes (`try_wait`)
2. Updates status based on exit codes and error patterns
3. Triggers auto-reconnect for eligible tunnels
4. Refreshes system tray menu if state changed

### Connection Signal Detection

For password-auth sessions, "connected" status requires detecting SSH login success in logs:

```
Success patterns:
  - "authenticated to "
  - "entering interactive session"
  - "pledge: network"
```

Before this signal is detected, a running prompted session reports `Idle` (not `Connected`).

### Auto-Reconnect Logic

Reconnect is triggered when:
- `auto_reconnect` is enabled on the tunnel
- SSH process exited with non-zero status
- Disconnect was NOT user-initiated
- No authentication error detected (prevents infinite retry loops)
- Prompted session didn't exit before connecting

**Error classification**:

| Category | Patterns | Reconnect? |
|----------|----------|------------|
| Auth errors | `permission denied`, `host key verification failed`, `too many authentication failures`, etc. | No |
| Network errors | `connection refused`, `connection timed out`, `could not resolve hostname`, `broken pipe`, etc. | Yes |

---

## Connectivity Testing

Two-phase probe executed from the editor:

1. **SSH Login probe**: Runs `ssh -T user@host "printf '__SSHTUNNEL_LOGIN_OK__' >&2"`
2. **Target Reachability probe**: Runs `ssh -T user@host "sh -lc 'nc -z -w 5 host port || bash -c \"exec 3<>/dev/tcp/host/port\"'"`

Each phase has a 15-second timeout. Results use marker strings in stderr for detection:
- `__SSHTUNNEL_LOGIN_OK__` - SSH login succeeded
- `__SSHTUNNEL_TARGET_OK__` - Remote target reachable
- `__SSHTUNNEL_TARGET_FAIL__` - Remote target unreachable
- `__SSHTUNNEL_TARGET_TOOL_MISSING__` - Neither `nc` nor `bash` available on remote

---

## System Tray

### Tray Menu Structure

```
┌─ 打开主界面 (Open main window)
├─ 最近隧道 (Recent tunnels)
│  ├─ 连接/断开：tunnel-name (user@host)
│  ├─ 连接/断开：tunnel-name (user@host)
│  └─ 连接/断开：tunnel-name (user@host)  [max 3]
├─ ─────────
├─ 全部断开 (Disconnect all)
└─ 退出 (Quit)
```

### Tray Signature Optimization

Tray menu is only rebuilt when a serialized signature of all items changes. This prevents unnecessary menu reconstruction on every 1-second maintenance tick.

### Recent Tunnel Ordering

Tunnels are ordered by most recently interacted with. Any connect/disconnect/save/delete action promotes the tunnel to the front of `recent_tunnel_ids`.

---

## Configuration & Security

### Config Storage

- Config file: `{system-config-dir}/sshtunnel-manager/config.json`
- Contains `StoredConfig { tunnels: Vec<TunnelDefinition> }`
- Passwords are NEVER stored in config

### Credential Storage

- Uses OS-native credential stores via `keyring` crate:
  - Linux: Secret Service (GNOME Keyring / KDE Wallet)
  - Windows: Credential Manager
- Entry format: service=`sshtunnel-manager`, username=`profile:{tunnel-id}`
- Password validation on save requires either new password or existing stored credential

### Security Measures

- Frontend renders all tunnel fields as `textContent` (no `innerHTML`) to prevent XSS
- `StrictHostKeyChecking=accept-new` only for password auth (accepts new hosts, still verifies known)
- Private key path selected via native file dialog
- SSH passwords never written to disk

---

## Theming

### Theme System

- CSS custom properties defined in `:root` (light) and `:root[data-theme="dark"]` (dark)
- Theme detected on load: `localStorage > prefers-color-scheme > light default`
- Theme applied immediately via inline `<script>` to prevent flash
- Toggle button in sidebar updates `data-theme` attribute and `localStorage`

### Design Tokens

Key CSS variables:

| Variable | Purpose |
|----------|---------|
| `--ink` | Primary text color |
| `--muted` | Secondary text color |
| `--surface` | Card/panel background |
| `--surface-layer` | Input background |
| `--border` | Border color |
| `--primary` | Primary action color |
| `--danger` | Destructive action color |
| `--connected` | Connected status color |
| `--error` | Error status color |
| `--select-bg-color` | Select dropdown background |

---

## Testing

### Rust Tests

| Crate | Test Module | Count | Focus |
|-------|------------|-------|-------|
| `sshtunnel-core` | `tests/ssh_launch.rs` | 3 | Launch plan construction |
| `sshtunnel-core` | `tests/tunnel_definition.rs` | 4 | Validation, SSH args, probe args |
| `sshtunnel-app` | `lib.rs::runtime_tests` | 7 | Runtime lifecycle, status transitions |
| `sshtunnel-app` | `lib.rs::save_delete_tests` | 5 | Config persistence, credential checks |
| `sshtunnel-app` | `lib.rs::command_flow_tests` | 11 | Connect/disconnect flows, tray logic |
| `sshtunnel-app` | `lib.rs::tray_disconnect_all_tests` | 3 | Bulk disconnect |
| `sshtunnel-app` | `lib.rs::tray_menu_copy_tests` | 2 | Tray label correctness |
| `sshtunnel-app` | `managed_process.rs::tests` | 7 | Process spawn, PTY, cleanup |
| `sshtunnel-app` | `tray_model_tests.rs` | - | Tray model unit tests |

**Running tests**:
```bash
node --check web/app.js       # Frontend syntax check
cargo test --workspace         # All Rust tests
cargo check -p sshtunnel-app   # Compile check only
```

### JavaScript Tests

Using Node.js built-in test runner (`node:test`):

| File | Count | Focus |
|------|-------|-------|
| `web/tests/view-model.test.js` | 18 | View model display functions |
| `web/tests/app.test.js` | 22 | Controller, rendering, bridge, CSS assertions |

**Notable**: CSS tests verify dark theme variables, select styling, log panel sizing, and shell scrollability exist in the stylesheet.

**Running tests**:
```bash
node --test web/tests/app.test.js web/tests/view-model.test.js
```

---

## CI/CD Pipeline

GitHub Actions (`.github/workflows/ci.yml`) runs three jobs:

| Job | Runner | Steps | Artifact |
|-----|--------|-------|----------|
| `ubuntu-check` | `ubuntu-latest` | Lint frontend, test core, check app | - |
| `ubuntu-deb` | `ubuntu-latest` | Build `.deb` package | `linux-deb/*.deb` |
| `windows-installer` | `windows-latest` | Build NSIS installer | `windows-installer/*.exe` |

**Linux dependencies**: `libdbus-1-dev`, `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `libxdo-dev`, `librsvg2-dev`, `libssl-dev`

**Rust toolchain**: stable (via `dtolnay/rust-toolchain`)
**Node version**: 22

---

## Vendored Dependencies

### portable-pty (vendor/portable-pty)

Patched version of the `portable-pty` crate. Provides PTY (pseudo-terminal) functionality for driving interactive SSH password sessions. The patch is applied via `[patch.crates-io]` in the workspace `Cargo.toml`.

---

## UI Layout

```
┌──────────────────────────────────────────────────────────────┐
│ Shell                                                         │
│ ┌──────────────┐ ┌──────────────────────────────────────────┐│
│ │ Control      │ │ Command Center                           ││
│ │ Column       │ │ ┌──────────────────────────────────────┐ ││
│ │              │ │ │ Hero Card                            │ ││
│ │ ┌──────────┐ │ │ │ Title + Status + Actions             │ ││
│ │ │ Brand    │ │ │ │ Forward | Auth | Health properties   │ ││
│ │ └──────────┘ │ │ └──────────────────────────────────────┘ ││
│ │ [New] [⚙] [🌙]│ │ ┌──────────────────────────────────────┐ ││
│ │              │ │ │ Timeline Card                        │ ││
│ │ ┌──────────┐ │ │ │ ┌─────────────┐ ┌─────────────────┐  │ ││
│ │ │ Tunnel   │ │ │ │ │ Status      │ │ SSH Output      │  │ ││
│ │ │ List     │ │ │ │ │ Events      │ │                 │  │ ││
│ │ │          │ │ │ │ │ (lane)      │ │ (lane)          │  │ ││
│ │ └──────────┘ │ │ │ └─────────────┘ └─────────────────┘  │ ││
│ └──────────────┘ │ └──────────────────────────────────────┘ ││
│                  └──────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────┘
```

**Overlays**: Editor drawer (right), Settings modal (centered)
