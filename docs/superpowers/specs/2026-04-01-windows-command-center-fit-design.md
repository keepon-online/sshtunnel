# Windows Command Center Fit Design

**Date:** 2026-04-01

## Goal

Improve the desktop UI fit on Windows so the main command center remains fully reachable above the taskbar, while keeping the current SSH tunnel workflow and command structure unchanged.

## Current Context

- The main Tauri window currently opens at `1220x820` with a relatively tall minimum height.
- The root page shell uses a fixed `100vh` layout with `overflow: hidden`.
- The command center and timeline are optimized for a full-height viewport, which makes the layout fragile on Windows when taskbar height or display scaling reduces the effective work area.
- The diagnostic timeline already supports internal scrolling, but it still competes with the full-page height lock.

## Chosen Approach

Use a mixed desktop adaptation instead of a full redesign:

- reduce the default window height pressure and center the main window at launch
- let the overall page scroll vertically when the viewport is shorter than expected
- keep the diagnostic log lanes internally scrollable with a bounded, stable height
- preserve existing information hierarchy, actions, and backend behavior

## Scope

- Tauri main window sizing and placement
- root shell and workspace height behavior
- timeline/log panel sizing and scrolling behavior
- frontend regression tests for CSS, config, and log auto-follow behavior

## Testing Strategy

Use the existing frontend Node tests plus Rust library tests:

- assert the Tauri config keeps the main window centered and uses less aggressive height constraints
- assert the shell layout no longer hard-locks to `100vh` with hidden overflow
- assert log panels use bounded height rules
- assert log auto-follow only snaps to the latest entry when the user was already near the bottom
- run the existing Rust library test suite to confirm no runtime behavior regresses

## Non-Goals

- No changes to SSH launch, connection, reconnection, or password handling
- No changes to the card structure, actions, or data model
- No redesign of the left tunnel list or editor drawer interaction model
