# Audit Fixes Design

**Date:** 2026-03-31

## Goal

Fix the issues found during code audit so the desktop app no longer exposes a frontend injection path, does not misreport interactive SSH sessions as connected, enforces password-credential invariants on the backend, and makes the `auto_connect` / `auto_reconnect` settings behave as advertised.

## Current Context

- The frontend tunnel list renders user-controlled strings with `innerHTML`.
- Password-auth SSH sessions only react to password prompts and may remain blocked on first-use host key prompts.
- Runtime status currently treats any live SSH child process as a connected tunnel.
- Password-auth save validation is enforced only in the frontend.
- Tunnel definitions persist `auto_connect` and `auto_reconnect`, but the backend does not execute either behavior.

## Chosen Approach

Keep the current architecture and close the gaps at the existing boundaries:

- replace unsafe frontend list templating with explicit DOM construction
- extend password-auth SSH args with `StrictHostKeyChecking=accept-new`
- track runtime connection signals from SSH output so interactive or blocked sessions are not reported as connected
- make backend save logic require a usable credential for password-auth tunnels
- implement startup auto-connect and runtime auto-reconnect in the Tauri backend

This keeps the app structure stable while aligning behavior with the UI and README promises.

## Frontend Safety

`web/app.js` will stop interpolating tunnel fields into HTML. The tunnel list will be rendered with `document.createElement(...)` and `textContent` for every user-visible field. This removes script injection opportunities even when a malicious or malformed tunnel definition is loaded from disk.

No larger UI refactor is needed. Existing view-model formatting helpers can remain unchanged.

## SSH Runtime Behavior

Password-auth launch arguments will opt into `StrictHostKeyChecking=accept-new`, allowing first-use connections to proceed without a blocking yes/no prompt.

The managed process layer will classify SSH output into:

- waiting-for-password / interactive prompt states
- explicit error states
- signals that indicate forwarding is established

The runtime status mapping will use those signals instead of assuming that a running process means a working tunnel. A session that is still waiting for interaction will stay out of the `connected` state. A failed session will surface an error. A session with positive connection evidence will report `connected`.

## Backend Validation

`save_tunnel` will enforce password-auth credentials at the command layer:

- creating a password-auth tunnel requires a password in the save payload
- editing a password-auth tunnel may omit the password only if a credential already exists in keychain storage

This prevents invalid password-auth configurations from being persisted when frontend validation is bypassed.

## Auto Connect And Reconnect

During app setup, the backend will scan persisted tunnels and attempt to connect those marked `auto_connect`.

When a tunnel process exits unexpectedly and the tunnel enables `auto_reconnect`, the backend will schedule a reconnect attempt. User-initiated disconnects must not trigger reconnect. The first implementation should stay simple: single in-process scheduling with a fixed delay, no complex backoff state machine.

## Testing Strategy

Add tests before implementation for each repaired behavior:

- frontend regression for safe text rendering of tunnel list fields
- SSH arg test for `StrictHostKeyChecking=accept-new`
- runtime tests that distinguish waiting prompts from real connected state
- backend tests for rejecting password-auth saves without credentials
- auto-connect / auto-reconnect backend tests for startup, abnormal exit, and user disconnect paths

Verification should include targeted frontend tests plus `cargo test --workspace`.

## Non-Goals

- No redesign of the tunnel editor UI
- No exponential reconnect backoff or reconnect counters
- No change to credential storage backend
- No browser-mode support for real tunnel operations
