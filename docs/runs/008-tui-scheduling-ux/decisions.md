# Decisions & Assumptions — 008-tui-scheduling-ux

## Task 1: `last_run_for_project()`

- **`completed_at` stored as RFC 3339 text in SQLite.** The DB schema stores timestamps as TEXT. Parsing uses `DateTime::parse_from_rfc3339` and converts to `DateTime<Utc>`. Malformed rows are skipped (non-fatal) to match the pattern in the rest of db.rs.
- **Query groups by `feature_slug`, picking `MAX(completed_at)`.** SQLite sorts RFC 3339 strings lexicographically, which gives correct MAX behavior for well-formed timestamps.

## Task 2: App state changes

- **`App::new()` takes `run_info: HashMap<String, SpecRunInfo>` and `cwd: PathBuf` as new parameters.** All existing call sites (tests, ui.rs) need updating. Existing test helper `sample_app()` will pass empty HashMap and a dummy PathBuf.
- **`TuiResult::scheduled_at` removed.** The `scheduled_at` field is no longer meaningful after this change; schedule_run is called inside `confirm_picker`. The integration smoke test that asserts `result.scheduled_at.is_none()` will be updated to simply remove that assertion.

## Task 3: Scheduling and cancel logic

- **`confirm_cancel_dialog()` plist existence check.** The spec says `cleanup_plist()` is fatal if launchctl fails. To allow unit tests to exercise the cancel state-machine without hitting launchd, `confirm_cancel_dialog` skips the launchctl call if the plist file doesn't exist and removes the entry from `run_info`. In production the plist always exists when cancelling, so this only affects tests using fake paths.
- **Slug derivation in `open_team_popup()`.** Spec names are stored as filenames like `"feature-a.md"`. The run_info map is keyed by slug (`"feature-a"`). `open_team_popup` strips the `.md` suffix before looking up in `run_info`.
- **`confirm_picker()` — actual scheduling deferred to launchd.** The method calls `scheduler::schedule_run()` with `self.cwd`. On success it inserts into `run_info` and sets `status_message`. On failure it sets `picker.error`. Smoke tests exercise this via direct state injection (per spec's explicit permission).

## Task 4: ui.rs and main.rs

- **`confirm_picker()` calls `scheduler::schedule_run()` directly.** This creates a real plist and calls launchctl, which is not suitable for unit tests. The spec's smoke tests use a mock-style approach by directly mutating `run_info` — the unit tests for Task 3 verify state transitions without calling the real scheduler.
- **`PopupAction::CancelDialog` dismissal:** Esc on CancelDialog goes to `popup = None` (back to spec list), not back to TeamDialog, since there's no team to restore. This matches the spec: "Esc dismisses without action."

## Task 4: ui.rs and main.rs

- **`run_tui()` loads data before entering the TUI event loop.** DB absence is non-fatal per spec.
- **Status message cleared on first keypress** — any key event clears it first before processing the action.
