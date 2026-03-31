# Audit Fixes Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Repair the audited security and behavior gaps in the SSH tunnel desktop app without changing the overall product structure.

**Architecture:** Keep the existing static frontend and Tauri backend, but tighten each layer at its current boundary: safe DOM rendering in the frontend, stricter backend save validation, explicit SSH runtime signal tracking, and lightweight startup/reconnect orchestration in the backend runtime. Implement each behavior test-first and keep the reconnect logic intentionally small.

**Tech Stack:** Rust workspace, Tauri 2, static HTML/CSS/JS frontend, Node `node:test`

---

## File Structure

- Modify: `web/app.js`
- Modify: `web/tests/app.test.js`
- Modify: `crates/core/src/ssh_args.rs`
- Modify: `crates/core/tests/ssh_launch.rs`
- Modify: `src-tauri/src/managed_process.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `README.md`
- Add: `docs/superpowers/specs/2026-03-31-audit-fixes-design.md`
- Add: `docs/superpowers/plans/2026-03-31-audit-fixes.md`

## Chunk 1: Frontend Safety

### Task 1: Replace unsafe list HTML rendering with safe text nodes

**Files:**
- Modify: `web/app.js`
- Modify: `web/tests/app.test.js`

- [ ] **Step 1: Write the failing test**

Add a frontend test proving tunnel list values containing HTML-like strings are rendered as text, not parsed markup.

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test web/tests/app.test.js`
Expected: FAIL because list rendering still relies on `innerHTML`.

- [ ] **Step 3: Write minimal implementation**

Refactor list rendering in `web/app.js` to build button contents with `createElement` and `textContent` only.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test web/tests/app.test.js`
Expected: PASS.

## Chunk 2: SSH Args And Runtime Status

### Task 2: Add password-auth host key policy and runtime connection signal tracking

**Files:**
- Modify: `crates/core/src/ssh_args.rs`
- Modify: `crates/core/tests/ssh_launch.rs`
- Modify: `src-tauri/src/managed_process.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

Add Rust tests for:
- password-auth SSH args include `StrictHostKeyChecking=accept-new`
- runtime does not report `connected` for an interactive prompt
- runtime reports error on explicit SSH failure output

- [ ] **Step 2: Run tests to verify they fail**

Run:
- `env PATH=$HOME/.cargo/bin:$PATH cargo test -p sshtunnel-core --test ssh_launch`
- `env PATH=$HOME/.cargo/bin:$PATH cargo test -p sshtunnel-app --lib runtime_tests`

Expected: FAIL on the new assertions.

- [ ] **Step 3: Write minimal implementation**

Update SSH args and managed-process/runtime signal handling so blocked sessions are not marked connected and first-use host keys are auto-accepted.

- [ ] **Step 4: Run tests to verify they pass**

Run:
- `env PATH=$HOME/.cargo/bin:$PATH cargo test -p sshtunnel-core --test ssh_launch`
- `env PATH=$HOME/.cargo/bin:$PATH cargo test -p sshtunnel-app --lib runtime_tests`

Expected: PASS.

## Chunk 3: Backend Credential Enforcement

### Task 3: Reject password-auth saves that do not have a usable credential

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

Add backend tests covering:
- new password-auth tunnel save without password is rejected
- edited password-auth tunnel save without new password succeeds only if stored credential exists

- [ ] **Step 2: Run test to verify it fails**

Run: `env PATH=$HOME/.cargo/bin:$PATH cargo test -p sshtunnel-app --lib save_delete_tests`
Expected: FAIL on the new save-validation assertions.

- [ ] **Step 3: Write minimal implementation**

Introduce backend-only credential validation in the save flow before persisting password-auth tunnels.

- [ ] **Step 4: Run test to verify it passes**

Run: `env PATH=$HOME/.cargo/bin:$PATH cargo test -p sshtunnel-app --lib save_delete_tests`
Expected: PASS.

## Chunk 4: Auto Connect And Auto Reconnect

### Task 4: Make persisted auto-connect and runtime auto-reconnect behavior real

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `README.md`

- [ ] **Step 1: Write the failing tests**

Add backend tests covering:
- startup attempts to connect tunnels marked `auto_connect`
- abnormal process exit schedules reconnect when `auto_reconnect` is enabled
- user disconnect suppresses reconnect

- [ ] **Step 2: Run test to verify it fails**

Run: `env PATH=$HOME/.cargo/bin:$PATH cargo test -p sshtunnel-app --lib command_flow_tests`
Expected: FAIL on the new orchestration assertions.

- [ ] **Step 3: Write minimal implementation**

Add startup auto-connect orchestration and simple delayed auto-reconnect scheduling for unexpected exits only.

- [ ] **Step 4: Run test to verify it passes**

Run: `env PATH=$HOME/.cargo/bin:$PATH cargo test -p sshtunnel-app --lib command_flow_tests`
Expected: PASS.

## Chunk 5: Final Verification

### Task 5: Run full regression verification

**Files:**
- Modify: `web/app.js`
- Modify: `web/tests/app.test.js`
- Modify: `crates/core/src/ssh_args.rs`
- Modify: `crates/core/tests/ssh_launch.rs`
- Modify: `src-tauri/src/managed_process.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `README.md`

- [ ] **Step 1: Run frontend verification**

Run:
- `node --test web/tests/app.test.js`

Expected: PASS.

- [ ] **Step 2: Run full Rust verification**

Run:
- `env PATH=$HOME/.cargo/bin:$PATH cargo test --workspace`

Expected: PASS.

- [ ] **Step 3: Run compile verification**

Run:
- `env PATH=$HOME/.cargo/bin:$PATH cargo check -p sshtunnel-app`

Expected: PASS.
