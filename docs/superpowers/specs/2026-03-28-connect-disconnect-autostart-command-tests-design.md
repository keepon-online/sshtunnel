# Connect/Disconnect/Autostart Command Tests Design

**Date:** 2026-03-28

## Goal

Add focused backend tests for the remaining command flows around `connect_tunnel`, `disconnect_tunnel`, and `set_autostart` without introducing full Tauri integration tests.

## Current Context

- `save_tunnel` and `delete_tunnel` already have helper-level coverage for normalization, mutation, and persistence.
- `connect_tunnel` still mixes tunnel lookup, validation, credential handling, runtime replacement, process spawn, and recent-tunnel updates in one command body.
- `disconnect_tunnel` and `set_autostart` are smaller, but they are still only exercised indirectly.

## Chosen Approach

Extract a few small helpers inside `src-tauri/src/lib.rs` so tests can cover command behavior at the business-rule boundary:
- a connect-preparation helper
- a connect-success state-application helper
- a disconnect state-application helper
- a tiny autostart decision seam

Keep Tauri command signatures, tray refresh, and plugin interaction unchanged.

## Connect Coverage

Tests should verify that connecting:
- returns an error for an unknown tunnel ID
- for password auth with missing credentials, does not produce a launch request and instead stores an error plus `missing password credential`
- when an old runtime exists, disconnects it before applying the new connected state
- after a successful connect application, clears stale errors, stores the new process, appends the spawned log entry, and moves the tunnel to the front of `recent_tunnel_ids`

## Disconnect Coverage

Tests should verify that disconnecting:
- kills a running process, appends `stopped ssh process`, clears stale errors, and moves the tunnel to the front of `recent_tunnel_ids`
- is idempotent when the tunnel has no runtime entry

## Autostart Coverage

Tests should verify that:
- `enabled = true` dispatches to the enable branch
- `enabled = false` dispatches to the disable branch
- underlying errors are converted to `String` and returned unchanged

## Testing Strategy

- Add Rust unit tests in `src-tauri/src/lib.rs`
- Reuse the existing `ManagedProcess` test style for connect/disconnect runtime assertions
- Avoid tray and plugin assertions in this iteration
- Keep all tests at the helper boundary rather than spinning up a Tauri app runtime

## Non-Goals

- No full Tauri command integration tests
- No real `ssh -V` availability tests
- No real keyring integration tests
- No real autostart plugin integration tests
