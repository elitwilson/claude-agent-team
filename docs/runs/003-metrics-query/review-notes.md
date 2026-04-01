# Metrics Query — Review Notes

_Written by Reviewer agent after each test review pass._

<!-- Entries added during the run below -->

## Task: Query Module
**Result:** APPROVED
**Notes:** All spec requirements covered by 6 tests:
- `test_fetch_runs_sums_tokens_across_agents` — verifies token summing across multiple agents per run, including cache_creation + cache_read combined into total_cache
- `test_fetch_runs_maps_fields_correctly` — verifies run_date formatted as YYYY-MM-DD, feature_slug, team, and exit_code mapping
- `test_fetch_runs_ordered_by_started_at_desc` — verifies most-recent-first ordering
- `test_fetch_runs_returns_empty_vec_when_no_rows` — verifies empty DB returns empty vec
- `test_fetch_runs_includes_run_with_no_agent_usage` — verifies runs with no agent_usage rows return zero totals (LEFT JOIN behavior)
- `test_fetch_runs_does_not_mix_tokens_between_runs` — verifies token sums are scoped per run, not cross-contaminated

No spec gaps found. Tests target observable behavior, not implementation details.

## Task: Screen State
**Result:** APPROVED
**Notes:** All spec requirements covered by 8 tests across two files:

In `src/tui/app/tests.rs`:
- `test_app_defaults_to_launcher_screen` — verifies `app.screen` defaults to `Screen::Launcher`
- `test_app_metrics_state_is_none_by_default` — verifies `app.metrics_state` starts as `None`

In `src/tui/metrics/tests.rs`:
- `test_new_sets_runs_and_zero_scroll` — verifies `MetricsState::new` stores runs and initializes `scroll_offset` to 0
- `test_scroll_down_increments_offset` — verifies `scroll_down` increments offset
- `test_scroll_down_clamps_at_last_row` — verifies `scroll_down` clamps at `len - 1`
- `test_scroll_down_noop_when_empty` — verifies `scroll_down` is no-op on empty runs
- `test_scroll_up_decrements_offset` — verifies `scroll_up` decrements offset
- `test_scroll_up_clamps_at_zero` — verifies `scroll_up` clamps at 0

No spec gaps found. Tests cover Screen enum defaults, MetricsState construction, and scroll navigation boundary conditions. All tests target observable behavior.

## Task: Metrics Rendering
**Result:** APPROVED
**Notes:** All spec requirements covered by 6 render tests in `src/tui/metrics/tests.rs`:
- `test_render_shows_column_headers` — verifies all 7 column headers: Date, Spec, Team, Input, Output, Cache, Status
- `test_render_shows_run_data` — verifies run date, feature slug, team, and token values appear in rendered output
- `test_render_exit_code_zero_shows_check` — verifies exit code 0 renders as `✓`
- `test_render_exit_code_nonzero_shows_cross` — verifies non-zero exit code renders as `✗`
- `test_render_empty_state_shows_message` — verifies empty runs vec shows a friendly message
- `test_render_error_state_shows_error_message` — verifies error state displays the error string

Tests use `TestBackend` + `render_to_string` helper to verify observable rendered output. No spec gaps found.

## Task: Event Loop Wiring
**Result:** APPROVED
**Notes:** All testable spec requirements covered by 5 tests in `src/tui/app/tests.rs`:
- `test_open_metrics_switches_screen` — verifies `open_metrics()` sets screen to `Screen::Metrics`
- `test_open_metrics_stores_state` — verifies `open_metrics()` stores `MetricsState` with runs
- `test_close_metrics_returns_to_launcher` — verifies `close_metrics()` sets screen back to `Screen::Launcher`
- `test_move_up_on_metrics_screen_scrolls` — verifies `move_up()` delegates to `MetricsState.scroll_up()` when on Metrics screen
- `test_move_down_on_metrics_screen_scrolls` — verifies `move_down()` delegates to `MetricsState.scroll_down()` when on Metrics screen

Render branching and lazy DB load are wiring concerns in `ui.rs`/`main.rs` — appropriately deferred to the smoke test (task #13). No spec gaps at this layer.

## Task: Smoke Test
**Result:** APPROVED
**Notes:** All spec requirements covered by 4 smoke tests in `src/tui/app/tests.rs`:
- `test_smoke_full_navigation_m_then_esc` — constructs App with seeded RunSummary data, opens metrics, verifies Metrics screen, closes via Esc, verifies return to Launcher (directly matches spec)
- `test_smoke_full_navigation_m_then_q` — same flow using `q` to return (spec requires both `q` and `Esc`)
- `test_smoke_metrics_scroll_navigation` — verifies scroll down/up works during metrics screen
- `test_smoke_launcher_unchanged_after_metrics_roundtrip` — verifies launcher state (spec_index, team_index, flags) preserved after metrics roundtrip (spec: "launcher behaves identically to before")

Tests pass immediately (D4) because implementation was completed in prior tasks — they serve as regression guards per the spec's mandatory smoke test requirement. Placed as unit tests (D5) because the crate is binary-only. Both decisions are documented and reasonable. No spec gaps found.
