# Drawer Form Readability Design

**Date:** 2026-03-31

## Goal

Fix readability issues in the editor drawer form so authentication controls and related fields remain legible in installed desktop builds as well as local development.

## Current Context

- The editor drawer uses a dark glass style.
- Inputs and selects share most of the same styling, but `select` can still fall back to platform-native rendering in some WebView environments.
- The authentication selector has already shown a white-on-white readability failure in installed builds.
- The drawer would benefit from a small consistency pass across labels, controls, placeholders, and disabled states.

## Chosen Approach

Keep the current drawer layout and visual direction, but explicitly style the form controls that can drift under platform defaults:

- force custom dark rendering for `select`
- keep `option` rows legible in dropdown menus
- align label, placeholder, and disabled-state contrast with the existing dark palette
- preserve the existing structure and interactions

## Scope

- `input`, `select`, and `option` readability in the drawer
- `label span`, placeholder text, and disabled states
- no DOM changes
- no main workspace redesign

## Testing Strategy

Use the existing `web/tests/app.test.js` CSS regression style and assert that:

- `select` disables native appearance and keeps explicit dark colors
- disabled controls have explicit readable styling
- label and placeholder contrast rules remain defined

## Non-Goals

- No layout changes
- No animation changes
- No redesign of the main command center
