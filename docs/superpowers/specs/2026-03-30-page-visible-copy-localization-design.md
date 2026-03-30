# Page Visible Copy Localization Design

**Date:** 2026-03-30

## Goal

Translate the visible English copy in the desktop page into Chinese while keeping command IDs, data fields, and internal APIs unchanged.

## Current Context

- The main page is mostly Chinese, but several visible labels still remain in English.
- The remaining English appears in static HTML labels and view-model generated status/meta copy.
- The request is limited to page-visible copy, not product naming across packaging, tray menus, or repository docs.

## Chosen Approach

Update only the visible page strings rendered in:
- `web/index.html`
- `web/view-model.js`

Keep internal identifiers such as command names, object keys, auth values, and tunnel model fields unchanged.

## Translation Scope

Translate these visible strings:
- `Tray-first SSH Tunnels`
- `System SSH`
- `Autostart`
- `Config`
- `Tunnels`
- `Workspace`
- `Connection`
- `Authentication`
- `Troubleshooting`
- `Editor`
- `enabled`
- `disabled`
- `available`
- `missing`

## Testing Strategy

- Update `web/tests/view-model.test.js` expectations for localized snapshot/status copy
- Run existing frontend tests and syntax checks after the copy change

## Non-Goals

- No tray menu localization in this iteration
- No app/product renaming in the window title or package metadata
- No backend data format changes
