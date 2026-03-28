# Diagnostic Log Panel Design

**Date:** 2026-03-28

## Goal

Make the workspace log panel easier to scan by grouping existing log lines into status events and SSH output, while highlighting errors without changing the backend payload format.

## Current Context

- The backend already provides `recent_log` as a string array.
- The current workspace renders those lines in one flat list, which is harder to scan during failures.
- Several lifecycle messages already have stable text patterns, so the frontend can classify them reliably enough for a first pass.

## Chosen Approach

Add a pure view-model helper that splits the current log lines into:
- `状态事件`
- `SSH 输出`

Each entry should also carry a severity flag so obvious failures render in a stronger style. Keep the backend payload unchanged.

## Classification Rules

Treat these as `状态事件`:
- `spawned ssh process`
- `stopped ssh process`
- `ssh process exited`
- `failed to query process status`
- `missing password credential`
- `password sent`
- `ssh exited with status`

Treat all remaining lines as `SSH 输出`.

Highlight as errors when a line includes:
- `error`
- `failed`
- `missing`
- `denied`
- `refused`
- `exit status`
- `permission`

## Panel Structure

- Header summary showing event counts
- Section 1: `状态事件`
- Section 2: `SSH 输出`
- Each section has its own empty-state copy when no lines exist

## Testing Strategy

- Add view-model tests for:
  - grouped status event classification
  - grouped SSH output classification
  - error highlighting
  - empty-state summaries
- Re-run existing frontend and Rust verification commands after wiring the grouped renderer.

## Non-Goals

- No backend log schema changes
- No timestamps in this iteration
- No log search/filter controls yet
