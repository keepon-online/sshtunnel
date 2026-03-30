# Windows Hide SSH Console Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stop the Windows build from showing a console window when SSH tunnels are started, for both native and password-prompt launch paths.

**Architecture:** Keep the existing hidden SSH child-process behavior, hide the Windows app console at the Tauri binary entrypoint with `windows_subsystem = "windows"`, and lock that behavior with a regression test in `src-tauri/src/main.rs`.

**Tech Stack:** Rust, Tauri 2, system OpenSSH, portable-pty

---

## File Structure

- Modify: `src-tauri/src/main.rs`
- Create: `docs/superpowers/specs/2026-03-30-windows-hide-ssh-console-design.md`
- Create: `docs/superpowers/plans/2026-03-30-windows-hide-ssh-console.md`

## Chunk 1: Failing Test For Windows GUI Subsystem

### Task 1: Add the helper test

**Files:**
- Modify: `src-tauri/src/main.rs`

- [x] **Step 1: Write the failing test**

Add a unit test in `src-tauri/src/main.rs` that asserts the source contains `windows_subsystem = "windows"`.

- [x] **Step 2: Run test to verify it fails**

Run:
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app --bin sshtunnel-app tests::windows_release_build_uses_gui_subsystem -- --exact`

Expected: FAIL because the subsystem attribute does not exist yet.

## Chunk 2: Windows Release GUI Subsystem

### Task 2: Implement the Tauri entrypoint fix

**Files:**
- Modify: `src-tauri/src/main.rs`

- [x] **Step 1: Write minimal implementation**

Add `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` to the Tauri binary entrypoint.

- [x] **Step 2: Run targeted verification**

Run:
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app --bin sshtunnel-app tests::windows_release_build_uses_gui_subsystem -- --exact`

Expected: PASS.

## Chunk 3: Full Verification

### Task 3: Verify release behavior and document outcome

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/specs/2026-03-30-windows-hide-ssh-console-design.md`
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`

- [x] **Step 1: Run full verification**

Run:
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app`

- [x] **Step 2: Confirm packaged Windows behavior**

Confirm on a real Windows packaged build that tunnel launch no longer shows a black console window and that connection behavior remains normal.

Expected: PASS.
