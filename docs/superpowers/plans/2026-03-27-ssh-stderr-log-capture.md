# SSH Stderr Log Capture Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Capture live `ssh` stderr output for native child processes and surface it in the existing runtime recent log.

**Architecture:** Extend `ManagedProcess` so native process launches pipe `stderr` into a background reader thread backed by the same shared log buffer already used by PTY sessions. Keep runtime log consumption inside `src-tauri/src/lib.rs` unchanged except for receiving the additional lines.

**Tech Stack:** Rust, std::process, std::thread, Tauri backend runtime, existing `ManagedProcess` abstraction

---

## File Structure

- Modify: `src-tauri/src/managed_process.rs`
  - Add native child wrapper state, stderr reader lifecycle, and focused unit tests.
- Modify: `README.md`
  - Update the documented project status and remaining limitations.
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`
  - Mark live stderr capture as completed and leave the remaining backlog accurate.

## Chunk 1: Native Stderr Capture

### Task 1: Add managed-process regression tests

**Files:**
- Modify: `src-tauri/src/managed_process.rs`
- Test: `src-tauri/src/managed_process.rs`

- [ ] **Step 1: Write the failing test**

Add a unit test that spawns a short native command which writes at least two lines to `stderr`, waits briefly, and asserts that `take_logs()` returns both lines.

- [ ] **Step 2: Run test to verify it fails**

Run: `/home/top/.cargo/bin/cargo test -p sshtunnel-app native_process_captures_stderr_lines --lib`
Expected: FAIL because native process logs are currently discarded.

- [ ] **Step 3: Write minimal implementation**

Introduce a native process wrapper that owns the child plus a stderr reader thread, pipe `stderr`, feed lines into the shared log buffer, and join the reader during shutdown.

- [ ] **Step 4: Run test to verify it passes**

Run: `/home/top/.cargo/bin/cargo test -p sshtunnel-app native_process_captures_stderr_lines --lib`
Expected: PASS.

## Chunk 2: Docs And Verification

### Task 2: Update project status docs

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`

- [ ] **Step 1: Update docs**

Record that live stderr capture is implemented and keep the remaining backlog focused on stronger automated tests and Windows real-machine validation.

- [ ] **Step 2: Run full verification**

Run:
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-core`
- `node --test web/tests/view-model.test.js`
- `node --check web/app.js`
- `/home/top/.cargo/bin/cargo check -p sshtunnel-app`

Expected: all commands pass.
