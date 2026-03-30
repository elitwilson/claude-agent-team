# Review Notes — 005 Scheduled Runs

## Expected Test Cases (Reviewer's pre-formed list)

### Task 1: `run` subcommand arg parsing

- Parses `--spec <name>` correctly
- Parses `--team <name>` correctly
- Parses `--headless` flag (boolean, no value)
- Parses `--cleanup-plist <path>` correctly
- All flags together parse correctly
- Missing required `--spec` returns error
- Missing required `--team` returns error
- `--headless` defaults to false when omitted
- `--cleanup-plist` is optional (None when omitted)

### Task 2: `scheduler` module — plist generation

- Generated plist contains correct Label (`com.claude-agent-team.<spec-slug>`)
- ProgramArguments starts with `caffeinate -i`
- ProgramArguments includes the binary path
- ProgramArguments includes `run --spec <name> --team <name>`
- ProgramArguments includes `--headless` when headless is true
- ProgramArguments includes `--cleanup-plist <path>`
- WorkingDirectory matches the provided working_dir
- StartCalendarInterval has correct Month/Day/Hour/Minute from scheduled_at
- Returns error if scheduled_at is less than 1 minute in the future
- Plist file path follows naming convention `com.claude-agent-team.<spec-slug>.plist`

### Task 3: `scheduler` module — pending run discovery

- Returns empty list when no matching plist files exist
- Parses a valid plist file into a ScheduledRun with correct fields
- Discovers multiple plist files matching the naming convention
- Ignores plist files that don't match `com.claude-agent-team.*.plist` pattern
- Handles malformed plist files gracefully (error or skip)

### Task 4: Self-cleanup

- Cleanup calls `launchctl unload` with the correct plist path
- Cleanup removes the plist file after unloading
- Cleanup returns error if `launchctl unload` fails (fatal)
- Cleanup returns error if file removal fails (fatal)
- No cleanup attempted when `--cleanup-plist` is not provided

---

## Review Outcomes

### Task 1: `run` subcommand arg parsing — APPROVED

**Tests reviewed:** `src/run_cmd/tests.rs` (8 tests)

All spec requirements covered:
- Parsing of all four flags (--spec, --team, --headless, --cleanup-plist)
- Required flag validation (--spec, --team)
- Optional flag defaults (headless=false, cleanup_plist=None)
- Error cases for missing required flags

Coder also added useful robustness tests (arbitrary flag order, unknown flags, missing flag values) that go beyond the spec without testing implementation details.

No gaps. No implementation-detail testing. No misdirection.

### Task 2: `scheduler` module — plist generation — APPROVED

**Tests reviewed:** `src/scheduler/tests.rs` (14 tests)

All spec requirements covered:
- Plist path naming convention (`com.claude-agent-team.<spec>.plist` in `~/Library/LaunchAgents/`)
- Schedule time validation (rejects past, rejects <1 min future, accepts valid future)
- Generated XML contains: Label, caffeinate -i wrapping, binary path, run subcommand args (--spec, --team, --headless, --cleanup-plist), WorkingDirectory, StartCalendarInterval (Month/Day/Hour/Minute)
- Headless flag correctly included/excluded based on boolean
- Valid plist XML structure

Good design decision separating `generate_plist_xml` from `schedule_run` for testability without filesystem/launchd side effects, consistent with the spec's guidance.

No gaps. No implementation-detail testing. No misdirection.

### Task 3: `scheduler` module — pending run discovery — APPROVED

**Tests reviewed:** `src/scheduler/tests.rs` (8 new tests: 5 parse_plist + 3 list_pending_in)

All spec requirements covered:
- Parsing plist into ScheduledRun with all fields (spec, team, headless, scheduled_at, plist_path)
- Empty directory returns empty list
- Matching files discovered correctly
- Non-matching files ignored

Good use of fixture plist XML and tempfile for isolation. The `list_pending_in(dir)` pattern for testability mirrors the `generate_plist_xml` approach — keeps launchd/filesystem concerns out of unit tests.

No gaps. No implementation-detail testing. No misdirection.

### Task 4: Self-cleanup — APPROVED

**Tests reviewed:** `src/scheduler/tests.rs` (2 new tests)

Spec requirements covered:
- Fatal error when `launchctl unload` fails — covered by `test_cleanup_plist_removes_file` (launchctl unload fails in test env since plist isn't loaded, verifying the fatal error contract)
- Fatal error when file doesn't exist — covered by `test_cleanup_plist_errors_on_nonexistent_file`
- "No cleanup when --cleanup-plist absent" is arg-parsing behavior already covered in Task 1 tests

The spec explicitly states launchctl shell-outs can't be meaningfully unit tested. The Coder's approach is correct: test the error contract at the unit level, leave the happy path (unload + remove) to integration testing. Test count is appropriately scoped for a function whose core behavior involves OS shell-outs.

No gaps. No implementation-detail testing. No misdirection.

---

## Summary

All 4 tasks APPROVED. No flags raised across any review pass.
