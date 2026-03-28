# Connect/Disconnect/Autostart Command Tests Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add helper-level Rust coverage for the remaining connect, disconnect, and autostart command flows while keeping the Tauri command API unchanged.

**Architecture:** Extract small business-rule helpers from `src-tauri/src/lib.rs` for connect preparation, connect success application, disconnect application, and autostart branch selection. Keep tray refresh, plugin calls, and command signatures in place.

**Tech Stack:** Rust, Tauri backend crate, existing `ManagedProcess` runtime helpers, standard library closures for seam injection

---

## File Structure

- Modify: `src-tauri/src/lib.rs`
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`

## Chunk 1: Connect Helper Coverage

### Task 1: Add failing tests for connect preparation and success application

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing test**

Add tests for:
- unknown tunnel ID returns an error
- missing password credential sets an error and log instead of preparing a launch
- successful connect application clears stale runtime state and moves the tunnel to the front of recent items

- [ ] **Step 2: Run test to verify it fails**

Run: `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib command_flow_tests`
Expected: FAIL because the new helpers do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Extract the minimal connect helpers and wire `connect_tunnel` through them.

- [ ] **Step 4: Run test to verify it passes**

Run: `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib command_flow_tests`
Expected: PASS.

## Chunk 2: Disconnect And Autostart Coverage

### Task 2: Add failing tests for disconnect and autostart seams

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing test**

Add tests for:
- disconnecting a running tunnel stops the runtime and updates recent ordering
- disconnecting without a runtime remains idempotent
- autostart branch selection chooses enable/disable correctly and preserves errors

- [ ] **Step 2: Run test to verify it fails**

Run: `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib command_flow_tests`
Expected: FAIL because the disconnect/autostart helpers do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Extract the minimal disconnect and autostart helpers and wire the command paths through them.

- [ ] **Step 4: Run test to verify it passes**

Run: `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib command_flow_tests`
Expected: PASS.

## Chunk 3: Docs And Verification

### Task 3: Update backlog docs and verify

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`

- [ ] **Step 1: Update docs**

Record that command-flow coverage now includes connect/disconnect/autostart helper tests and narrow the backlog to remaining real-machine validation or deeper integration if still needed.

- [ ] **Step 2: Run targeted verification**

Run:
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib command_flow_tests`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-core`
- `node --test web/tests/view-model.test.js`
- `node --check web/app.js`
- `/home/top/.cargo/bin/cargo check -p sshtunnel-app`

Expected: all commands pass.
