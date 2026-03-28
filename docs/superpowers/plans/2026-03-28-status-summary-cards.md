# Status Summary Cards Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Improve the workspace summary cards so the main connection state is clearer and key tunnel diagnostics are visible without opening logs.

**Architecture:** Add a pure `describeStatusSummaryCards` helper in `web/view-model.js`, test it with `node:test`, then have `web/app.js` render the primary status card and supporting summary cards from that helper. Update CSS to emphasize the primary card and inline error summary.

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

## Chunk 1: Status Card Copy Helper

### Task 1: Add failing tests and helper

**Files:**
- Modify: `web/tests/view-model.test.js`
- Modify: `web/view-model.js`

- [ ] **Step 1: Write the failing test**

Add tests for:
- empty-state status card copy
- connected tunnel status summary
- error tunnel status summary including inline error text and auth/auto-reconnect details

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test web/tests/view-model.test.js`
Expected: FAIL because `describeStatusSummaryCards` does not exist yet.

- [ ] **Step 3: Write minimal implementation**

Add `describeStatusSummaryCards` and export it.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test web/tests/view-model.test.js`
Expected: PASS.

## Chunk 2: Workspace Wiring

### Task 2: Render the stronger summary cards

**Files:**
- Modify: `web/index.html`
- Modify: `web/styles.css`
- Modify: `web/app.js`

- [ ] **Step 1: Update card markup**

Add the minimal elements needed for:
- primary status subtitle
- inline error summary
- auth card secondary line

- [ ] **Step 2: Wire the app**

Populate the cards from `describeStatusSummaryCards`.

- [ ] **Step 3: Tune styles**

Make the primary card visually dominant and improve state/error contrast.

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

Record the stronger workspace status overview and keep the remaining backlog focused on log-panel rendering and Tauri command-flow coverage.

- [ ] **Step 2: Run full verification**

Run:
- `node --test web/tests/view-model.test.js`
- `node --check web/app.js`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-core`
- `/home/top/.cargo/bin/cargo check -p sshtunnel-app`

Expected: all commands pass.
