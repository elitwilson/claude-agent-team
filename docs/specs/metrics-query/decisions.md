# Metrics Query — Decisions & Assumptions

_Logged by Lead agent. Each entry includes assumption made and rationale._

<!-- Entries added during the run below -->

## D1: LEFT JOIN for runs with no agent_usage (Coder)

The spec says "empty state message" when no rows exist, but doesn't address runs that have no agent_usage rows. Decision: use LEFT JOIN so runs without agent_usage still appear with zeroed token totals. This is more robust and avoids silently hiding runs.

## D2: run_date format from started_at (Coder)

The spec says `run_date` should be "formatted as YYYY-MM-DD" from `started_at`. The `started_at` column stores ISO 8601 timestamps (e.g., `2026-03-27T10:00:00Z`). Decision: extract the date portion using `SUBSTR(started_at, 1, 10)` in SQL rather than parsing in Rust, since the format is consistent.

## D3: Error state on MetricsState (Coder)

The spec says "If the DB read fails, store an error string and render it instead of the table." Decision: added `error: Option<String>` field to `MetricsState` and a `with_error()` constructor. When `error` is Some, the render function displays the error message instead of the table. This keeps the error display co-located with the metrics screen state rather than on App.

## D4: Smoke tests pass immediately (Coder)

The task list called for RED-phase smoke tests, but all navigation wiring was already implemented and tested in earlier tasks (#10, #12). The smoke tests exercise the full end-to-end navigation flow (open_metrics, scroll, close_metrics, verify launcher state preserved) and pass immediately. This is expected — they serve as regression guards.

## D5: Smoke tests as unit tests, not integration tests (Coder)

The crate is binary-only (no lib.rs). Integration tests in tests/ can only access library targets. Rather than adding a lib.rs just for the smoke test, the tests are placed in src/tui/app/tests.rs alongside the other app tests.
