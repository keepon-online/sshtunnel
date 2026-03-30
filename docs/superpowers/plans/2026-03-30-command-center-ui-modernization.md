# Command Center UI Modernization Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild the desktop frontend into a command-center layout that makes the selected tunnel's status the dominant visual signal while preserving the tray-first workflow and existing backend behavior.

**Architecture:** Keep the Tauri command bridge, snapshot shape, and drawer-based editing flow. Expand the pure view-model copy helpers for the new hero/support-card/timeline presentation, then rewrite `web/index.html`, `web/styles.css`, and the workspace-rendering portions of `web/app.js` around the approved command-center layout.

**Tech Stack:** Static HTML/CSS/JS frontend, existing Tauri bridge, Node `node:test`, Rust/Tauri backend verification

---

## File Structure

- Modify: `web/view-model.js`
  - Add or adjust pure helpers for command-center hero copy, supporting cards, and timeline summaries.
- Modify: `web/tests/view-model.test.js`
  - Add failing tests for the new command-center empty state and selected-tunnel copy.
- Modify: `web/index.html`
  - Replace the current shell structure with the new left control column plus command-center workspace.
- Modify: `web/styles.css`
  - Introduce the new visual system, spacing, hierarchy, timeline styling, and responsive fallback.
- Modify: `web/app.js`
  - Rewire DOM references and render logic to the new layout while keeping existing command behavior.
- Modify: `web/tests/app.test.js`
  - Extend helper coverage only where the new layout introduces testable frontend helper behavior.
- Modify: `README.md`
  - Record the new command-center desktop direction after implementation.
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`
  - Update the implementation status summary for the redesigned desktop shell.

## Chunk 1: Command-Center Copy Contracts

### Task 1: Lock the new workspace copy with failing tests

**Files:**
- Modify: `web/tests/view-model.test.js`
- Modify: `web/view-model.js`

- [ ] **Step 1: Write the failing test**

Add tests for:
- command-center empty-state hero copy when no tunnel is selected
- selected-tunnel hero copy for idle, connected, and error states
- supporting card copy for forwarding, auth, and health/reconnect details
- timeline summary text for mixed status-event and SSH-output logs

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test web/tests/view-model.test.js`
Expected: FAIL because the new copy contract does not exist yet.

- [ ] **Step 3: Write minimal implementation**

Update `web/view-model.js` to return the new command-center descriptors while preserving current consumers until the layout rewrite is ready.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test web/tests/view-model.test.js`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add web/view-model.js web/tests/view-model.test.js
git commit -m "Add command center view-model copy"
```

## Chunk 2: Main Window Layout Rewrite

### Task 2: Replace the current shell with the command-center structure

**Files:**
- Modify: `web/index.html`
- Modify: `web/styles.css`

- [ ] **Step 1: Update the HTML structure**

Replace the current main shell with:
- compact left control column
- status hero region in the main workspace
- supporting card row
- timeline-style troubleshooting section
- existing drawer and backdrop hooks preserved for editing

- [ ] **Step 2: Add the new visual system**

Implement CSS for:
- light desktop surfaces with dark emphasis zones
- stronger hierarchy for the hero and active list item
- restrained operational color tones
- timeline-style logs
- responsive desktop-first fallback without collapsing the UI into a mobile layout

- [ ] **Step 3: Run static verification**

Run:
- `node --check web/app.js`

Expected: PASS, confirming the HTML/CSS rewrite did not require invalid JS edits yet.

- [ ] **Step 4: Commit**

```bash
git add web/index.html web/styles.css
git commit -m "Build command center shell layout"
```

## Chunk 3: Workspace Wiring And Helper Verification

### Task 3: Reconnect app rendering to the new shell

**Files:**
- Modify: `web/app.js`
- Modify: `web/tests/app.test.js`

- [ ] **Step 1: Write the failing helper test**

Add or extend helper-level tests for any new app-side behavior introduced by the rewrite, such as:
- rendering the new hero error area correctly
- preserving drawer error behavior after the DOM refactor
- keeping snapshot refresh paused while the drawer is open

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test web/tests/app.test.js`
Expected: FAIL because the updated render/helper logic is not wired yet.

- [ ] **Step 3: Write minimal implementation**

Update `web/app.js` to:
- use the new DOM ids and regions
- render the hero, support cards, and timeline sections from the updated view-model helpers
- preserve connect/disconnect/edit/delete behavior
- preserve drawer open/close and save flows

- [ ] **Step 4: Run targeted verification**

Run:
- `node --test web/tests/app.test.js`
- `node --test web/tests/view-model.test.js`
- `node --check web/app.js`

Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add web/app.js web/tests/app.test.js web/tests/view-model.test.js
git commit -m "Wire command center workspace rendering"
```

## Chunk 4: Docs And Full Verification

### Task 4: Update project docs and verify the full stack

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`

- [ ] **Step 1: Update docs**

Record that the desktop frontend now uses the command-center shell and keep the remaining backlog focused on any still-missing integration or visual coverage.

- [ ] **Step 2: Run full verification**

Run:
- `node --test web/tests/view-model.test.js`
- `node --test web/tests/app.test.js`
- `node --check web/app.js`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-core`
- `/home/top/.cargo/bin/cargo check -p sshtunnel-app`

Expected: all commands pass.

- [ ] **Step 3: Commit**

```bash
git add README.md docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md
git commit -m "Document command center desktop redesign"
```

Plan complete and saved to `docs/superpowers/plans/2026-03-30-command-center-ui-modernization.md`. Ready to execute?
