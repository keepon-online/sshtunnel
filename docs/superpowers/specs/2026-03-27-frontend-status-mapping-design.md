# Frontend Tunnel Status Mapping Design

**Date:** 2026-03-27

## Goal

Extract tunnel status and action-label mapping into pure frontend view-model helpers so the list badge text and connect/disconnect button state can be verified without introducing DOM-heavy tests.

## Current Context

- `web/app.js` currently renders the tunnel list badge text directly from backend status values such as `idle`, `connected`, and `error`.
- The connect and disconnect buttons exist in the main form header, but their enabled state is not derived from a tested helper.
- `web/view-model.js` already hosts snapshot-level mapping helpers and is covered by lightweight `node:test` tests.

## Chosen Approach

Add pure mapping helpers to `web/view-model.js` for tunnel status copy and button-state rules, then make `web/app.js` consume those helpers during render. Keep the backend payload unchanged and avoid introducing DOM test infrastructure in this iteration.

## Responsibilities

- `web/view-model.js`
  - Map backend tunnel statuses to stable UI copy
  - Decide connect/disconnect button labels and disabled state from the selected tunnel
- `web/tests/view-model.test.js`
  - Cover `idle`, `connected`, `error`, and no-selection cases
- `web/app.js`
  - Use the view-model helpers for list badges, status card copy, and button state

## Error Handling

- Unknown statuses fall back to the raw backend value so the UI remains readable if new backend statuses appear.
- No selected tunnel keeps both action buttons disabled.
- `error` state allows reconnect so users can retry without editing the tunnel.

## Testing Strategy

- Add failing unit tests for the new pure helpers before implementation.
- Verify the tests fail because the helpers are missing.
- Implement the minimal mapping helpers and render wiring.
- Re-run frontend tests, syntax checks, and the existing Rust verification suite.

## Non-Goals

- No DOM test runner setup
- No form-state testing in this iteration
- No backend API changes
- No redesign of button layout or visual styling
