# Package Name And Tray Copy Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rename packaged artifacts to `ssh-tunnel-manager` and simplify the tray open action copy to `打开主界面`.

**Architecture:** Keep packaging and tray wording changes isolated. Use TDD for the tray wording by extracting or asserting the open-label text in Rust before changing the implementation, then update Tauri packaging config separately without widening the behavior surface.

**Tech Stack:** Tauri 2, Rust unit tests, JSON config

---

## File Structure

- Add: `docs/superpowers/specs/2026-03-31-package-name-and-tray-copy-design.md`
- Add: `docs/superpowers/plans/2026-03-31-package-name-and-tray-copy.md`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/tauri.conf.json`

## Chunk 1: Tray Copy Regression

### Task 1: Add a failing Rust assertion for the tray open label

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing test**

Add a small unit test that asserts the tray open menu label is `打开主界面`.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sshtunnel-app tray_open_label -- --nocapture`
Expected: FAIL because the current label is still `打开 SSH 隧道管理器`.

## Chunk 2: Minimal Implementation

### Task 2: Update tray copy and package naming

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Write minimal implementation**

Update the tray open action label to `打开主界面`, and set the Tauri product name to `ssh-tunnel-manager`.

- [ ] **Step 2: Run verification**

Run:
- `cargo test --workspace`
- `cargo check -p sshtunnel-app`

Expected: PASS.
