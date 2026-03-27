# Frontend List Rendering Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add pure, tested tunnel list-item rendering helpers for title, metadata, localized badge text, and active selection state.

**Architecture:** Keep list display rules in `web/view-model.js` and have `web/app.js` consume one helper per list item. Continue using lightweight pure-function tests instead of introducing DOM test infrastructure.

**Tech Stack:** Static web frontend, Node `node:test`, existing UMD-style view-model module

---

## File Structure

- Modify: `web/view-model.js`
  - Add `describeTunnelListItem`.
- Modify: `web/tests/view-model.test.js`
  - Add failing tests for list title, subtitle, forwarding text, localized badge text, and selected state.
- Modify: `web/app.js`
  - Use the new helper inside `renderList()`.
- Modify: `README.md`
  - Update frontend rendering test coverage status.
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`
  - Narrow the remaining backlog after adding list-rendering coverage.

## Chunk 1: List Helper And Tests

### Task 1: Add failing tests for list rendering data

**Files:**
- Modify: `web/tests/view-model.test.js`
- Test: `web/tests/view-model.test.js`

- [ ] **Step 1: Write the failing test**

Add tests for:
- title/subtitle/forward text mapping
- localized badge text for a connected tunnel
- `isActive` true when `selectedId` matches the tunnel id
- `isActive` false when it does not match

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test web/tests/view-model.test.js`
Expected: FAIL because `describeTunnelListItem` does not exist yet.

- [ ] **Step 3: Write minimal implementation**

Add `describeTunnelListItem` to `web/view-model.js` and update `web/app.js` to use it inside `renderList()`.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --test web/tests/view-model.test.js`
Expected: PASS.

## Chunk 2: Docs And Verification

### Task 2: Update docs and run verification

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`

- [ ] **Step 1: Update docs**

Record the expanded frontend rendering coverage and keep the remaining backlog focused on status-card/log rendering or Tauri command flow tests plus Windows validation.

- [ ] **Step 2: Run full verification**

Run:
- `node --test web/tests/view-model.test.js`
- `node --check web/app.js`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-core`
- `/home/top/.cargo/bin/cargo check -p sshtunnel-app`

Expected: all commands pass.
