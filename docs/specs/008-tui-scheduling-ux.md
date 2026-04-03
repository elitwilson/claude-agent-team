---
number: 008
status: complete
base_branch: main
---

# Feature: TUI Scheduling UX

## Summary

Three related improvements that make scheduling a first-class citizen in the TUI. First, scheduling a run no longer closes the TUI — after confirming a scheduled time the user lands back on the spec list and can schedule more runs, change their mind, or launch a different spec immediately. Second, the spec list gains a "Run Info" column showing each spec's pending scheduled run (team + datetime) or most recent completed run (team + date). Third, pressing Enter on a spec that already has a scheduled run opens a cancel dialog instead of the normal team/action flow, allowing the user to cancel and optionally reschedule.

---

## Requirements

- After the user confirms a scheduled time in the schedule picker, the TUI returns to the launcher screen without closing
- A status line confirms what was just scheduled (spec name, team, datetime)
- The spec list shows a "Run Info" column; for a scheduled spec it displays the team and scheduled datetime; for a spec with a prior completed run it displays the team and completion date; the column is blank if neither applies
- Scheduled run info takes display priority over last-run info when both are present
- When the user presses Enter on a spec that has a pending scheduled run, a cancel dialog is shown instead of the normal team/action popup
- The cancel dialog shows the spec name, scheduled team, and scheduled datetime, and offers a single "Cancel Scheduled Run" action; Esc dismisses without action
- Confirming the cancel dialog removes the scheduled run and immediately clears the spec's run info column
- After cancellation, pressing Enter on that spec opens the normal team/action flow
- The Execute Now flow is unchanged
- The Drafter (Requirements tab) flow is unchanged
- The schedule picker's Esc behavior is unchanged (returns to action popup)

---

## Scope

### In Scope

- `src/tui/app.rs` — new run info state, cancel dialog popup variant, updated picker confirmation logic
- `src/tui/ui.rs` — three-column spec table rendering, cancel dialog popup rendering, scheduling side effects, status message display
- `src/metrics/db.rs` — new query for last completed run per spec for the current project
- `src/main.rs` — remove the post-TUI scheduled-run branch; pass `cwd` into `run_tui()`

### Out of Scope

- Editing a scheduled run's time in place (workflow: cancel then reschedule)
- Multiple simultaneous scheduled runs for the same spec (one plist per slug, unchanged)
- Showing runs scheduled by other projects
- Schedule picker UI changes

---

## Technical Approach

- **`run_tui()` signature:** Add `cwd: &Path` parameter. Before starting the event loop, load initial run info:
  1. Call `scheduler::list_pending()` to get pending plists.
  2. Open the metrics DB at `$HOME/.claude/claude-agent-team-metrics.db` (same path as `main.rs`). If the file does not exist, skip the DB load and treat all last-run info as empty — do not fail.
  3. Derive the project string as `cwd.to_str().unwrap_or("").replace('/', "-")` (same formula used in `main.rs`).
  4. Call `metrics::db::last_run_for_project(conn, &project)` to get last-run data.
  5. Merge both into the initial `run_info` map (scheduled entries take priority) and pass into `App::new()`.

- **`LastRun` struct (new, in `metrics/db.rs`):**
  ```rust
  pub struct LastRun {
      pub team: String,
      pub completed_at: DateTime<Utc>,
  }
  ```
  New function: `last_run_for_project(conn, project: &str) -> Result<HashMap<String, LastRun>>` — selects the most recent `completed_at` + `team` per `feature_slug` for the given project.

- **`SpecRunInfo` enum (new, in `tui/app.rs`):**
  ```rust
  pub enum SpecRunInfo {
      Scheduled { team: String, at: DateTime<Local>, plist_path: PathBuf },
      LastRun { team: String, completed_at: DateTime<Utc> },
  }
  ```
  `plist_path` is carried on `Scheduled` so the cancel path can call `cleanup_plist()` directly from `run_info` without a separate map.

- **`App` state changes:** Add `run_info: HashMap<String, SpecRunInfo>` (keyed by feature slug — this is the single canonical map for both display and cancellation). Add `cwd: PathBuf` (needed for `schedule_run` call). Add `status_message: Option<String>` for post-schedule confirmation display. Remove `scheduled_at: Option<DateTime<Local>>` (scheduling no longer exits the TUI). No separate `pending_runs` field is needed.

- **`TuiResult`:** Remove `scheduled_at` field. Scheduled runs no longer produce a `TuiResult`.

- **`App::confirm_picker()`:** Instead of setting `confirmed = true`, call `scheduler::schedule_run()` using `self.cwd`. On success: insert into `run_info`, set `status_message`, reset `screen` to `Screen::Launcher`. On failure: set `picker.error` (same path as validation errors today).

