# Backend Runtime Transition Tests Design

**Date:** 2026-03-27

## Goal

Add focused Rust tests for backend runtime helpers so core tunnel state transitions and recent-log behavior are verified without needing full Tauri command integration tests.

## Current Context

- `src-tauri/src/lib.rs` contains the state machine behavior for runtime refresh, disconnect handling, and recent-log trimming.
- These helpers currently have no direct tests even though they control the user-visible `idle`, `connected`, and `error` states.
- Full command-level testing would pull in Tauri app setup, tray behavior, and platform services, which is wider than needed for the immediate regression risk.

## Chosen Approach

Add unit tests alongside `src-tauri/src/lib.rs` that exercise `push_log`, `refresh_runtime`, and `disconnect_runtime` using real `ManagedProcess` instances launched from short-lived local commands. Keep production refactoring minimal and only add test-only helpers if access boundaries require them.

## Responsibilities

- `src-tauri/src/lib.rs`
  - Host runtime regression tests
  - Optionally expose tiny test-only helpers if direct construction is awkward
- `README.md`
  - Record stronger backend runtime test coverage
- `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`
  - Update the remaining backlog to focus on deeper command-level testing and Windows validation

## Testing Strategy

- `push_log` test: append more than 12 entries and assert only the latest 12 remain.
- `refresh_runtime` success test: spawn a short command that exits with status 0 and assert the runtime becomes `Idle` with an exit log.
- `refresh_runtime` failure test: spawn a short command that exits non-zero and assert the runtime becomes `Error` with `last_error` populated.
- `disconnect_runtime` test: spawn a longer-running command, disconnect it, and assert the process is cleared and a stop log is recorded.

## Error Handling

- Tests use real `ManagedProcess` execution, so they validate the same process lifecycle code as production.
- Platform-specific shell commands are isolated behind test helpers for Linux/macOS versus Windows.
- Tests avoid Tauri `AppHandle` or tray interactions entirely.

## Non-Goals

- No Tauri command invocation tests in this iteration
- No tray integration coverage
- No Windows real-machine SSH validation
- No redesign of runtime state structures
