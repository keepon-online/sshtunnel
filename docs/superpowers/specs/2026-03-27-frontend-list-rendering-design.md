# Frontend List Rendering Design

**Date:** 2026-03-27

## Goal

Extract tunnel list item rendering data into a pure frontend helper so list copy, localized badge text, and active selection state can be tested without introducing DOM-specific test infrastructure.

## Current Context

- `web/app.js` currently builds each tunnel list item inline inside `renderList()`.
- Badge localization already comes from `describeTunnelStatus`, but title, subtitle, forward target text, and active class logic still live in DOM code.
- `web/view-model.js` already hosts pure display helpers and has lightweight `node:test` coverage.

## Chosen Approach

Add a `describeTunnelListItem(tunnel, selectedId)` helper to `web/view-model.js`, then have `renderList()` consume it when building button markup. Keep the HTML structure and click behavior unchanged.

## Responsibilities

- `web/view-model.js`
  - Map a tunnel view object into list-ready display data
  - Reuse `describeTunnelStatus` for localized badge copy
- `web/tests/view-model.test.js`
  - Verify title, subtitle, forward text, badge data, and active state
- `web/app.js`
  - Render list DOM from the helper output only

## Testing Strategy

- Add failing tests before implementation.
- Verify:
  - `title` equals tunnel name
  - `subtitle` equals `username@ssh_host`
  - `forwardText` equals `local_bind_port -> remote_host:remote_port`
  - selected tunnel produces `isActive: true`
  - badge text stays localized via the existing status mapping
- Re-run frontend tests, syntax checks, and existing Rust verification commands.

## Non-Goals

- No DOM test runner setup
- No changes to backend payloads
- No changes to list item HTML structure or click event flow
- No form or status-card rendering changes in this iteration
