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
