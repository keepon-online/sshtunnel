# Desktop UI Redesign Design

**Date:** 2026-03-28

## Goal

Redesign the desktop window so it feels like a modern tray-first utility instead of a form-heavy admin page. The main window should emphasize tunnel status, quick actions, and troubleshooting, while tunnel editing moves into a focused drawer.

## Current Context

- The current UI keeps the full tunnel form visible at all times, which makes the main window feel busy.
- Tray actions already cover quick connect/disconnect, so the window should support lower-frequency tasks: inspect, edit, and troubleshoot.
- The backend API and snapshot structure are already stable and should not change for this redesign.

## Chosen Approach

Keep the existing Tauri commands and data model, but reorganize the frontend into a left navigation rail and a right workspace. Use a modal-style side drawer for new/edit flows so the main workspace can remain focused on the selected tunnel.

## Layout

- Left rail:
  - Product title and short description
  - Compact system status card for SSH, autostart, and config path
  - Primary `新建隧道` action
  - Clearer tunnel list with stronger active-state treatment and localized status badges
- Right workspace:
  - Header with tunnel title, destination summary, and primary actions
  - Summary cards for status, forwarding target, and auth mode
  - Dedicated troubleshooting panel with recent logs
  - Empty state when no tunnel is selected

## Editing Flow

- `新建隧道` opens an overlay + drawer with an empty form
- `编辑` opens the same drawer with the current tunnel prefilled
- Save closes the drawer and preserves current selection
- Cancel/overlay click closes the drawer without disturbing current selection

## Visual Direction

- Desktop-tool styling with layered warm-gray surfaces instead of flat white panels
- Distinct but restrained status tones: teal for connected, slate for idle, brick red for error
- Stronger hierarchy through spacing, typography, and panel grouping rather than decorative effects
- Log panel should feel operational and diagnostic without switching to a dark-mode terminal look

## Testing Strategy

- Add view-model tests for new pure copy helpers that support:
  - workspace empty state
  - workspace title/subtitle
  - drawer mode copy for create vs edit
- Reuse existing frontend tests for status/actions/list rendering and keep Rust verification intact

## Non-Goals

- No backend API changes
- No tray behavior rewrite
- No DOM test framework introduction in this iteration
- No mobile-specific redesign beyond maintaining responsive fallback behavior
