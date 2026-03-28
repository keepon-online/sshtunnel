# Tray Disconnect-All Tests Design

**Date:** 2026-03-28

## Goal

Add focused backend coverage for the tray `disconnect_all` behavior by extracting and testing a helper that applies the batch disconnect semantics to `InnerState`.

## Current Context

- Tray item ordering and labels already have pure model coverage.
- Single-tunnel disconnect logic already has helper coverage.
- The tray `disconnect_all` branch still performs its batch loop inline inside the tray event handler.

## Chosen Approach

Extract an `apply_disconnect_all(inner)` helper inside `src-tauri/src/lib.rs` and make the tray event branch call that helper before refreshing the tray menu.

This keeps the tray runtime and menu event objects out of scope while still testing the business behavior that matters.

## Coverage

Tests should verify that `apply_disconnect_all`:
- disconnects every running tunnel
- clears stale runtime errors
- appends `stopped ssh process` logs for tunnels that had active processes
- moves all tunnels into recent order without duplicates
- does not create new runtime entries for tunnels that were already idle

## Testing Strategy

- Add Rust unit tests in `src-tauri/src/lib.rs`
- Reuse the existing `ManagedProcess` test style for running runtimes
- Keep this at the state-helper layer and avoid real tray runtime setup

## Non-Goals

- No real tray click integration tests
- No assertions about tray menu refresh success
- No menu label assertions in this iteration
