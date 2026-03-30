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

The SSH child-process paths already request hidden process creation:
- the native `Command` path applies `CREATE_NO_WINDOW`
- the vendored `portable-pty` Windows spawn path also includes `CREATE_NO_WINDOW`

The remaining visible black window came from the Tauri Windows binary entrypoint itself: `src-tauri/src/main.rs` did not declare the Windows GUI subsystem, so the release app still had a console attached.

## Chosen Approach

Keep the current system-SSH architecture and hide the app console at the binary entrypoint:

- Add `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` to `src-tauri/src/main.rs`.
- Keep the existing hidden child-process behavior in `managed_process.rs` and the vendored `portable-pty` Windows path.
- Add a regression test in `src-tauri/src/main.rs` that asserts the source still contains the Windows subsystem attribute.

This keeps the existing SSH command line, log capture, and password prompt handling intact while removing the visible console window from the Windows release build.

## Testing Strategy

- Add a focused unit test in `src-tauri/src/main.rs` for the Windows subsystem attribute.
- Run `cargo test -p sshtunnel-app` to confirm the application test suite still passes.
- Validate the packaged Windows build on a real machine and confirm tunnel launch no longer shows a black console window.

## Non-Goals

- No SSH argument changes
- No replacement of `portable-pty`
- No changes to Linux process behavior
