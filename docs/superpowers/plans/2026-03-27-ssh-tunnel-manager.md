# SSH Tunnel Manager Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Linux/Windows desktop tray application that manages SSH local port-forwarding tunnels with key or password authentication, config persistence, runtime supervision, and credential storage.

**Architecture:** Use a Tauri shell with a Rust backend for tunnel/process management and a web frontend for the single-window UI. Keep regular config in a JSON store, isolate secret handling behind a credential adapter, and model each tunnel as one supervised system `ssh` child process.

**Tech Stack:** Tauri, Rust, HTML/CSS/TypeScript frontend, JSON config store, platform credential APIs

---

## Execution Status

### Completed

- Workspace bootstrap completed with a Rust workspace, Tauri app shell, static web frontend, README, and `.gitignore`.
- Tunnel validation and `ssh -L` argument generation are implemented in `crates/core` with Rust tests.
- Config persistence is implemented in the Tauri backend with JSON storage and password separation via `keyring`.
- Runtime supervision exists for key auth and password auth, including PTY-driven password entry for system `ssh`.
- Native `ssh` processes now stream live stderr lines into the recent log view for easier debugging.
- Frontend shell exists with tunnel list, form, runtime status, recent log, and autostart toggle.
- Frontend view-model tests now cover snapshot meta, tunnel status copy, and connect/disconnect action states.
- Frontend list-rendering tests now cover item copy, localized badges, and active selection state.
- Main window now uses a command-center desktop shell with a compact left control column, dominant status hero, supporting cards, and drawer-based editing.
- Frontend command-center view-model tests now cover hero copy, support-card copy, and timeline summaries.
- Frontend app helper tests now cover command-center render-model behavior, including error and fallback health messaging.
- Workspace summary cards now surface connection state, forwarding details, auth mode, and reconnect/error summaries.
- Workspace log panel now uses a timeline-style diagnostic layout for status events and SSH output.
- Backend runtime tests now cover recent-log trimming, exit-state handling, and disconnect cleanup.
- Backend save/delete command helpers now have coverage for mutation order, auth-field normalization, runtime cleanup, and config persistence.
- Backend connect/disconnect/autostart command helpers now have coverage for missing credentials, runtime replacement, disconnect idempotence, recent-order updates, and autostart branch selection.
- Tray disconnect-all flow now has helper coverage for batch disconnect, idle safety, and recent-order updates.
- Tray menu now rebuilds dynamically with recent tunnel quick actions for direct connect/disconnect.
- Desktop editor flow now guards Tauri bridge access, shows inline drawer save errors, and supports private-key file picking via the dialog plugin.
- Windows release build now uses the GUI subsystem entrypoint, so starting tunnels no longer shows a `cmd` black window in the packaged app.
- Packaging and CI are set up: local `cargo tauri build` support, Ubuntu verification workflow, Linux `.deb` artifact upload, and Windows installer workflow artifact upload.
- Windows packaged build has now been real-machine verified for normal tunnel startup behavior.

### Remaining Backlog

- Add deeper integration coverage only where helper-level tests are still insufficient.

---

## File Structure

- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/src/app_state.rs`
- Create: `src-tauri/src/models.rs`
- Create: `src-tauri/src/config_store.rs`
- Create: `src-tauri/src/tunnel_manager.rs`
- Create: `src-tauri/src/ssh_process.rs`
- Create: `src-tauri/src/credential_store.rs`
- Create: `src-tauri/src/platform.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `web/index.html`
- Create: `web/app.js`
- Create: `web/styles.css`
- Create: `web/tests/tunnel-manager.test.js`
- Create: `README.md`
- Modify later if Git is initialized: `.gitignore`

## Chunk 1: Workspace Bootstrap

### Task 1: Document workspace constraints

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Write the failing test**

Create a workspace smoke test that expects the project metadata files to exist after bootstrap.

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test web/tests/tunnel-manager.test.js`
Expected: FAIL because the app files do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Create the baseline project structure, initial README, and placeholder app files.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test web/tests/tunnel-manager.test.js`
Expected: PASS for file existence assertions.

## Chunk 2: Tunnel Domain Model

### Task 2: Define tunnel config validation

**Files:**
- Create: `src-tauri/src/models.rs`
- Test: `web/tests/tunnel-manager.test.js`

- [ ] **Step 1: Write the failing test**

Add a test that encodes a valid `-L` tunnel profile and an invalid one with conflicting ports or missing host fields.

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test web/tests/tunnel-manager.test.js`
Expected: FAIL because validation logic is missing.

- [ ] **Step 3: Write minimal implementation**

Implement the tunnel profile schema and validation rules.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test web/tests/tunnel-manager.test.js`
Expected: PASS for validation cases.

