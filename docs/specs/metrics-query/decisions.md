# Metrics Query — Decisions & Assumptions

_Logged by Lead agent. Each entry includes assumption made and rationale._

<!-- Entries added during the run below -->

## D1: LEFT JOIN for runs with no agent_usage (Coder)

The spec says "empty state message" when no rows exist, but doesn't address runs that have no agent_usage rows. Decision: use LEFT JOIN so runs without agent_usage still appear with zeroed token totals. This is more robust and avoids silently hiding runs.

## D2: run_date format from started_at (Coder)

The spec says `run_date` should be "formatted as YYYY-MM-DD" from `started_at`. The `started_at` column stores ISO 8601 timestamps (e.g., `2026-03-27T10:00:00Z`). Decision: extract the date portion using `SUBSTR(started_at, 1, 10)` in SQL rather than parsing in Rust, since the format is consistent.
