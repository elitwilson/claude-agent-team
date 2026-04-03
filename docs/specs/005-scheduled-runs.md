---
number: 005
status: complete
base_branch: main
---

# Feature: Scheduled Agent Runs (Backend)

## Summary

Today, running an agent team requires the user to stay in the terminal until the run completes. This feature adds the ability to schedule a run for a future date and time, then exit the terminal entirely. At the scheduled time, macOS fires the agent run automatically via launchd — the OS's native job scheduler — without any background process from `claude-bros` itself. The feature also adds a non-interactive `run` subcommand so the binary can be invoked directly without the TUI.

---

## Requirements

- The binary must accept a `run` subcommand for non-interactive execution: `claude-bros run --spec <name> --team <name> [--headless]`
- A `scheduler` module must expose a function to schedule a run: given a spec name, team name, headless flag, working directory, and a future `DateTime`, it writes a launchd plist and registers it with `launchctl load`
- Scheduled runs must execute wrapped in `caffeinate -i` to prevent the Mac from sleeping during the agent run
- After a scheduled run completes, the binary must delete the plist file and unload it from launchd (self-cleanup)
- The plist path is passed as `--cleanup-plist <path>` in the scheduled invocation so the binary knows exactly what to remove
- The scheduler must return an error if the scheduled time is not at least 1 minute in the future
- Plist files are written to `~/Library/LaunchAgents/` with the naming convention `com.claude-agent-team.<spec-slug>.plist`
- A `scheduler::list_pending` function returns all scheduled runs belonging to this tool by scanning `~/Library/LaunchAgents/` for files matching the naming convention

---

## Scope

### In Scope

- `run` subcommand with `--spec`, `--team`, `--headless`, and `--cleanup-plist` flags
- `src/scheduler.rs` module: plist generation, `launchctl load/unload`, pending run discovery
- Self-cleanup after scheduled execution
- Unit tests for plist generation and pending run discovery

### Out of Scope

- TUI changes — the action dialog and time picker are covered in a separate spec
- Cancellation of scheduled runs via the TUI
- Any platform other than macOS (launchd is macOS-only; this binary already is)
- Recurring schedules — all scheduled runs are one-shot

---

## Technical Approach

**`run` subcommand** — detected in `main()` by checking `std::env::args()`. If the first arg is `"run"`, parse the remaining args manually (no clap needed for four flags) and call directly into the existing preflight + prompt render + `runner::run_claude` flow. After the run, handle `--cleanup-plist` if present. The existing `run()` function in `main.rs` becomes the TUI path; add a `run_scheduled()` function for the direct path.

**`src/scheduler.rs`** — owns everything launchd-related:

```rust
pub struct ScheduledRun {
    pub spec: String,
    pub team: String,
    pub headless: bool,
    pub scheduled_at: chrono::DateTime<chrono::Local>,
    pub plist_path: PathBuf,
}

pub fn schedule_run(
    spec: &str,
    team: &str,
    headless: bool,
    working_dir: &Path,
    scheduled_at: DateTime<Local>,
) -> Result<ScheduledRun>

pub fn list_pending() -> Result<Vec<ScheduledRun>>
```

**Plist format** — the plist embeds `caffeinate -i <binary> run ...` as `ProgramArguments`, sets `WorkingDirectory` to the project directory (so relative paths like `docs/specs/` resolve correctly), and uses `StartCalendarInterval` with Month/Day/Hour/Minute fields derived from the scheduled datetime. The plist path is also passed as `--cleanup-plist <path>` so the binary removes it after running.

`StartCalendarInterval` without a year field repeats annually, but self-cleanup after the first run makes it effectively one-shot.

**Self-cleanup** — when `--cleanup-plist <path>` is present in args, after `run_claude` returns, call `launchctl unload <path>` then `fs::remove_file(<path>)`. Both steps are **fatal** — return an error if either fails. Because `StartCalendarInterval` has no `Year` field, a missed cleanup silently turns a one-shot run into a recurring annual job.

**Plist parsing for `list_pending`** — scan `~/Library/LaunchAgents/` for files matching `com.claude-agent-team.*.plist`, parse the Label, ProgramArguments, StartCalendarInterval, and WorkingDirectory from the XML. Use `plist` crate or manual XML parsing. The `plist` crate is the cleaner option.

**Binary path** — the plist must reference the binary by absolute path. Resolve it at schedule time with `std::env::current_exe()`.

---

## Success Criteria

- [ ] `claude-bros run --spec 005-scheduled-runs.md --team feature-dev --headless` runs the agent without launching the TUI
- [ ] `scheduler::schedule_run(...)` writes a valid plist to `~/Library/LaunchAgents/` and `launchctl load` succeeds
- [ ] The plist's `ProgramArguments` wraps the invocation in `caffeinate -i`
- [ ] `scheduler::schedule_run(...)` returns an error if `scheduled_at` is less than 1 minute in the future
- [ ] After a scheduled run completes, the plist file is removed from `~/Library/LaunchAgents/`
- [ ] `scheduler::list_pending()` returns the scheduled run after `schedule_run` and an empty list after cleanup
- [ ] Unit tests cover: plist generation output, future-time validation, pending run discovery

---

## Tasks

- [ ] **`run` subcommand arg parsing:** Add `run` subcommand detection to `main()`. Parse `--spec`, `--team`, `--headless`, `--cleanup-plist` from `std::env::args()`. Add `run_scheduled()` in `main.rs` that reuses the existing preflight + prompt render + `runner::run_claude` flow. Handle `--cleanup-plist` after the run. Unit test the arg parser.

- [ ] **`scheduler` module — plist generation:** Implement `scheduler::schedule_run`. Generate the plist XML with correct `ProgramArguments` (caffeinate wrapping, binary path via `current_exe`, all run args including `--cleanup-plist`), `WorkingDirectory`, `StartCalendarInterval`, and `Label`. Write to `~/Library/LaunchAgents/`. Call `launchctl load`. Unit test the generated XML structure against expected values.

- [ ] **`scheduler` module — pending run discovery:** Implement `scheduler::list_pending`. Scan `~/Library/LaunchAgents/` for `com.claude-agent-team.*.plist` files, parse each into a `ScheduledRun`. Unit test with fixture plist files. No `working_dir` parameter — discovery is global across all scheduled runs for this tool.

- [ ] **Self-cleanup:** In `run_scheduled()`, after `run_claude` returns, if `--cleanup-plist` was provided: run `launchctl unload <path>`, then `fs::remove_file`. Both steps are **fatal** — return an error if either fails, since a missed cleanup turns a one-shot run into a silently recurring annual job. Test that cleanup is called with the correct path.

---

## Considerations

- `StartCalendarInterval` has no `Year` field — without self-cleanup, a scheduled run would repeat annually. Self-cleanup is what makes it one-shot; cleanup failures are fatal for this reason.
- `launchctl load` and `unload` shell out to the OS. These can't be meaningfully unit tested — integration tested only. Keep the shell-out surface thin in `scheduler.rs` so the plist generation and parsing logic can be tested without hitting launchd.
- The `plist` crate (`plist = "1"`) handles Apple property list serialization cleanly. Check if adding it is acceptable before using manual XML string building.
- `WorkingDirectory` in the plist must be the project directory at schedule time (the CWD when `claude-bros` was invoked from the TUI). This must be captured and embedded, not re-derived at run time.
- The binary path from `current_exe()` may point to a debug build during development. The scheduled plist will then use that path — fine for development, but worth noting.
- `caffeinate -i` prevents idle sleep. It does not prevent the user from manually sleeping the machine. On next wake, launchd re-fires the job.
