# Review Notes — 008-tui-scheduling-ux

## Task 1: last_run_for_project

**Verdict: Approved**

### Requirements Checklist (derived from spec)

1. Returns empty map when no runs exist for the project — covered by `test_last_run_for_project_returns_empty_when_no_runs`
2. Returns the most recent run per slug (MAX completed_at wins) — covered by `test_last_run_for_project_returns_most_recent_per_slug`, which also asserts both `team` and `completed_at` on the winning row
3. Returns one entry per slug; multiple distinct slugs each get their own entry — covered by `test_last_run_for_project_returns_one_entry_per_slug`
4. Filters by project — runs belonging to a different project string are excluded — covered by `test_last_run_for_project_filters_by_project`
5. Malformed `completed_at` is handled non-fatally (no panic) — covered by `test_last_run_for_project_returns_err_on_bad_timestamp`

### Notes

All spec requirements have corresponding test coverage. The bad-timestamp test is intentionally permissive (asserts only no-panic, not a specific outcome), which is consistent with the decisions doc stating "malformed rows are skipped (non-fatal)." This is acceptable — the test correctly leaves the exact behavior open since either skip or Err are declared valid outcomes.

No gaps, no misdirection. Tests target observable behavior (query results, struct field values) rather than implementation details.

## Task 3: Scheduling and cancel logic

**Verdict: Approved**

### Requirements Checklist (derived from spec)

1. `confirm_picker()` on success: inserts into `run_info`, sets `status_message`, resets `screen` to `Screen::Launcher` — covered by `test_confirm_picker_on_success_returns_to_launcher_and_sets_status_message` (direct state injection per spec's smoke test note; assertions verify `screen`, `status_message`, `run_info`, and `confirmed`)
2. `PopupAction::CancelDialog` variant exists with `spec_slug`, `team`, and `at` fields — covered by `test_cancel_dialog_contains_correct_spec_team_and_time` and confirmed implicitly in all tests that construct or pattern-match the variant
3. `open_team_popup()` opens `CancelDialog` when spec has a `SpecRunInfo::Scheduled` entry — covered by `test_open_team_popup_opens_cancel_dialog_for_scheduled_spec`
4. `open_team_popup()` opens normal `TeamDialog` when spec has no scheduled entry — covered by `test_open_team_popup_opens_team_dialog_for_unscheduled_spec`
5. `CancelDialog` contains correct spec slug, team, and datetime — covered by `test_cancel_dialog_contains_correct_spec_team_and_time`
6. `confirm_cancel_dialog()` removes entry from `run_info` and dismisses popup — covered by `test_confirm_cancel_dialog_removes_run_info_entry`
7. After cancellation, pressing Enter opens normal team/action flow — covered by `test_after_cancel_open_team_popup_opens_team_dialog`
8. `dismiss_popup()` for `CancelDialog` dismisses without action (Esc path) — covered by `test_dismiss_cancel_dialog_returns_to_spec_list`
9. `confirm_picker()` on failure sets `picker.error` — not directly tested; acceptable per spec's explicit smoke test note ("these tests do not touch the filesystem or launchd"), which sanctions skipping the failure path at this level

### Notes

All spec-required behaviors have corresponding test coverage. The direct state-injection approach in `test_confirm_picker_on_success_returns_to_launcher_and_sets_status_message` is explicitly sanctioned by the spec's smoke test note. No misdirection and no implementation detail testing detected.

## Task 2: App state changes

**Verdict: Approved**

### Requirements Checklist (derived from spec)

1. `SpecRunInfo::Scheduled { team, at, plist_path }` variant exists — covered by `test_app_new_stores_run_info_entries`, which constructs the variant with all three fields
2. `SpecRunInfo::LastRun { team, completed_at }` variant exists — covered by `test_spec_run_info_last_run_variant`, which constructs the variant with both fields
3. `App.run_info: HashMap<String, SpecRunInfo>` field exists — covered by `test_app_new_initializes_run_info` (empty map) and `test_app_new_stores_run_info_entries` (populated map, keyed by slug)
4. `App.cwd: PathBuf` field exists — covered by `test_app_new_stores_cwd`
5. `App.status_message: Option<String>` field exists, initialized to `None` — covered by `test_app_new_initializes_status_message_as_none`
6. `App::new()` accepts `run_info` param — covered by `sample_app_with_run_info` helper and all Task 2 tests that call it
7. `App::new()` accepts `cwd: PathBuf` param — covered by `test_app_new_stores_cwd` and the `sample_app_with_run_info` helper
8. `scheduled_at` removed from `App` — enforced at compile time; no behavioral test is possible for field absence in Rust. Acceptable.
9. `scheduled_at` removed from `TuiResult` — covered by `test_tui_result_has_no_scheduled_at_field`, which relies on compile-time proof (if the field existed, the struct init in `result()` would include it and fail to compile without the field set). This is the correct approach in Rust.

### Notes

All spec requirements are covered. The `test_tui_result_has_no_scheduled_at_field` test is primarily a compile-time guard — the runtime assertions confirm the result is still produced correctly after the field removal. This is sound. No misdirection and no implementation detail testing.
