# SSH Stderr Log Capture Design

**Date:** 2026-03-27

## Goal

Capture live `ssh` output written to `stderr` and surface it through the existing `recent_log` field so connection failures and authentication issues are visible in the desktop UI.

## Current Context

- `ManagedProcess` already owns process spawning and low-level output collection.
- Prompted password sessions run under a PTY and already stream transcript lines into an internal log buffer.
- Native process sessions currently discard `stderr`, so key-based connection failures are only visible through exit status and lifecycle messages.
- The Tauri backend already drains process logs through `flush_process_logs()` and appends them to `TunnelRuntime.recent_log`.

## Chosen Approach

Extend `ManagedProcess` so native child processes pipe `stderr` into a background reader thread, using the same internal string log buffer already used by PTY sessions. Keep `ManagedProcess::take_logs()` as the only log-consumption interface. Do not introduce a new event model in this iteration.

## Responsibilities

- `src-tauri/src/managed_process.rs`
  - Spawn native processes with a piped `stderr`
  - Read `stderr` on a background thread
  - Append reader errors and captured lines into the shared log buffer
  - Own thread lifecycle and cleanup for both native and prompted process modes
- `src-tauri/src/lib.rs`
  - Continue draining logs through `flush_process_logs()`
  - No API or UI payload changes

## Error Handling

- If the native process `stderr` pipe cannot be obtained, process spawn fails immediately.
- Reader thread I/O failures are appended into the log buffer as readable text.
- Existing lifecycle logs remain in place so process start and stop transitions are still visible alongside live output.

## Testing Strategy

- Add focused unit tests in `managed_process.rs` for native process log capture.
- Use a short platform-specific shell command that writes known lines to `stderr`, then exits.
- Verify that `ManagedProcess::take_logs()` returns those lines.
- Re-run the existing app and core verification commands to ensure no regressions.

## Non-Goals

- No structured log event format
- No frontend redesign for log display
- No PTY architecture changes beyond preserving current behavior
- No Windows-specific credential flow changes in this iteration
