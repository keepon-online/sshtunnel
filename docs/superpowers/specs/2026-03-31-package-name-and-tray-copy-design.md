# Package Name And Tray Copy Design

**Date:** 2026-03-31

## Goal

Make packaged installer artifacts use the hyphenated machine-friendly name `ssh-tunnel-manager`, and simplify the tray menu entry that opens the main window to `打开主界面`.

## Current Context

- Tauri packaging currently uses `productName: "SSH Tunnel Manager"`.
- The current tray menu entry reads `打开 SSH 隧道管理器`.
- The user wants installer/package naming without spaces, while the tray copy should be shorter and more task-oriented.

## Chosen Approach

Keep the change intentionally small and split by responsibility:

- update Tauri packaging display/product naming to `ssh-tunnel-manager` so generated installer artifacts stop using spaced names
- update the tray menu open action copy to `打开主界面`
- add a lightweight Rust regression test around the tray copy so future menu refactors do not silently revert the wording

## Scope

- `src-tauri/tauri.conf.json` package-facing product naming
- `src-tauri/src/lib.rs` tray menu open label
- Rust unit test coverage for the tray open label

## Testing Strategy

- Add a Rust unit test that asserts the tray open label text.
- Run `cargo test --workspace`.
- Run `cargo check -p sshtunnel-app`.

## Non-Goals

- No tray interaction behavior changes
- No README update
- No redesign of main window title or other user-facing copy beyond the requested tray item
