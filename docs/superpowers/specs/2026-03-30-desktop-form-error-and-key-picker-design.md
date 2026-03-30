# Desktop Form Error And Key Picker Design

**Date:** 2026-03-30

## Goal

Fix the desktop editor flow so the frontend no longer crashes when Tauri globals are unavailable, saving failures are shown inside the drawer, and private-key authentication can pick a file path from the system file dialog.

## Current Context

- `web/app.js` currently assumes `window.__TAURI__.core.invoke` exists at load time.
- Opening the web page in a normal browser causes an immediate runtime crash before the UI can render.
- The editor submit flow does not catch save errors, so failures are not surfaced inside the drawer.
- Private-key auth only supports manual path entry.

## Root Causes

- The Tauri bridge is dereferenced eagerly at module top level.
- Submit and action handlers call `invoke(...)` directly without a user-visible error channel in the editor.
- There is no frontend hook for Tauri file dialog selection.

## Chosen Approach

Keep the app desktop-first, but make the bridge access defensive:
- detect whether Tauri invoke and dialog APIs are available
- if not available, show a clear desktop-only message instead of crashing

Add a dedicated editor error area inside the drawer:
- save failures stay in the drawer
- file picker failures also render there
- successful save clears the error and closes the drawer

Add a private-key picker button beside the path input:
- available only for `private_key` auth
- opens the native file dialog via Tauri plugin dialog API
- writes the selected path back into the existing text input
- keeps manual typing as fallback

## Frontend Structure

Add a few small helpers in `web/app.js`:
- safe Tauri bridge detection
- editor error rendering/clearing
- key file picking action
- desktop-only guard for commands that require Tauri

Keep the existing page structure and view-model module. Do not introduce a larger frontend framework or unrelated refactor.

## Copy Changes

- Localize `Forwarding` to `本地转发`
- Show desktop-only fallback text when the page is opened outside the desktop shell
- Show save/file-picker failures inline in the editor

## Testing Strategy

- Extend `web/tests/view-model.test.js` for localized forwarding copy
- Add a new lightweight `web/tests/app.test.js` that exercises pure/isolated helpers for:
  - safe bridge detection
  - desktop-only save/file-picker error behavior
  - private-key path fill behavior
- Keep tests on `node:test` without adding a browser DOM dependency

## Non-Goals

- No browser mode with real persistence or tunnel actions
- No drag-and-drop key file support
- No field-by-field validation redesign in this iteration
