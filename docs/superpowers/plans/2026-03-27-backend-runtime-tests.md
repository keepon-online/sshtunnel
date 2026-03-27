# Backend Runtime Tests Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add direct Rust coverage for backend runtime log trimming, process refresh transitions, and disconnect behavior.

**Architecture:** Keep tests close to `src-tauri/src/lib.rs` and exercise the existing runtime helpers with real `ManagedProcess` instances created from short shell commands. Avoid command-level Tauri setup and only make minimal production changes if the helpers are not directly testable.

**Tech Stack:** Rust, existing `ManagedProcess`, standard process spawning, crate-local unit tests

---

## File Structure

- Modify: `src-tauri/src/lib.rs`
  - Add runtime regression tests and any tiny test-friendly helper exposure if required.
- Modify: `README.md`
  - Update documented backend test coverage.
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`
  - Narrow the remaining backlog after adding runtime helper tests.

## Chunk 1: Runtime Helper Tests

### Task 1: Add failing regression tests

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing test**

Add tests for:
- `push_log` trimming to the most recent 12 entries
- `refresh_runtime` returning `Idle` after a zero-exit process
- `refresh_runtime` returning `Error` and setting `last_error` after a non-zero exit
- `disconnect_runtime` clearing the process and adding a stop log

- [ ] **Step 2: Run test to verify it fails**

Run: `/home/top/.cargo/bin/cargo test -p sshtunnel-app runtime_tests --lib`
Expected: FAIL because the tests or required access paths do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Add the smallest code changes needed to make the tests compile and pass. Prefer test-only helpers over broad refactors.

- [ ] **Step 4: Run test to verify it passes**

Run: `/home/top/.cargo/bin/cargo test -p sshtunnel-app runtime_tests --lib`
Expected: PASS.

## Chunk 2: Docs And Verification

### Task 2: Update status docs and verify everything

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`

- [ ] **Step 1: Update docs**

Record the stronger backend runtime helper coverage and keep the remaining backlog focused on deeper integration tests plus Windows validation.

- [ ] **Step 2: Run full verification**

Run:
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-core`
- `node --test web/tests/view-model.test.js`
- `node --check web/app.js`
- `/home/top/.cargo/bin/cargo check -p sshtunnel-app`

Expected: all commands pass.
