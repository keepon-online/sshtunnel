# Command Center UI Modernization Design

**Date:** 2026-03-30

## Goal

Modernize the desktop window so it feels like a polished, professional tunnel control surface instead of a generic form-based admin page. The main window should make current tunnel health the first visual signal, while preserving the existing tray-first product model.

## Current Context

- The frontend already has a tray-first workflow, a tunnel list, a right-side workspace, grouped logs, and a drawer-based editor.
- The current interface is functionally complete, but its visual hierarchy is still too flat and conventional.
- The existing backend snapshot, Tauri commands, and editor save flow are already stable and should stay intact.
- The product direction is still tray-first: common quick actions remain available from the tray, while the main window is for inspection, troubleshooting, and configuration changes.

## Product Constraints

- Keep the tray-first behavior and tone.
- Keep the current backend data model and command interface.
- Keep the editor as a drawer-based flow rather than returning to an always-visible form.
- Allow a substantial layout rewrite in the main window if it improves clarity.
- Prefer a light overall interface with dark, focused areas that add hierarchy without turning the app into a dark console.

## Chosen Approach

Adopt a "Command Center" layout:

- a restrained left control column for tunnel switching and compact global status
- a dominant status hero area in the main workspace that immediately communicates the selected tunnel's health
- a second row of smaller supporting cards for forwarding, authentication, and reliability indicators
- a wider, timeline-like troubleshooting area below that makes recent events easy to scan

This keeps the app recognizable as a desktop tool while giving the selected tunnel a much stronger sense of presence and operational state.

## Layout

### Left Control Column

- Compact product header with short descriptive copy
- Small global status module for:
  - system SSH availability
  - autostart state
  - config path
- Primary `新建隧道` action
- Tunnel list redesigned as operational objects rather than plain rows:
  - stronger active-state treatment
  - localized status badges
  - forwarding summary visible at list level

### Main Command Center

- Top hero section for the selected tunnel
  - tunnel name
  - user and SSH host
  - high-visibility current state
  - main actions: connect or disconnect, edit
  - delete remains available but visually de-emphasized
- Supporting card row below the hero
  - forwarding destination
  - authentication mode
  - health or reconnect signal
- Bottom troubleshooting section
  - recent status events first
  - SSH output second
  - timeline-like presentation instead of generic stacked text boxes

### Empty State

- The main window still uses the same overall command-center structure when no tunnel is selected.
- The hero becomes a guided empty state rather than collapsing into generic placeholder blocks.
- Supporting cards explain what will appear after selection instead of rendering as blank UI.

## Interaction Model

- Selecting a tunnel updates the hero, support cards, and troubleshooting area immediately.
- The most important action for the current state appears first:
  - idle tunnel: connect
  - connected tunnel: disconnect
  - error tunnel: reconnect path remains prominent
- Edit continues to open the existing drawer with prefilled values.
- New tunnel continues to open the same drawer in create mode.
- Drawer open/close behavior stays the same so the redesign does not create backend or workflow regressions.

## Visual Direction

- Light desktop surface as the baseline
- Deep slate-blue or blue-green used for emphasis zones such as the hero or key status surfaces
- Clean, tool-like typography with stronger spacing and grouping
- Status colors stay restrained:
  - connected: cool teal/green
  - idle: slate
  - error: alert tone with limited, targeted usage
- Use stronger structural hierarchy rather than decorative noise:
  - larger corner radii
  - more intentional spacing
  - bolder grouping of related information
- Avoid turning the app into a monitoring dashboard or terminal simulator

## Testing Strategy

- Extend pure view-model coverage for the new command-center copy and hero/card labels.
- Keep `web/tests/app.test.js` focused on frontend helper behavior that is still testable without introducing a DOM framework.
- Re-run existing Rust backend tests to prove the redesign does not affect command behavior.
- Preserve responsive fallback behavior for narrower widths, but desktop remains the primary design target.

## Non-Goals

- No backend API changes
- No tray menu workflow rewrite
- No new frontend framework adoption
- No DOM test framework introduction in this iteration
- No redesign of the drawer form schema itself beyond visual integration with the new shell
