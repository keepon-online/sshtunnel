# Status Summary Cards Design

**Date:** 2026-03-28

## Goal

Refine the right-side workspace summary so connection state is the primary visual signal, while forwarding target and authentication details remain readable at a glance.

## Current Context

- The workspace already uses three summary cards plus a log panel.
- The main status card currently shows only a single state word and does not surface a clear subtitle or inline error summary.
- Forwarding and authentication data exist in the selected tunnel payload and can be displayed without backend changes.

## Chosen Approach

Add a pure view-model helper that returns workspace summary card copy for the selected tunnel. Keep the existing three-card structure, but make the first card visually dominant and feed it richer status information.

## Summary Cards

- Primary status card:
  - label: `Connection`
  - large status word: `已连接 / 空闲 / 错误 / 未选择`
  - subtitle: `username@ssh_host`
  - optional error summary when `last_error` exists
- Forwarding card:
  - clear forwarding string with local bind and remote target
- Authentication card:
  - `密钥认证 / 密码认证`
  - auto-reconnect state as a second line

## Visual Direction

- Primary status card gets stronger padding and tone-specific background
- Error state uses a readable inline summary instead of burying the message in logs
- Secondary cards stay calmer and informational

## Testing Strategy

- Add view-model tests for empty state, connected state, and error state summary copy
- Re-run existing frontend and Rust verification commands after wiring the helper into the workspace renderer

## Non-Goals

- No backend payload changes
- No log panel redesign in this iteration
- No new test framework
