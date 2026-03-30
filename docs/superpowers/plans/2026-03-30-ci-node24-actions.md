# CI Node 24 Actions Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Update the GitHub Actions workflow to use Node 24-native official actions so CI stops emitting Node 20 deprecation warnings.

**Architecture:** Keep the existing single workflow and packaging jobs intact, changing only the action versions and removing the temporary force flag. The build commands, artifact names, and runner types remain unchanged.

**Tech Stack:** GitHub Actions workflow YAML

---

## File Structure

- Modify: `.github/workflows/ci.yml`
- Create: `docs/superpowers/specs/2026-03-30-ci-node24-actions-design.md`
- Create: `docs/superpowers/plans/2026-03-30-ci-node24-actions.md`

## Chunk 1: Workflow Update

### Task 1: Upgrade official actions to Node 24-native versions

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Update action versions**

Change:
- `actions/checkout@v4` -> `actions/checkout@v6`
- `actions/setup-node@v4` -> `actions/setup-node@v6`
- `actions/upload-artifact@v4` -> `actions/upload-artifact@v6`

Also remove `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24` from the jobs.

- [ ] **Step 2: Validate workflow syntax**

Run:
- `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml')); print('yaml-ok')"`

Expected: `yaml-ok`

## Chunk 2: CI Verification

### Task 2: Verify warnings are gone in GitHub Actions

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Push workflow update**

Push the change to `master`.

- [ ] **Step 2: Inspect the new CI run**

Confirm the packaging workflow still succeeds and that the previous Node 20 deprecation annotations are no longer present.
