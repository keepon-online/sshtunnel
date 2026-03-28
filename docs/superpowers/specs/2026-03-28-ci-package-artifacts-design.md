# CI Package Artifacts Design

**Date:** 2026-03-28

## Goal

Make GitHub Actions build downloadable Linux `.deb` and Windows `.exe` package artifacts so the desktop app can be tested without local packaging.

## Current Context

- The current workflow already uploads a Windows NSIS `.exe` artifact.
- The Ubuntu workflow only performs checks and does not build bundle artifacts.
- Tauri packaging is currently disabled via `"bundle.active": false`, so Linux bundle output cannot be produced yet.

## Chosen Approach

Keep the current fast Ubuntu check job, keep the current Windows installer job, and add a dedicated Ubuntu packaging job for `.deb` artifacts.

Update Tauri config so bundling is active in CI and local builds.

## Workflow Shape

- `ubuntu-check`
  - keep as fast verification
- `ubuntu-deb`
  - install Linux system packaging dependencies
  - run Tauri bundle build for `deb`
  - upload `target/release/bundle/deb/*.deb`
- `windows-installer`
  - keep NSIS bundle build
  - upload `target/release/bundle/nsis/*.exe`

## Packaging Config

- Change `src-tauri/tauri.conf.json`:
  - set `"bundle.active": true`
- Keep icon and bundle target defaults unchanged beyond enabling packaging

## Testing Strategy

- First run a small config/workflow assertion that fails because `.deb` packaging is not configured yet
- Then update config and workflow
- Verify:
  - JSON parses
  - YAML parses
  - workflow includes the new Ubuntu packaging job
  - bundle config is active

## Non-Goals

- No GitHub Release publishing
- No signed installers
- No manual-dispatch-only workflow split
