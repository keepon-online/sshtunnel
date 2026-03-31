# Drawer Form Readability Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the drawer form controls consistently readable in installed desktop builds without changing layout or behavior.

**Architecture:** Keep the current static frontend structure and solve the issue entirely in CSS, with lightweight regression assertions in the existing Node-based frontend test file. Target only drawer form controls and supporting text styles.

**Tech Stack:** Static HTML/CSS/JS frontend, Node `node:test`

---

## File Structure

- Modify: `web/styles.css`
- Modify: `web/tests/app.test.js`
- Add: `docs/superpowers/specs/2026-03-31-drawer-form-readability-design.md`
- Add: `docs/superpowers/plans/2026-03-31-drawer-form-readability.md`

## Chunk 1: Readability Regression Test

### Task 1: Add failing style assertions for drawer form readability

**Files:**
- Modify: `web/tests/app.test.js`

- [ ] **Step 1: Write the failing test**

Add assertions that `select` has explicit custom appearance and dark colors, and that disabled controls plus label/placeholder contrast rules are explicitly styled.

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test web/tests/app.test.js`
Expected: FAIL on the new readability assertions.

## Chunk 2: Drawer Form CSS Fix

### Task 2: Implement minimal dark-theme form control cleanup

**Files:**
- Modify: `web/styles.css`

- [ ] **Step 1: Write minimal implementation**

Refine form control styling for `select`, `option`, disabled controls, labels, and placeholders while preserving the current drawer layout.

- [ ] **Step 2: Run verification**

Run:
- `node --test web/tests/app.test.js`
- `node --check web/app.js`

Expected: PASS.