## Chunk 3: Config Persistence

### Task 3: Add JSON config storage

**Files:**
- Create: `src-tauri/src/config_store.rs`
- Test: `web/tests/tunnel-manager.test.js`

- [ ] **Step 1: Write the failing test**

Add a persistence test that saves tunnel metadata without a password field and reloads it.

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test web/tests/tunnel-manager.test.js`
Expected: FAIL because config storage is missing.

- [ ] **Step 3: Write minimal implementation**

Implement config save/load logic in the backend and keep secret references separate from config payloads.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test web/tests/tunnel-manager.test.js`
Expected: PASS for persistence and no-secret assertions.

## Chunk 4: SSH Command Assembly

### Task 4: Build safe `ssh -L` launch arguments

**Files:**
- Create: `src-tauri/src/ssh_process.rs`
- Test: `web/tests/tunnel-manager.test.js`

- [ ] **Step 1: Write the failing test**

Add tests for generated `ssh` args for key auth and password auth profiles.

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test web/tests/tunnel-manager.test.js`
Expected: FAIL because the builder does not exist.

- [ ] **Step 3: Write minimal implementation**

Implement command generation with explicit `-L`, host, port, user, and key options, plus placeholders for password-interaction mode.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test web/tests/tunnel-manager.test.js`
Expected: PASS for command assembly assertions.

## Chunk 5: Runtime Supervision

### Task 5: Manage per-tunnel lifecycle

**Files:**
- Create: `src-tauri/src/tunnel_manager.rs`
- Create: `src-tauri/src/app_state.rs`
- Test: `web/tests/tunnel-manager.test.js`

- [ ] **Step 1: Write the failing test**

Add tests for state transitions: idle -> starting -> connected -> stopped -> error.

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test web/tests/tunnel-manager.test.js`
Expected: FAIL because no runtime manager exists.

- [ ] **Step 3: Write minimal implementation**

Implement tunnel state tracking, child-process bookkeeping, recent log capture, and stop/reconnect operations.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test web/tests/tunnel-manager.test.js`
Expected: PASS for lifecycle behavior.

## Chunk 6: Credential Abstraction

### Task 6: Isolate secret storage

**Files:**
- Create: `src-tauri/src/credential_store.rs`
- Create: `src-tauri/src/platform.rs`
- Test: `web/tests/tunnel-manager.test.js`

- [ ] **Step 1: Write the failing test**

Add tests that verify password profiles use a credential key reference instead of serializing the password.

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test web/tests/tunnel-manager.test.js`
Expected: FAIL because the credential adapter is missing.

- [ ] **Step 3: Write minimal implementation**

Define a credential-store trait and platform-specific stub adapters for Windows and Linux.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test web/tests/tunnel-manager.test.js`
Expected: PASS for secret-separation behavior.

## Chunk 7: Frontend Shell

### Task 7: Build single-window tray-oriented UI

**Files:**
- Create: `web/index.html`
- Create: `web/styles.css`
- Create: `web/app.js`
- Test: `web/tests/tunnel-manager.test.js`

- [ ] **Step 1: Write the failing test**

Add a DOM-oriented test for rendering the tunnel list and status panel from a sample state object.

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test web/tests/tunnel-manager.test.js`
Expected: FAIL because the frontend shell is missing.

- [ ] **Step 3: Write minimal implementation**

Implement a single-window UI with list, form, status panel, and recent log panel.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test web/tests/tunnel-manager.test.js`
Expected: PASS for rendering assertions.

## Chunk 8: Packaging And Docs

### Task 8: Prepare runnable project metadata

**Files:**
- Create: `src-tauri/tauri.conf.json`
- Modify: `README.md`

- [ ] **Step 1: Write the failing test**

Add a final smoke test that expects the Tauri config and README usage instructions to exist.

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test web/tests/tunnel-manager.test.js`
Expected: FAIL because packaging metadata is missing.

- [ ] **Step 3: Write minimal implementation**

Add Tauri config, usage notes, and explicit environment prerequisites.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test web/tests/tunnel-manager.test.js`
Expected: PASS for metadata checks.

## Environment Preconditions

- Rust toolchain is required to compile the Tauri backend.
- Node.js and npm are already present in the current environment.
- If the workspace remains outside a Git repository, commit steps should be skipped.

## Verification

- `node --test web/tests/tunnel-manager.test.js`
- `cargo test` once Rust is installed
- `cargo tauri build` or equivalent packaging command once Tauri dependencies are installed

Plan complete and saved to `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`. Ready to execute?
