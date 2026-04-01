# Windows Command Center Fit Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the main desktop UI fit better on Windows by relaxing full-height layout assumptions and stabilizing the diagnostic log region without changing tunnel behavior.

**Architecture:** Keep the existing static frontend structure and tunnel workflow, then solve the fit issue through three focused changes: Tauri window sizing, CSS layout constraints, and smarter frontend log auto-follow logic. Use the current Node-based frontend tests and Rust library tests as regression coverage.

**Tech Stack:** Tauri 2 config, static HTML/CSS/JS frontend, Node `node:test`, Rust `cargo test`

---

## File Structure

- Modify: `src-tauri/tauri.conf.json`
- Modify: `web/styles.css`
- Modify: `web/app.js`
- Modify: `web/tests/app.test.js`
- Add: `docs/superpowers/specs/2026-04-01-windows-command-center-fit-design.md`
- Add: `docs/superpowers/plans/2026-04-01-windows-command-center-fit.md`

## Chunk 1: Regression Coverage

### Task 1: Add failing assertions for window sizing and layout behavior

**Files:**
- Modify: `web/tests/app.test.js`

- [ ] **Step 1: Write the failing tests**

Add assertions that:
- the main Tauri window config is centered and uses reduced height pressure
- the shell layout no longer depends on `height: 100vh` plus `overflow: hidden`
- the log panel keeps a bounded height
- log rendering only auto-scrolls when the user was already near the bottom

- [ ] **Step 2: Run tests to verify they fail**

Run: `node --test web/tests/app.test.js`
Expected: FAIL on the new layout and auto-follow assertions.

## Chunk 2: Desktop Layout Adjustment

### Task 2: Relax full-height layout assumptions and resize the main window

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: `web/styles.css`

- [ ] **Step 1: Write minimal implementation**

Update the Tauri window defaults for better Windows fit and make the page shell vertically scrollable while preserving the current two-column desktop layout.

- [ ] **Step 2: Run targeted verification**

Run: `node --test web/tests/app.test.js`
Expected: PASS for config and CSS assertions.

## Chunk 3: Log Panel Auto-Follow Refinement

### Task 3: Keep logs readable without fighting the user’s scroll position

**Files:**
- Modify: `web/app.js`
- Modify: `web/tests/app.test.js`

- [ ] **Step 1: Write minimal implementation**

Adjust log rendering so it follows new output only when the panel was already near the bottom before re-rendering.

- [ ] **Step 2: Run full verification**

Run:
- `node --test web/tests/app.test.js`
- `cargo test --lib`

Expected: PASS.
