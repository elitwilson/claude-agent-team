---
number: 002
status: complete
base_branch: main
---

# Feature: Per-Run Token Metrics Collection

> **Note:** This feature is implemented as part of `claude-bros` (see `docs/specs/001-claude-bros.md`), not as a standalone Python script. The technical design below remains the authoritative reference for the metrics subsystem (`metrics/parser.rs` and `metrics/db.rs`). The entry point, language, and integration sections no longer apply as written.

## Summary

After each agent team run completes, collect token usage data from Claude Code's local session files and persist it to a global SQLite database. Data is captured per-agent and per-run, tagged with project and team context, so token costs can be quantified and compared across runs over time. This is a collection-only feature — querying and reporting are out of scope.

---

## Requirements

- Token metrics are collected automatically at the end of every agent team run (interactive and headless)
- Metrics are written to a global SQLite database at `~/.claude/claude-agent-team-metrics.db`
- Each run record captures: project (cwd), team type, feature slug, start time, end time, agent exit code
- Token usage is recorded per agent: input, output, cache creation, and cache read tokens
- Agent roles are attributed by name where possible: `orchestrator`, `coder`, `reviewer`
- If role attribution cannot be determined, agents fall back to `agent_1`, `agent_2`, etc. ordered by first message timestamp
- If metrics collection fails for any reason, the run is not affected — a warning is logged and execution continues
- The database and its tables are created automatically on first run if they do not exist

---

## Scope

### In Scope

- `scripts/collect-metrics.py` — standalone script, called by `run-agent-team.py` after `run_agent()` returns
- SQLite schema: `runs` and `agent_usage` tables
- JSONL file discovery and parsing from `~/.claude/projects/<project-dir>/`
- Message-level timestamp filtering to isolate messages belonging to the current run
- Agent role attribution via heuristic (documented below)
- DB initialisation (CREATE TABLE IF NOT EXISTS) on first run
- Integration into `run-agent-team.py`: capture `started_at`, call `collect-metrics.py`, handle failure gracefully

### Out of Scope

- Querying or reporting on collected data (future feature)
- Cost calculation in dollars (token counts only)
- Metrics for non-agent-team Claude Code sessions
- Retry logic for failed DB writes

---

## Technical Approach

**Entry point:** `run-agent-team.py` captures `started_at = datetime.utcnow().isoformat()` immediately before calling `run_agent()`. After `run_agent()` returns, it invokes:

```bash
python3 scripts/collect-metrics.py \
  --feature-slug <slug> \
  --team <team> \
  --project <cwd> \
  --started-at <ISO8601> \
  --exit-code <int>
```

`collect-metrics.py` is responsible for everything else.

**JSONL file discovery:** Claude Code writes session data to `~/.claude/projects/<project-dir>/` where `<project-dir>` is the `cwd` with leading `/` dropped and remaining `/` replaced with `-`. Example: `/Users/charlo/dev/myproject` → `-Users-charlo-dev-myproject`.

> **Assumption:** This path mapping is inferred from observed behavior, not official documentation. It may need adjustment if Claude Code changes its file layout.

Two file types exist in that directory:
- Main session file: any `.jsonl` file not prefixed with `agent-` → attributed to `orchestrator`
- Sub-agent files: `agent-*.jsonl` files → attributed to `coder`, `reviewer`, or `agent_N` fallback

**Message-level timestamp filtering:** Do not filter by file `mtime`. Instead, parse all JSONL files in the project directory and include only messages where `message.timestamp >= started_at`. This correctly handles the case where a pre-existing session file has new messages appended during the run.

**Token extraction:** Token data appears only on `assistant` type messages, in `message.usage`. Extract four fields per message:
- `input_tokens` (required)
- `output_tokens` (required)
- `cache_creation_input_tokens` (optional, default 0)
- `cache_read_input_tokens` (optional, default 0)

Sum all four fields across all matching messages per file to get per-agent totals.

**Agent role attribution:** For each `agent-*.jsonl` file, read the first `user` message whose timestamp falls within the run window. That message is the spawn prompt written by the Lead and will contain the role instructions. Apply the following heuristic in order:

