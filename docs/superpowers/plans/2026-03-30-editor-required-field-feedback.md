# Editor Required Field Feedback Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the editor show the first Chinese validation error inside the drawer when users try to save an incomplete tunnel configuration.

**Architecture:** Keep validation in the existing static frontend by adding a small pure helper in `web/app.js` that inspects the prepared payload before `save_tunnel` runs. Reuse the existing `#editor-error` area so the UX stays consistent with other editor failures.

**Tech Stack:** Static HTML/JS frontend, Tauri desktop bridge, Node `node:test`

---

## File Structure

- Modify: `web/app.js`
- Modify: `web/tests/app.test.js`
- Create: `docs/superpowers/specs/2026-03-30-editor-required-field-feedback-design.md`
- Create: `docs/superpowers/plans/2026-03-30-editor-required-field-feedback.md`

## Chunk 1: Validation Test

### Task 1: Add the failing validation test

**Files:**
- Modify: `web/tests/app.test.js`

- [ ] **Step 1: Write the failing test**

Add a test that builds an empty tunnel payload and expects the first visible Chinese validation message, `请输入名称。`.

- [ ] **Step 2: Run test to verify it fails**

Run:
- `node --test web/tests/app.test.js`

Expected: FAIL because the validation helper is not implemented/exported yet.

## Chunk 2: Submit Guard

### Task 2: Implement minimal frontend validation

**Files:**
- Modify: `web/app.js`
- Modify: `web/tests/app.test.js`

- [ ] **Step 1: Write minimal implementation**

Add a pure `validateTunnelPayload(...)` helper, call it from the submit handler before `save_tunnel`, and route the returned message into `setEditorError(...)`.

- [ ] **Step 2: Run targeted verification**

Run:
- `node --test web/tests/app.test.js`
- `node --check web/app.js`

Expected: PASS.
