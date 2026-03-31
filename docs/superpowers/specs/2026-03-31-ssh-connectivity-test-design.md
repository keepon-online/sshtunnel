# SSH Connectivity Test Design

**Date:** 2026-03-31

## Goal

Add a dedicated connectivity test action in the editor drawer so a user can validate SSH login and post-login remote target reachability with the current unsaved form values, while also writing the result into the main diagnostic timeline.

## Current Context

- The editor drawer already collects all fields needed for a tunnel definition plus an in-memory password value.
- The current product only supports full tunnel connection, which updates runtime state, tray state, and reconnect behavior.
- Current troubleshooting is hindered by missing one-off diagnostics for "can I log in?" versus "can I reach the target behind SSH?".
- The user explicitly wants the test button to use unsaved drawer values and to write results back into the main timeline for later review.

## Chosen Approach

Implement a dedicated one-shot backend command that is independent from the real tunnel runtime:

- add a `test_tunnel_connectivity` command that accepts unsaved drawer payload
- reuse existing tunnel normalization and auth validation rules where appropriate, but do not persist config or change runtime connection state
- run two sequential checks:
  1. SSH login test against `ssh_host:ssh_port`
  2. remote target reachability test for `remote_host:remote_port` after SSH login succeeds
- return a structured test result to the drawer
- append test summary and raw test output into app-level diagnostic logs that are returned in the normal snapshot payload

## UI Design

- Add a secondary `测试连接` button next to the drawer save action.
- The button uses current form values and does not require a prior save.
- Reuse existing form validation before sending the request.
- Show a lightweight result area inside the drawer:
  - success summary in success styling
  - failure summary in error styling
- Disable the button during execution and show `测试中...`.

## Backend Design

- Introduce a dedicated payload type for drawer-driven testing.
- Normalize and validate the incoming draft tunnel similarly to save flow, but do not mutate stored config or keychain state.
- For password auth:
  - require the current drawer password for test execution
  - do not fall back to stored credentials for this feature
- Execute a one-shot SSH command for login verification using explicit success markers and exit status.
- If login succeeds, execute a second one-shot SSH command to test remote target reachability.
- Capture both summary events and raw command output.

## Logging Design

- Add app-level recent diagnostic logs for connectivity tests.
- Store:
  - concise status events such as "test started", "ssh login ok", "remote target failed"
  - raw output lines from the one-shot test commands
- Merge these logs into the existing snapshot so the main timeline can render them without requiring a live tunnel runtime.
- Prefix test-originated lines with a consistent marker such as `[测试]`.

## Remote Reachability Strategy

- The test command checks reachability after SSH login using a remote shell command.
- Use a compatibility fallback chain:
  1. `nc -z`
  2. `/dev/tcp/host/port`
- If neither mechanism is available remotely, report that the target could not be tested rather than returning a false success.

## Testing Strategy

- Rust:
  - connectivity test command returns SSH failure without running target test
  - connectivity test command returns target failure after SSH login success
  - app-level diagnostic logs are appended and trimmed correctly
  - Windows/password waiting-state regressions continue to pass
- Frontend:
  - drawer shows a `测试连接` action
  - button loading state and result message rendering
  - timeline view-model keeps test summaries and test raw output visible

## Non-Goals

- No change to the real connect/disconnect workflow
- No tray behavior change
- No persistence of test result history to config
- No long-lived background diagnostic task