1. If the message content contains the word `Coder` (case-insensitive) → role = `coder`
2. If the message content contains the word `Reviewer` (case-insensitive) → role = `reviewer`
3. Otherwise → role = `agent_1`, `agent_2`, etc. ordered by first message timestamp ascending

> **Documented heuristic:** Role attribution depends on the Lead's spawn prompt containing the role name. This works reliably given the current prompt structure in `prompts/teams/feature-dev.md`. If a new team type uses different role names, the keyword list in `collect-metrics.py` must be updated to match.

**Database schema:**

```sql
CREATE TABLE IF NOT EXISTS runs (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    feature_slug        TEXT NOT NULL,
    team                TEXT NOT NULL,
    project             TEXT NOT NULL,
    started_at          TEXT NOT NULL,
    completed_at        TEXT NOT NULL,
    agent_exit_code     INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS agent_usage (
    id                          INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id                      INTEGER NOT NULL REFERENCES runs(id),
    agent_role                  TEXT NOT NULL,
    input_tokens                INTEGER NOT NULL DEFAULT 0,
    output_tokens               INTEGER NOT NULL DEFAULT 0,
    cache_creation_tokens       INTEGER NOT NULL DEFAULT 0,
    cache_read_tokens           INTEGER NOT NULL DEFAULT 0
);
```

**Error handling:** All of `collect-metrics.py` is wrapped in a top-level try/except. On any failure, print a warning to stderr and exit 0. `run-agent-team.py` does not check the exit code of the metrics script — failure is intentionally non-fatal.

---

## Success Criteria

- [ ] After a completed run, a row exists in `runs` with correct feature slug, team, project, and timestamps
- [ ] A row exists in `agent_usage` for each agent that participated in the run
- [ ] `agent_role` is `orchestrator` for the main session and `coder`/`reviewer` for sub-agents (given standard feature-dev team prompts)
- [ ] Token counts across all agents sum to a non-zero total consistent with a real run
- [ ] Running the script twice for the same run does not duplicate rows (idempotent — see considerations)
- [ ] If `~/.claude/claude-agent-team-metrics.db` does not exist, the script creates it with correct schema
- [ ] If metrics collection raises an exception, `run-agent-team.py` continues and creates the MR normally
- [ ] No metrics are collected for messages timestamped before `started_at`

---

## Tasks

- [ ] **DB module:** Implement `scripts/metrics_db.py` — handles DB connection, schema initialisation (`CREATE TABLE IF NOT EXISTS`), and insert functions for `runs` and `agent_usage`. No JSONL parsing dependency.
- [ ] **JSONL parser:** Implement `scripts/metrics_parser.py` — discovers JSONL files for a given project dir, filters messages by timestamp, extracts token usage per file, attributes agent roles via heuristic. Returns structured data, no DB dependency.
- [ ] **collect-metrics.py:** Wire together parser and DB module. Accept CLI args (`--feature-slug`, `--team`, `--project`, `--started-at`, `--exit-code`). Wrap in top-level try/except with stderr warning on failure.
- [ ] **run-agent-team.py integration:** Capture `started_at` before `run_agent()`. Call `collect-metrics.py` after `run_agent()` returns, passing all required args. Do not check exit code of metrics script.

---

## Considerations

- **Idempotency:** The spec does not require true idempotency (upsert on re-run). A re-run of the same spec on the same date will insert a new row. This is acceptable — each invocation of `run-agent-team.py` is a distinct run. Do not attempt to deduplicate.
- **No JSONL files found:** If no matching messages are found for the run window (e.g. run was extremely short or files are missing), still write the `runs` row with zero token totals. Do not treat this as an error.
- **Multiple main session files:** If more than one non-`agent-` JSONL file has messages in the run window, sum them all and attribute to `orchestrator`. Do not attempt to split.
- **Timezone:** All timestamps are UTC. `started_at` passed from `run-agent-team.py` must be UTC ISO8601. JSONL message timestamps are already UTC.
- **Project dir mapping:** Derive `<project-dir>` from `--project` arg using: `project.lstrip('/').replace('/', '-')`. This is an assumption — validate empirically on first run.
