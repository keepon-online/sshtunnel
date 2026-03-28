# Save/Delete Command Tests Design

**Date:** 2026-03-28

## Goal

Add focused backend tests for `save_tunnel` and `delete_tunnel` behavior so config mutation, recent-tunnel ordering, runtime cleanup, and password-field normalization are covered without needing full Tauri command integration.

## Current Context

- `save_tunnel` and `delete_tunnel` currently mix input normalization, state mutation, config persistence, and tray refresh inside Tauri command handlers.
- Existing backend tests cover runtime lifecycle behavior, but not command-style state updates.
- Full Tauri command integration is heavier than needed for the core mutation rules we want to protect first.

## Chosen Approach

Extract small pure-ish helpers inside `src-tauri/src/lib.rs` for:
- normalizing a tunnel before save
- applying save mutations to `InnerState`
- applying delete mutations to `InnerState`

Keep persistence and tray refresh in the command handlers, but move the testable mutation rules into helpers that accept plain Rust data.

## Save Coverage

Tests should verify that saving:
- inserts a new tunnel into `inner.tunnels`
- moves the saved tunnel ID to the front of `recent_tunnel_ids`
- updates an existing tunnel instead of duplicating it
- for password auth, clears `private_key_path` and sets `password_entry`
- for private-key auth, clears `password_entry`

## Delete Coverage

Tests should verify that deleting:
- removes the tunnel from `inner.tunnels`
- removes any runtime entry from `inner.runtimes`
- removes the tunnel ID from `recent_tunnel_ids`
- persists the reduced config set through the existing config writer

## Testing Strategy

- Add Rust unit tests in `src-tauri/src/lib.rs`
- Prefer temporary config files over mocking persistence
- Keep tests off the Tauri `AppHandle` path; cover the command logic at the extracted-helper boundary
- Re-run `sshtunnel-app` lib tests plus existing core/frontend checks after implementation

## Non-Goals

- No full command invocation tests through Tauri runtime in this iteration
- No tray-refresh integration tests in this iteration
- No platform keyring integration tests in this iteration
