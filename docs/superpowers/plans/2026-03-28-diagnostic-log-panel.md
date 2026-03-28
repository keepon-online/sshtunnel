# Diagnostic Log Panel Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Upgrade the workspace log panel into a grouped diagnostic view with status events, SSH output, and error highlighting.

**Architecture:** Keep backend `recent_log` unchanged and add pure classification helpers in `web/view-model.js`. Use those helpers in `web/app.js` to render two explicit log sections with counts and empty states.

**Tech Stack:** Static HTML/CSS/JS frontend, Node `node:test`, existing view-model module

---

## File Structure

- Modify: `web/view-model.js`
- Modify: `web/tests/view-model.test.js`
- Modify: `web/index.html`
- Modify: `web/styles.css`
- Modify: `web/app.js`
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`

## Chunk 1: Log Classification Helpers

### Task 1: Add failing tests and pure helpers

**Files:**
- Modify: `web/tests/view-model.test.js`
- Modify: `web/view-model.js`

- [ ] **Step 1: Write the failing test**

Add tests for:
- status-event grouping
- SSH-output grouping
- error highlighting
- empty summaries for no logs

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test web/tests/view-model.test.js`
Expected: FAIL because the new log grouping helpers do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Add the log grouping helper and export it.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test web/tests/view-model.test.js`
Expected: PASS.

## Chunk 2: Panel Rendering

### Task 2: Render grouped diagnostic sections

**Files:**
- Modify: `web/index.html`
- Modify: `web/styles.css`
- Modify: `web/app.js`

- [ ] **Step 1: Update markup**

Add grouped log panel containers for:
- summary line
- status events section
- SSH output section

- [ ] **Step 2: Update app rendering**

Render grouped logs and per-section empty states from the new helper.

- [ ] **Step 3: Update styles**

Add grouped diagnostic styling and error emphasis.

- [ ] **Step 4: Run targeted verification**

Run:
- `node --test web/tests/view-model.test.js`
- `node --check web/app.js`

Expected: PASS.

## Chunk 3: Docs And Full Verification

### Task 3: Update docs and verify

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`

- [ ] **Step 1: Update docs**

Record the grouped diagnostic log panel and keep the remaining backlog focused on Tauri command-flow tests and Windows validation.

- [ ] **Step 2: Run full verification**

Run:
- `node --test web/tests/view-model.test.js`
- `node --check web/app.js`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-core`
- `/home/top/.cargo/bin/cargo check -p sshtunnel-app`

Expected: all commands pass.
