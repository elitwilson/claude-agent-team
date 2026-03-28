# Feature: Metrics Query View

## Summary

Add a metrics screen to the `claude-bros` TUI. From the launcher, pressing `m` switches to a full-screen table showing past agent team runs — one row per run, columns for key token metrics. Pressing `q` or `Esc` returns to the launcher. This is a read-only, display-only POC — no filtering, no drill-down, no interaction beyond navigation.

---

## Requirements

- Pressing `m` from the launcher switches to the metrics screen
- The metrics screen shows a scrollable table of past runs, most recent first
- Each row shows: run date, feature slug, team, total input tokens, total output tokens, total cache tokens, exit code
- Pressing `q` or `Esc` from the metrics screen returns to the launcher
- If the database does not exist or contains no rows, display a friendly empty state message
- If the database read fails, display an error message — do not crash

---

## Scope

### In Scope

- New `metrics` screen/view in the existing `claude-bros` TUI
- Read from `~/.claude/claude-agent-team-metrics.db` (same path used by the runner)
- Run-level token totals only (sum across all agents per run)
- `src/tui/metrics.rs` — new module for the metrics screen state and rendering
- Navigation: `m` from launcher → metrics screen, `q`/`Esc` → back to launcher
- `src/metrics/query.rs` — DB read functions (separate from existing write functions in `db.rs`)

### Out of Scope

- Per-agent breakdown or drill-down
- Filtering by project, team, or date
- Sorting
- Pagination (scrolling the table is sufficient)
- Any write operations

---

## Technical Approach

**New screen state:** The TUI currently has one screen (launcher). Add a `Screen` enum to `tui/app.rs`:

```rust
pub enum Screen {
    Launcher,
    Metrics,
}
```

`App` holds the current screen. `m` sets it to `Metrics`, `q`/`Esc` sets it back to `Launcher`. The existing launcher rendering is unchanged — the event loop and render function branch on `app.screen`.

**New module `src/metrics/query.rs`:** Read functions only, no writes. One function:

```rust
pub fn fetch_runs(conn: &Connection) -> Result<Vec<RunSummary>>
```

`RunSummary` is a plain struct:

```rust
pub struct RunSummary {
    pub run_date: String,       // from started_at, formatted as YYYY-MM-DD
    pub feature_slug: String,
    pub team: String,
    pub total_input: u64,
    pub total_output: u64,
    pub total_cache: u64,       // cache_creation + cache_read combined
    pub exit_code: i32,
}
```

Totals are computed by joining `runs` and `agent_usage` and summing per run. Ordered by `started_at DESC`.

**New module `src/tui/metrics.rs`:** Holds the metrics screen state (`MetricsState`) and the render function. `MetricsState` contains the loaded `Vec<RunSummary>` and a scroll offset. Rendering uses Ratatui's `Table` widget with a header row and one data row per run. Exit code column renders `✓` for 0, `✗` for non-zero.

**DB connection in TUI:** Open the metrics DB connection lazily when the user first presses `m`. Cache the result on `App` — `Option<Vec<RunSummary>>`. If the DB doesn't exist, store an empty vec. If the read fails, store an error string and render it instead of the table.

**`main.rs`:** No changes needed — screen switching is handled entirely within the TUI event loop.

---

## Success Criteria

- [ ] Pressing `m` from the launcher switches to the metrics screen without visual artifacts
- [ ] Pressing `q` or `Esc` from the metrics screen returns to the launcher
- [ ] The table renders with correct columns and one row per run
- [ ] Rows are ordered most recent first
- [ ] Token totals are summed correctly across all agents for each run
- [ ] Exit code renders as `✓` or `✗`
- [ ] If no DB exists, an empty state message is shown (not a crash)
- [ ] If the DB read fails, an error message is shown (not a crash)
- [ ] The launcher behaves identically to before — this change is additive only

---

## Tasks

- [ ] **Query module:** Implement `src/metrics/query.rs` — define `RunSummary` struct, implement `fetch_runs()` joining `runs` and `agent_usage`, summing token columns per run, ordered by `started_at DESC`. Unit test with an in-memory SQLite DB.

- [ ] **Screen state:** Add `Screen` enum to `src/tui/app.rs`. Add `screen` field to `App`, defaulting to `Screen::Launcher`. Add `MetricsState` to `src/tui/metrics.rs` holding `Vec<RunSummary>` and scroll offset. Add methods for screen switching and scroll navigation.

- [ ] **Metrics rendering:** Implement the render function in `src/tui/metrics.rs` using Ratatui's `Table` widget. Header row: Date, Spec, Team, Input, Output, Cache, Status. Data rows from `RunSummary`. Empty state and error state handling.

- [ ] **Event loop + wiring:** Update `src/tui/ui.rs` event loop — `m` switches to metrics screen (loading runs lazily on first open), `q`/`Esc` on metrics screen returns to launcher, ↑↓ scroll the table. Branch render function on `app.screen`. Wire `fetch_runs` call through `main.rs` DB connection setup.

- [ ] **Smoke test:** Write an integration test that constructs an `App` with seeded `RunSummary` data, simulates `m` keypress, verifies screen switches to `Metrics`, simulates `Esc`, verifies screen returns to `Launcher`. Confirms the full navigation flow works end-to-end.

---

## Considerations

- **DB connection ownership:** `rusqlite::Connection` is not `Send` — keep it on the main thread. Open it in the event loop when `m` is first pressed, pass the loaded data into `MetricsState`. Do not store the connection on `App`.
- **Empty DB:** If `~/.claude/claude-agent-team-metrics.db` does not exist, `rusqlite::Connection::open` will create an empty file. Call `init_db` before `fetch_runs` to ensure the schema exists, then an empty result is normal — render the empty state message.
- **Cache total:** Sum `cache_creation_tokens + cache_read_tokens` into a single `total_cache` column for display simplicity. The distinction between creation and read can come in a future, more detailed view.
- **Scroll:** Ratatui's `Table` widget does not scroll natively — maintain a `scroll_offset: usize` in `MetricsState` and slice the rows accordingly before rendering.
- **This spec produces a modified binary with new interactive behavior** — the smoke test task is mandatory. Without it, screen switching will not be verified end-to-end and the wiring in `main.rs`/`ui.rs` will have no forcing function.
