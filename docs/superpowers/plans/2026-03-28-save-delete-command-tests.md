# Save/Delete Command Tests Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Cover `save_tunnel` and `delete_tunnel` backend mutation rules with Rust tests and minimal helper extraction inside `src-tauri/src/lib.rs`.

**Architecture:** Keep Tauri command signatures intact, but extract save/delete normalization and state-mutation helpers so the behavior can be tested directly with plain `InnerState` values and temporary config files.

**Tech Stack:** Rust, Tauri backend crate, existing `sshtunnel-core` models, standard library temp paths

---

## File Structure

- Modify: `src-tauri/src/lib.rs`
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`

## Chunk 1: Save Helper Coverage

### Task 1: Add failing tests for save mutation rules

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing test**

Add tests for:
- saving a new tunnel inserts it and updates recents
- saving an existing tunnel replaces instead of duplicating
- password-auth save normalization clears `private_key_path` and sets `password_entry`

- [ ] **Step 2: Run test to verify it fails**

Run: `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib save_delete_tests`
Expected: FAIL because the helper functions and tests do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Extract minimal save helpers and keep command behavior unchanged.

- [ ] **Step 4: Run test to verify it passes**

Run: `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib save_delete_tests`
Expected: PASS.

## Chunk 2: Delete Helper Coverage

### Task 2: Add failing tests for delete mutation rules

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing test**

Add a test that deletes a tunnel and verifies:
- `tunnels` entry removed
- `runtimes` entry removed
- `recent_tunnel_ids` entry removed
- persisted config only contains surviving tunnels

- [ ] **Step 2: Run test to verify it fails**

Run: `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib save_delete_tests`
Expected: FAIL because the delete helper does not exist yet.

- [ ] **Step 3: Write minimal implementation**

Extract minimal delete helpers and call them from the command path before persistence.

- [ ] **Step 4: Run test to verify it passes**

Run: `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib save_delete_tests`
Expected: PASS.

## Chunk 3: Docs And Verification

### Task 3: Update backlog docs and verify

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`

- [ ] **Step 1: Update docs**

Record that command-flow tests now include save/delete coverage and narrow the remaining backlog accordingly.

- [ ] **Step 2: Run targeted verification**

Run:
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib save_delete_tests`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-core`
- `node --test web/tests/view-model.test.js`
- `node --check web/app.js`
- `/home/top/.cargo/bin/cargo check -p sshtunnel-app`

Expected: all commands pass.
