# CI Node 24 Actions Design

**Date:** 2026-03-30

## Goal

Remove the GitHub Actions Node 20 deprecation warnings from the packaging workflow without changing the existing Linux `.deb` and Windows `.exe` build behavior.

## Current Context

- `.github/workflows/ci.yml` currently uses:
  - `actions/checkout@v4`
  - `actions/setup-node@v4`
  - `actions/upload-artifact@v4`
- Each job also sets `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true`.
- The latest successful CI run shows deprecation warnings for those actions being forced from Node 20 to Node 24.

## Chosen Approach

Upgrade the affected official actions to their current Node 24-capable major versions and remove the temporary force flag:

- `actions/checkout` -> `v6`
- `actions/setup-node` -> `v6`
- `actions/upload-artifact` -> `v6`
- remove `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24` from all jobs

This keeps the workflow structure, triggers, and bundle commands unchanged.

## Compatibility Notes

- Official action sources indicate these major versions run on Node 24.
- The minimum runner version requirements are satisfied by GitHub-hosted `ubuntu-latest` and `windows-latest`.
- No repository-specific action customization is needed for this workflow.

## Testing Strategy

- Validate the workflow YAML structure locally.
- Push the updated workflow and verify a new CI run completes without the previous Node 20 deprecation annotations.

## Non-Goals

- No changes to build commands
- No changes to bundle artifact names
- No changes to Rust or Tauri versions
