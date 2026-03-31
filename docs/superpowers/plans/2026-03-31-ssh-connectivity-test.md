# SSH Connectivity Test Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an editor-drawer `测试连接` action that verifies SSH login and remote target reachability using unsaved form values, then writes the result into the drawer and main diagnostic timeline without mutating real tunnel runtime state.

**Architecture:** Keep the feature off the live tunnel runtime path. Add a dedicated backend test command plus app-level diagnostic log storage, then wire the drawer to call it and render a one-shot result area. Reuse existing validation and SSH launch pieces where they fit, but treat connectivity testing as an isolated command flow.

**Tech Stack:** Tauri 2, Rust, static HTML/CSS/JS frontend, Node `node:test`

---

## File Structure

- Add: `docs/superpowers/specs/2026-03-31-ssh-connectivity-test-design.md`
- Add: `docs/superpowers/plans/2026-03-31-ssh-connectivity-test.md`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/managed_process.rs`
- Modify: `crates/core/src/ssh_args.rs` or neighboring SSH command helpers if extraction is needed
- Modify: `web/index.html`
- Modify: `web/app.js`
- Modify: `web/view-model.js`
- Modify: `web/tests/app.test.js`

## Chunk 1: Backend Regression Tests

### Task 1: Add failing Rust tests for one-shot connectivity checks and test-log snapshots

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/managed_process.rs` if helper tests fit better there

- [ ] **Step 1: Write failing tests**

Add focused tests that cover:
- SSH login success + target success
- SSH login success + target failure
- SSH login failure short-circuits target check
- app-level test logs appear in snapshot/timeline payload

- [ ] **Step 2: Run tests to verify they fail**

Run: `env PATH=$HOME/.cargo/bin:$PATH cargo test -p sshtunnel-app connectivity_test -- --nocapture`
Expected: FAIL because the backend connectivity test flow does not exist yet.

## Chunk 2: Backend Connectivity Test Command

### Task 2: Implement one-shot connectivity testing without touching real runtime state

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/managed_process.rs`
- Modify: `crates/core/src/ssh_args.rs` or extracted SSH helper module if needed

- [ ] **Step 1: Implement minimal backend command**

Add:
- payload/result structs
- app-level recent diagnostic test logs
- one-shot SSH login probe
- one-shot remote target probe
- snapshot integration
- `test_tunnel_connectivity` command registration

- [ ] **Step 2: Run backend verification**

Run:
- `env PATH=$HOME/.cargo/bin:$PATH cargo test -p sshtunnel-app connectivity_test -- --nocapture`
- `env PATH=$HOME/.cargo/bin:$PATH cargo test -p sshtunnel-app recent_tray_items_do_not_mark_prompted_waiting_session_connected -- --nocapture`

Expected: PASS.

## Chunk 3: Frontend Regression Tests

### Task 3: Add failing frontend tests for drawer test action and timeline mapping

**Files:**
- Modify: `web/tests/app.test.js`

- [ ] **Step 1: Write failing tests**

Add tests that cover:
- drawer exposes a test action
- test result state renders a useful summary
- timeline mapping keeps `[测试]` status summaries and raw SSH output visible

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test web/tests/app.test.js`
Expected: FAIL on the new connectivity-test assertions.

## Chunk 4: Drawer UI And Client Command Flow

### Task 4: Implement the drawer test button, loading state, result area, and snapshot refresh

**Files:**
- Modify: `web/index.html`
- Modify: `web/app.js`
- Modify: `web/view-model.js`

- [ ] **Step 1: Implement minimal frontend behavior**

Add:
- `测试连接` button
- drawer result area
- loading/disabled state while request is running
- command invocation using unsaved form payload
- render updates that include newly appended test logs in the timeline

- [ ] **Step 2: Run frontend verification**

Run:
- `node --test web/tests/app.test.js`
- `node --check web/app.js`

Expected: PASS.

## Chunk 5: Full Verification

### Task 5: Run complete verification on the integrated feature

**Files:**
- Modify: none

- [ ] **Step 1: Run complete verification**

Run:
- `env PATH=$HOME/.cargo/bin:$PATH cargo test --workspace`
- `env PATH=$HOME/.cargo/bin:$PATH cargo check -p sshtunnel-app`
- `node --test web/tests/app.test.js`
- `node --check web/app.js`

Expected: PASS.

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/managed_process.rs crates/core/src/ssh_args.rs web/index.html web/app.js web/view-model.js web/tests/app.test.js docs/superpowers/specs/2026-03-31-ssh-connectivity-test-design.md docs/superpowers/plans/2026-03-31-ssh-connectivity-test.md
git commit -m "feat: add ssh connectivity testing"
```
