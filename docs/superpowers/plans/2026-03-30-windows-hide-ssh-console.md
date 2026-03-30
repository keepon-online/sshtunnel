# Windows Hide SSH Console Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stop the Windows build from showing a console window when SSH tunnels are started, for both native and password-prompt launch paths.

**Architecture:** Keep process launching in the current backend, add a tiny Windows-only helper for native `Command` creation flags, and patch the local `portable-pty` Windows spawn code so both code paths request `CREATE_NO_WINDOW`. Preserve the existing SSH launch plan and runtime logging behavior.

**Tech Stack:** Rust, Tauri 2, system OpenSSH, portable-pty

---

## File Structure

- Modify: `Cargo.toml`
- Modify: `src-tauri/src/managed_process.rs`
- Create: `vendor/portable-pty/...`
- Create: `docs/superpowers/specs/2026-03-30-windows-hide-ssh-console-design.md`
- Create: `docs/superpowers/plans/2026-03-30-windows-hide-ssh-console.md`

## Chunk 1: Failing Test For Windows Spawn Flags

### Task 1: Add the helper test

**Files:**
- Modify: `src-tauri/src/managed_process.rs`

- [ ] **Step 1: Write the failing test**

Add a unit test for a small helper that returns the process creation flags used for hidden Windows SSH child processes.

- [ ] **Step 2: Run test to verify it fails**

Run:
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app managed_process`

Expected: FAIL because the helper does not exist yet.

## Chunk 2: Native SSH No-Window Launch

### Task 2: Implement the native Windows no-window helper

**Files:**
- Modify: `src-tauri/src/managed_process.rs`

- [ ] **Step 1: Write minimal implementation**

Add a Windows-only helper that applies `CREATE_NO_WINDOW` to `std::process::Command`, and call it from the native launch path before spawning `ssh`.

- [ ] **Step 2: Run targeted verification**

Run:
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app managed_process`

Expected: PASS.

## Chunk 3: Prompted Password No-Window Launch

### Task 3: Patch portable-pty for Windows

**Files:**
- Modify: `Cargo.toml`
- Create/Modify: `vendor/portable-pty/...`

- [ ] **Step 1: Vendor the dependency and patch Windows process creation flags**

Add a local `[patch.crates-io]` override for `portable-pty` and include `CREATE_NO_WINDOW` in its Windows `CreateProcessW` flags.

- [ ] **Step 2: Run full verification**

Run:
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app managed_process`
- `/home/top/.cargo/bin/cargo check -p sshtunnel-app`

Expected: PASS.
