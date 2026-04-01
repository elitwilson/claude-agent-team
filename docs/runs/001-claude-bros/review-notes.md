# Review Notes — claude-bros TDD

## Task #6: Config + Discovery — Review Failing Tests

**Status: APPROVED**

All spec requirements have corresponding test coverage:

- Config loading: all fields, partial fields with defaults, no config file defaults, unknown keys ignored
- Spec discovery: `.md` only, skips non-md, skips subdirectories, empty dir, nonexistent dir error
- Team discovery: strips `.md` extension, skips non-md, empty dir, nonexistent dir error

Tests are well-structured — they test observable behavior (function inputs/outputs) rather than implementation details. No gaps or misalignments found.

## Task #7: TUI — Review Failing Tests

**Status: FLAGGED — 1 gap**

14 tests covering construction, panel navigation, spec/team list navigation, headless toggle, and confirm/result. Good coverage overall.

**Gap:** No test for quit (`q` key). The spec lists "q quit" as a keybinding and the task description explicitly requires "Q/quit key causes TUI to exit without confirming." The initial state test checks `should_quit` starts false, but no test verifies that calling a quit method sets `should_quit = true` (or equivalent). This is a spec requirement with zero coverage.

**No other issues found.** Tests are behavioral, not implementation-detail testing. Rendering tests correctly excluded per task scope.

## Task #8: Run Pipeline — Review Failing Tests

**Status: FLAGGED — 3 gaps**

10 tests across preflight (2), prompt (5), runner (3). The approach of extracting pure helper functions is sound and the covered tests are well-written. However, several spec requirements from the task description have zero coverage:

**Gap 1: Dirty working tree check (preflight.rs).** The task description explicitly requires "git clean check detects dirty working tree and errors." No test exists for this. This is a preflight safety gate — the spec says "If preflight fails (dirty working tree...), print a clear error message to stdout and exit."

**Gap 2: `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` env var (runner.rs).** The task description explicitly requires this is set in the command environment. No test verifies it. The spec says "Must be set in the environment before spawning claude."

**Gap 3: OAuth token loading (runner.rs).** The task description explicitly requires "OAuth token loading — proceeds without token if security command fails, prints warning." No test exists. The spec says "If it fails or returns empty, proceed without setting CLAUDE_OAUTH_TOKEN and print a warning."

**Acknowledged:** Git operations and process spawning are harder to unit test, but these were explicitly listed in the task description as required coverage. At minimum, the testable aspects (e.g., function returns error on dirty tree, env var is present in built command config) should have tests.

**No other issues.** Prompt tests and pure helper tests are correct and behavioral.

## Task #9: Metrics — Review Failing Tests

**Status: FLAGGED — 1 gap**

20 tests across parser (14) and db (6). DB tests are thorough — schema init, idempotency, run insertion with data verification, agent_usage with foreign key, multiple agents per run. Parser helper tests (derive_project_dir, is_agent_file, attribute_role, extract_tokens) are well-designed and cover the spec's requirements including case insensitivity and precedence order.

**Gap: Message-level timestamp filtering (parser.rs).** The task description explicitly requires "message filtering by timestamp >= started_at." This is the core mechanism that isolates current-run data from pre-existing messages — the spec emphasizes "Do not filter by file mtime. Instead, parse all JSONL files and include only messages where message.timestamp >= started_at." No test verifies this filtering behavior. This is testable as a pure function (given a list of messages and a started_at, return only messages with timestamp >= started_at).

**No other issues.** The helper function decomposition is sound and all other spec requirements have coverage. File discovery and token summing are integration-level concerns that are reasonable to defer.

## Task #10: MR + Summary — Review Failing Tests

**Status: APPROVED**

9 tests covering MR title (success/incomplete prefix), MR description (success/warning), push args (GitLab push options), and post-run summary (all success/MR failed/metrics failed).

All spec requirements have corresponding test coverage:

- MR title: no prefix on exit code 0, `INCOMPLETE:` prefix on non-zero (including negative)
- MR description: clean on success, warning on failure
- Push args: includes origin, branch, GitLab push options (merge_request.create, target, title)
- Summary: includes branch name, MR status, metrics status; handles both success and failure cases

Tests are well-structured — pure helper functions with observable outputs, no implementation detail testing. No gaps or misalignments found.
