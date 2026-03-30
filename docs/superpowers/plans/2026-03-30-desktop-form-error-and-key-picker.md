# Desktop Form Error And Key Picker Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the desktop editor flow by making Tauri bridge access safe, surfacing drawer save errors inline, and adding a private-key file picker.

**Architecture:** Keep the current static frontend and extract a few small helper functions in `web/app.js` so the desktop bridge, editor error rendering, and key-file picking can be tested in isolation. Preserve the existing drawer layout and command names.

**Tech Stack:** Static HTML/CSS/JS frontend, Tauri desktop bridge APIs, Node `node:test`

---

## File Structure

- Modify: `web/app.js`
- Modify: `web/index.html`
- Modify: `web/styles.css`
- Modify: `web/view-model.js`
- Modify: `web/tests/view-model.test.js`
- Create: `web/tests/app.test.js`
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`

## Chunk 1: Failing Frontend Tests

### Task 1: Add failing tests for desktop bridge safety and localized copy

**Files:**
- Modify: `web/view-model.js`
- Modify: `web/tests/view-model.test.js`
- Create: `web/tests/app.test.js`

- [ ] **Step 1: Write the failing test**

Add tests for:
- forwarding summary copy is localized to `本地转发`
- bridge detection does not require `window.__TAURI__` to exist
- desktop-only command attempts return a user-facing error
- key picker writes the selected path into editor state

- [ ] **Step 2: Run test to verify it fails**

Run:
- `node --test web/tests/view-model.test.js`
- `node --test web/tests/app.test.js`

Expected: FAIL because the new helpers and copy changes do not exist yet.

## Chunk 2: Safe Desktop Bridge And Editor Error UI

### Task 2: Implement safe Tauri access and drawer error rendering

**Files:**
- Modify: `web/app.js`
- Modify: `web/index.html`
- Modify: `web/styles.css`

- [ ] **Step 1: Write minimal implementation**

Add safe bridge helpers, an editor error area, desktop-only guards, and submit/file-picker error rendering.

- [ ] **Step 2: Run targeted verification**

Run:
- `node --test web/tests/app.test.js`
- `node --check web/app.js`

Expected: PASS.

## Chunk 3: Key Picker Wiring And Docs

### Task 3: Add file picker wiring and update docs

**Files:**
- Modify: `web/app.js`
- Modify: `web/index.html`
- Modify: `web/styles.css`
- Modify: `web/view-model.js`
- Modify: `web/tests/view-model.test.js`
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`

- [ ] **Step 1: Finish the key picker flow**

Wire a button beside the private-key input to open the Tauri file dialog for private-key auth and keep manual path editing as fallback.

- [ ] **Step 2: Run full verification**

Run:
- `node --test web/tests/view-model.test.js`
- `node --test web/tests/app.test.js`
- `node --check web/app.js`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-app --lib`
- `/home/top/.cargo/bin/cargo test -p sshtunnel-core`
- `/home/top/.cargo/bin/cargo check -p sshtunnel-app`

Expected: all commands pass.
