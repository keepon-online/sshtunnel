# Tray Disconnect-All Tests Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add helper-level Rust coverage for tray `disconnect_all` behavior and route the tray branch through a dedicated batch disconnect helper.

**Architecture:** Extract `apply_disconnect_all(inner)` from the tray event handler in `src-tauri/src/lib.rs`. Reuse the existing disconnect primitives and cover the batch semantics with unit tests.

**Tech Stack:** Rust, Tauri backend crate, existing `ManagedProcess` runtime helpers

---

## File Structure

- Modify: `src-tauri/src/lib.rs`
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`

## Chunk 1: Disconnect-All Helper Coverage

### Task 1: Add failing tests for batch disconnect behavior

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing test**

Add tests for:
- batch disconnect stops all running runtimes and clears errors
- batch disconnect updates recent order without duplicates
- idle tunnels do not gain new runtime entries

- [ ] **Step 2: Run test to verify it fails**

Run: `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib tray_disconnect_all_tests`
Expected: FAIL because `apply_disconnect_all` does not exist yet.

- [ ] **Step 3: Write minimal implementation**

Extract `apply_disconnect_all(inner)` and route the tray `disconnect_all` branch through it.

- [ ] **Step 4: Run test to verify it passes**

Run: `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib tray_disconnect_all_tests`
Expected: PASS.

## Chunk 2: Docs And Verification

### Task 2: Update docs and verify

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`

- [ ] **Step 1: Update docs**

Record the new tray `disconnect_all` helper coverage and keep remaining tray risk focused on deeper runtime integration only if still needed.

- [ ] **Step 2: Run targeted verification**

Run:
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib tray_disconnect_all_tests`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-core`
- `node --test web/tests/view-model.test.js`
- `node --check web/app.js`
- `/home/top/.cargo/bin/cargo check -p sshtunnel-app`

Expected: all commands pass.
