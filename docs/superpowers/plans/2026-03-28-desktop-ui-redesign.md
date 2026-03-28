# Desktop UI Redesign Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign the frontend into a tray-first desktop utility layout with a focused workspace and drawer-based editing.

**Architecture:** Keep the backend snapshot and commands unchanged. Move display copy into `web/view-model.js` where helpful, then rebuild `web/index.html`, `web/styles.css`, and `web/app.js` around a left-list/right-workspace layout plus a reusable edit drawer.

**Tech Stack:** Static HTML/CSS/JS frontend, existing Tauri command bridge, Node `node:test` for pure view-model helpers

---

## File Structure

- Modify: `web/view-model.js`
  - Add pure helpers for workspace copy and editor drawer mode.
- Modify: `web/tests/view-model.test.js`
  - Add failing tests for empty-state and drawer/workspace copy.
- Modify: `web/index.html`
  - Replace the form-heavy main window with left rail, right workspace, and edit drawer structure.
- Modify: `web/styles.css`
  - Implement the new desktop-tool visual language and responsive layout.
- Modify: `web/app.js`
  - Manage selected tunnel workspace rendering and drawer open/close/edit flows.
- Modify: `README.md`
  - Update the current project status and remaining UI testing backlog.
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`
  - Record that the desktop UI has been redesigned around tray-first usage.

## Chunk 1: New Frontend Copy Helpers

### Task 1: Add failing tests for workspace and drawer copy

**Files:**
- Modify: `web/tests/view-model.test.js`
- Modify: `web/view-model.js`

- [ ] **Step 1: Write the failing test**

Add tests for:
- workspace empty-state copy when no tunnel is selected
- workspace title/subtitle for a selected tunnel
- editor drawer title and submit label for new vs edit mode

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test web/tests/view-model.test.js`
Expected: FAIL because the new helpers do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Add the new helpers to `web/view-model.js` and export them.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test web/tests/view-model.test.js`
Expected: PASS.

## Chunk 2: Layout And Interaction Rewrite

### Task 2: Rebuild the window around workspace + drawer

**Files:**
- Modify: `web/index.html`
- Modify: `web/styles.css`
- Modify: `web/app.js`

- [ ] **Step 1: Update HTML structure**

Replace the always-visible form layout with:
- left rail
- workspace header + summary cards + logs
- hidden drawer overlay for create/edit

- [ ] **Step 2: Implement CSS redesign**

Apply the approved desktop-tool layout, surface hierarchy, status styles, and responsive drawer behavior.

- [ ] **Step 3: Update app logic**

Add drawer open/close state, edit/new flow handling, workspace rendering, and save behavior that closes the drawer on success.

- [ ] **Step 4: Run targeted verification**

Run:
- `node --test web/tests/view-model.test.js`
- `node --check web/app.js`

Expected: both pass.

## Chunk 3: Docs And Full Verification

### Task 3: Update project status and verify the full stack

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`

- [ ] **Step 1: Update docs**

Record the new tray-first desktop window design and keep the remaining backlog focused on status/log rendering coverage and Tauri command flow tests.

- [ ] **Step 2: Run full verification**

Run:
- `node --test web/tests/view-model.test.js`
- `node --check web/app.js`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-core`
- `/home/top/.cargo/bin/cargo check -p sshtunnel-app`

Expected: all commands pass.
