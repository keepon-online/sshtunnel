# Editor Required Field Feedback Design

**Date:** 2026-03-30

## Goal

Fix the desktop editor so clicking `保存配置` with missing required fields shows a visible Chinese error inside the drawer instead of appearing to do nothing.

## Current Context

- `web/index.html` marks several fields as `required`.
- `web/app.js` only surfaces errors after `save_tunnel` is invoked.
- In the desktop WebView, native invalid-form feedback is not reliably visible for this flow.

## Root Cause

The submit handler relies on browser-native validation for empty fields, but the actual visible error channel in the editor is only used for async save failures. When native validation UI is not obvious, users get no clear feedback.

## Chosen Approach

Add an explicit frontend validation step before invoking `save_tunnel`:
- validate the prepared payload in `web/app.js`
- return only the first user-facing Chinese error
- render that error through the existing drawer error area `#editor-error`
- stop submission before any Tauri command is sent

Keep the existing HTML `required` attributes as a secondary guard, but do not rely on them for UX.

## Validation Scope

- Required text fields: name, SSH host, username, local bind address, remote host
- Required port fields: SSH port, local bind port, remote port
- Auth-specific requirements:
  - `private_key`: private key path required
  - `password`: password required

## Testing Strategy

- Extend `web/tests/app.test.js` with a failing unit test for the new validation helper
- Verify that an empty payload returns the first expected Chinese error
- Keep the test at helper level so it runs under `node:test` without a browser DOM harness

## Non-Goals

- No per-field inline validation redesign
- No new toast or modal error surface
- No backend validation format changes