- **`PopupAction::CancelDialog`:** New variant: `CancelDialog { spec_slug: String, team: String, at: DateTime<Local> }`. `open_team_popup()` checks `run_info` before opening — if the spec has a `Scheduled` entry, open `CancelDialog` instead of `TeamDialog`.

- **Cancel confirm:** New `App::confirm_cancel_dialog()` method — reads `plist_path` from `run_info[slug]`, calls `scheduler::cleanup_plist()`, removes the entry from `run_info`, dismisses popup.

- **Spec table columns:** Three columns in the spec list: Spec Name (stretches to fill), Status (~10 chars fixed), Run Info (~28 chars fixed). Scheduled entries use normal styling; last-run entries use dim styling. Both fixed columns truncate if terminal is too narrow.

- **`main.rs`:** Remove the `if let Some(scheduled_at) = selection.scheduled_at` branch and its `println!`. Pass `&cwd` to `run_tui()`.

---

## Success Criteria

- [ ] After scheduling, the TUI remains open on the launcher screen with a status message identifying the spec, team, and scheduled time
- [ ] The spec list Run Info column shows `team @ Mon Jan 2 8:00pm` for scheduled specs
- [ ] The spec list Run Info column shows `team · Jan 2` (dim) for specs with a prior completed run and no pending schedule
- [ ] Pressing Enter on a scheduled spec opens a cancel dialog showing the spec, team, and time
- [ ] Confirming the cancel dialog removes the plist, clears the Run Info column for that spec, and returns to the spec list
- [ ] After cancellation, pressing Enter on that spec opens the normal team/action popup
- [ ] Execute Now flow produces the same `TuiResult` as before
- [ ] `main.rs` no longer contains any `scheduled_at`-related branch
- [ ] All existing `app.rs` and `ui.rs` tests pass or are updated to reflect removed `scheduled_at` field

---

## Tasks

- [ ] **Add `last_run_for_project()` to `metrics/db.rs`:** Add `LastRun` struct and a query that returns the most recent completed run per feature slug for the given project. Unit test against an in-memory SQLite DB.

- [ ] **Update `App` state in `app.rs`:** Add `SpecRunInfo` enum, `pending_runs: HashMap<String, ScheduledRun>`, `run_info: HashMap<String, SpecRunInfo>`, `cwd: PathBuf`, `status_message: Option<String>`. Remove `scheduled_at`. Update `App::new()` signature to accept pending runs and last-run data. Depends on Task 1.

- [ ] **Implement scheduling and cancel logic in `app.rs`:** Update `confirm_picker()` to call `schedule_run()` instead of setting `confirmed`. Add `PopupAction::CancelDialog` variant. Update `open_team_popup()` to open `CancelDialog` when a pending scheduled run exists. Add `confirm_cancel_dialog()`. Update `dismiss_popup()` for the new variant. Remove `TuiResult::scheduled_at`. Update all tests. Depends on Task 2.

- [ ] **Update `ui.rs` layout and rendering:** Update `run_tui()` signature to accept `cwd`, load initial data, construct `App` with new params. Render three-column spec table. Render `CancelDialog` popup. Render `status_message` in footer or status bar. Update event handler for `CancelDialog` keypresses. Update `main.rs` to pass `&cwd` and remove the scheduling branch. Depends on Task 3.

- [ ] **App state-machine smoke tests:** In `app/tests.rs`, add tests that drive `App` through the new flows: (a) `confirm_picker()` with a mock `ScheduledRun` result updates `run_info`, sets `status_message`, and leaves `screen == Screen::Launcher` with `confirmed == false`; (b) `open_team_popup()` on a spec with a `SpecRunInfo::Scheduled` entry opens `CancelDialog` instead of `TeamDialog`; (c) `confirm_cancel_dialog()` removes the entry from `run_info`. These do not touch the filesystem or launchd. Depends on Task 3.

---

## Considerations

- `scheduler::list_pending()` scans `~/Library/LaunchAgents` for plists with the `com.claude-agent-team` prefix. At TUI startup this should be fast, but the result is only loaded once — the in-memory map is the source of truth after that. This is fine since the TUI is the only writer during a session.
- The plist parser (`parse_plist`) does not store the year (launchd `StartCalendarInterval` has no year key). The existing code sets year to `Local::now().year()`. When displaying scheduled times and comparing against pending info, be aware that the parsed `scheduled_at` may have the wrong year for runs scheduled near year-end. This is a pre-existing limitation; don't fix it here.
- `cleanup_plist()` calls `launchctl unload` which is fatal if it fails (per its current contract). This is appropriate for cancel too — if unloading fails, the run is still registered with launchd and we should not remove it from the UI.
- The `status_message` should be cleared on the next keypress after it is set. Any key event that is processed by the event loop should clear it first, before handling the key action. This is simpler and more predictable than frame-counting.
- `run_tui()` currently returns `Option<TuiResult>`. After this change, `None` still means "user quit without selecting." The return type is unchanged.
