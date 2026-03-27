# Frontend Status Mapping Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add tested pure frontend mapping helpers for tunnel status badges and connect/disconnect button states.

**Architecture:** Keep all display rules in `web/view-model.js` as pure functions that accept snapshot or tunnel data and return stable UI copy plus button disabled flags. `web/app.js` should only render the returned values into the existing DOM.

**Tech Stack:** Static web frontend, Node `node:test`, existing UMD-style `view-model.js`

---

## File Structure

- Modify: `web/view-model.js`
  - Add pure tunnel status and action-state helpers.
- Modify: `web/tests/view-model.test.js`
  - Add failing tests for status copy and button-state rules.
- Modify: `web/app.js`
  - Replace inline status/action mapping with calls to the new helpers.
- Modify: `README.md`
  - Record the broader frontend test coverage.
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`
  - Update the remaining backlog wording to reflect completed frontend view-model coverage.

## Chunk 1: Status And Action Helpers

### Task 1: Add failing view-model tests

**Files:**
- Modify: `web/tests/view-model.test.js`
- Test: `web/tests/view-model.test.js`

- [ ] **Step 1: Write the failing test**

Add tests for:
- `describeTunnelStatus("idle")` returning localized idle copy
- `describeTunnelStatus("connected")` returning localized connected copy
- `describeTunnelActions(null)` disabling both buttons
- `describeTunnelActions({ status: "error" })` enabling reconnect and disabling disconnect

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test web/tests/view-model.test.js`
Expected: FAIL because the new helpers do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Add the new helpers to `web/view-model.js` and export them.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test web/tests/view-model.test.js`
Expected: PASS.

## Chunk 2: App Wiring And Verification

### Task 2: Consume helpers in the UI

**Files:**
- Modify: `web/app.js`
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`

- [ ] **Step 1: Wire app rendering to the new helpers**

Use the helpers for:
- list badge text
- status card status text
- connect/disconnect button text and disabled state

- [ ] **Step 2: Update docs**

Note the expanded frontend view-model coverage and keep the remaining backlog focused on deeper runtime tests and Windows real-machine validation.

- [ ] **Step 3: Run full verification**

Run:
- `node --test web/tests/view-model.test.js`
- `node --check web/app.js`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-core`
- `/home/top/.cargo/bin/cargo check -p sshtunnel-app`

Expected: all commands pass.
