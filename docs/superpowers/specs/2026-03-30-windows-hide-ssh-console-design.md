# Windows Hide SSH Console Design

**Date:** 2026-03-30

## Goal

Prevent the Windows desktop app from showing a flashing `cmd`/console window when it starts SSH tunnels, for both private-key and password authentication.

## Current Context

- Tunnel launch is centralized in `src-tauri/src/managed_process.rs`.
- Private-key auth uses `std::process::Command`.
- Password auth uses `portable-pty`, which creates a Windows pseudoconsole and then calls `CreateProcessW`.
- Windows users report that connecting a tunnel visibly launches a black console window.

## Root Cause

Neither Windows launch path requests hidden process creation:
- the native `Command` path does not set `CREATE_NO_WINDOW`
- the `portable-pty` Windows spawn path also omits `CREATE_NO_WINDOW`

Because the app delegates to the system `ssh.exe`, Windows creates a visible console host for that child process.

## Chosen Approach

Keep the current system-SSH architecture and suppress the console at process creation time:

- Add a small Windows-only helper in `managed_process.rs` that applies `CREATE_NO_WINDOW` to native child processes.
- Patch the local `portable-pty` dependency so its Windows `CreateProcessW` flags also include `CREATE_NO_WINDOW`.
- Wire the workspace to the patched local copy via `[patch.crates-io]`.

This keeps the existing SSH command line, log capture, and password prompt handling intact while removing the visible console window.

## Testing Strategy

- Add a focused unit test in `managed_process.rs` for the Windows creation-flags helper.
- Run existing `managed_process` tests on Linux to confirm the refactor does not break stderr/log capture.
- Run `cargo check -p sshtunnel-app` to validate the application still compiles on the local environment.

## Non-Goals

- No SSH argument changes
- No replacement of `portable-pty`
- No changes to Linux process behavior
