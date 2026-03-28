# CI Package Artifacts Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Update GitHub Actions to produce downloadable `.deb` and `.exe` package artifacts and enable Tauri bundling in config.

**Architecture:** Keep the existing fast verification workflow, add a dedicated Ubuntu packaging job for `.deb`, preserve the Windows NSIS packaging job, and enable Tauri bundle output in `tauri.conf.json`.

**Tech Stack:** GitHub Actions, Tauri 2 config, YAML/JSON validation commands

---

## File Structure

- Modify: `.github/workflows/ci.yml`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`

## Chunk 1: Failing Packaging Check

### Task 1: Verify current CI config is missing `.deb` packaging

**Files:**
- Modify later: `.github/workflows/ci.yml`
- Modify later: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Write the failing check**

Run a small assertion that expects:
- `bundle.active` to be `true`
- an `ubuntu-deb` workflow job to exist
- a `.deb` upload path to exist

- [ ] **Step 2: Run check to verify it fails**

Run: `python3 -c "<assert current workflow/config contains deb packaging>"`
Expected: FAIL because bundling is disabled and no Ubuntu packaging job exists yet.

## Chunk 2: Enable Bundling And Upload Artifacts

### Task 2: Update workflow and config

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Write minimal implementation**

Update `tauri.conf.json` to enable bundling and add a new `ubuntu-deb` job that builds and uploads `.deb` artifacts while preserving the Windows `.exe` artifact job.

- [ ] **Step 2: Run targeted verification**

Run:
- `python3 -c "import json; json.load(open('src-tauri/tauri.conf.json')); print('json-ok')"`
- `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml')); print('yaml-ok')"`

Expected: PASS.

## Chunk 3: Docs And Summary

### Task 3: Update docs

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/plans/2026-03-27-ssh-tunnel-manager.md`

- [ ] **Step 1: Update docs**

Record that CI now builds Linux `.deb` and Windows `.exe` artifacts for download-based testing.

- [ ] **Step 2: Summarize artifact locations**

Document the uploaded artifact paths:
- `target/release/bundle/deb/*.deb`
- `target/release/bundle/nsis/*.exe`
