# Page Visible Copy Localization Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Translate the visible English copy in the desktop page into Chinese without changing internal command or data identifiers.

**Architecture:** Keep the change confined to static page labels in `web/index.html` and display-mapping helpers in `web/view-model.js`. Update frontend tests first so the localized status/meta copy is validated before implementation.

**Tech Stack:** Static HTML/CSS/JS frontend, Node `node:test`

---

## File Structure

- Modify: `web/index.html`
- Modify: `web/view-model.js`
- Modify: `web/tests/view-model.test.js`

## Chunk 1: Failing Localization Tests

### Task 1: Update status/meta expectations

**Files:**
- Modify: `web/tests/view-model.test.js`
- Modify later: `web/view-model.js`

- [ ] **Step 1: Write the failing test**

Update the existing snapshot/autostart expectations so they require Chinese display copy for:
- autostart state
- ssh availability
- summary card labels already produced by the view-model

- [ ] **Step 2: Run test to verify it fails**

Run: `node --test web/tests/view-model.test.js`
Expected: FAIL because the current view-model still returns English display strings.

## Chunk 2: Update Visible Page Copy

### Task 2: Translate view-model and static labels

**Files:**
- Modify: `web/view-model.js`
- Modify: `web/index.html`

- [ ] **Step 1: Write minimal implementation**

Translate visible strings in the view-model and static page labels while keeping internal values and IDs unchanged.

- [ ] **Step 2: Run targeted verification**

Run:
- `node --test web/tests/view-model.test.js`
- `node --check web/app.js`

Expected: PASS.
