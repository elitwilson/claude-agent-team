# Feature: claude-bros — Rust TUI Launcher

## Summary

`claude-bros` is a Rust terminal UI that serves as the primary entrypoint for running agent teams. It replaces the existing Python scripts entirely. The user launches `claude-bros` from within a target project directory, selects a spec file, selects a team, configures run options, and confirms. The TUI then clears and hands the terminal over to the `claude` interactive session unobstructed. When the session exits, `claude-bros` resumes in the background to collect metrics and create the GitLab MR, then prints a brief summary and exits.

---

## Requirements

- `claude-bros` is invoked from within a target project directory with no arguments
- The TUI presents a spec file picker populated from `docs/specs/` (`.md` files only, no subdirectories)
- The TUI presents a team selector populated from available teams in `prompts/teams/`
- The TUI presents a run options panel with a headless mode toggle (default: off)
- Confirming the selection clears the TUI and hands the terminal to `claude` interactive session — the claude experience is completely unaffected
- After `claude` exits, `claude-bros` collects token metrics and writes them to `~/.claude/claude-agent-team-metrics.db`
- After metrics collection, `claude-bros` creates a GitLab MR
- `claude-bros` prints a brief post-run summary (branch, MR created, metrics written) and exits
- A project-level `.claude-agent-team.toml` config file can override convention defaults
- If no config file is present, all defaults apply — the tool works out of the box with no config required

---

## Scope

### In Scope

- Full Rust rewrite of all functionality currently in `scripts/run-agent-team.py` and `scripts/preflight.py`
- Ratatui TUI: spec selection, team selection, run options panel
- Project-level TOML config with overridable defaults
- Preflight: git clean check, pull latest, branch creation
- Prompt template loading and variable substitution
- `claude` process spawning (spawn + wait, not exec)
- Token metrics collection (port of `docs/specs/metrics-collection.md` — see note in Considerations)
- MR creation via `git push` with GitLab push options
- macOS Keychain OAuth token loading
- Post-run summary output

### Out of Scope

- Metrics querying or display within the TUI
- Multiple simultaneous runs
- Windows or Linux support (macOS only for v1)
- Mouse interaction
- `claude-bros` displaying claude's output inline — claude owns the terminal during the run
- Porting the agent prompts, role definitions, or markdown files — those remain as-is

---

## Technical Approach

**Binary name:** `claude-bros`

**Project location:** Rust project at the repo root. `Cargo.toml` sits alongside `README.md`, `scripts/`, `docs/`, etc. The project has already been scaffolded with all dependencies installed — do not run `cargo init`.

```
<repo-root>/
├── Cargo.toml
├── src/
    ├── main.rs
    ├── config.rs           # TOML config loading and defaults
    ├── preflight.rs        # Git checks and branch creation
    ├── prompt.rs           # Template loading and variable substitution
    ├── runner.rs           # claude process spawning and waiting
    ├── mr.rs               # MR creation via git push
    ├── metrics/
    │   ├── mod.rs
    │   ├── parser.rs       # JSONL discovery, filtering, token extraction, role attribution
    │   └── db.rs           # SQLite schema init and writes
    └── tui/
        ├── mod.rs
        ├── app.rs          # TUI state machine
        └── ui.rs           # Ratatui rendering
```

**Key dependencies:**
- `ratatui` + `crossterm` — TUI rendering and terminal backend
- `serde` + `serde_json` — JSONL parsing
- `toml` + `serde` — config file parsing
- `rusqlite` — SQLite
- `chrono` — timestamp handling
- `anyhow` — error handling
- `string-template` or manual substitution — prompt variable interpolation

**Config file (`.claude-agent-team.toml`):**

```toml
specs_dir = "docs/specs"      # default
default_team = "feature-dev"  # default
base_branch = "main"          # default
```

Loaded from `cwd` at startup. If not present, defaults are used. Unknown keys are ignored.

**TUI layout:**

Three vertically stacked panels with Tab to move between them:

```
┌─ Spec ──────────────────────────────┐
│ > my-feature.md                     │
│   metrics-collection.md             │
│   another-feature.md                │
├─ Team ──────────────────────────────┤
│ > feature-dev                       │
├─ Run Options ───────────────────────┤
│   [ ] Headless                      │
└─────────────────────────────────────┘
  ↑↓ navigate  Tab switch panel  Enter confirm  q quit
```

- Spec list: reads `docs/specs/` (or `specs_dir` from config), lists `.md` files only, skips subdirectories
- Team list: reads `prompts/teams/`, lists `.md` files stripped of extension
- Run options: toggleable flags with Space; only `Headless` for v1
- Enter on any panel confirms and launches

**Run sequence (after confirm):**

1. Clear TUI, restore terminal to normal mode
2. Run preflight (git clean check, checkout base branch, pull, create feature branch)
3. Load and render prompt template with substitutions (`SPEC_FILE`, `FEATURE_SLUG`, `WORKFLOW_DIR`, `TEAM`)
4. Record `started_at` (UTC)
5. Spawn `claude` process — if headless, pass `--print`; otherwise interactive. Wait for it to exit.
6. Record `completed_at`, capture exit code
7. Collect metrics (parse JSONL files, write to SQLite) — non-fatal on failure
8. Create MR via `git push` with GitLab push options — include `INCOMPLETE:` prefix and warning if exit code != 0
9. Print post-run summary to stdout and exit

**Process spawning:**

Use `std::process::Command` with `spawn()` + `wait()`. Do not use `exec` — `claude-bros` must resume after `claude` exits to handle metrics and MR creation.

The `claude` invocation must include these flags:

```
claude \
  --max-turns 200 \
  --dangerously-skip-permissions \
  --teammate-mode in-process \
  <rendered-prompt>
```

- `--teammate-mode in-process` is required for sub-agent spawning. Without it the team runs as a single agent.
- `--dangerously-skip-permissions` is required to prevent claude from halting on permission prompts during an unattended run.
- `--max-turns 200` caps runaway sessions.
- The rendered prompt string is passed as the final positional argument, not via stdin or a flag.
- In headless mode, prepend `--print` before the other flags.

In interactive mode, stdin is inherited from the terminal. Stdout handling is covered under **Headless mode and log file** in Considerations.

**OAuth token loading:**

Invoke `security find-generic-password -w -s claude-token-1 -a claude` via `Command`. If it fails or returns empty, proceed without setting `CLAUDE_OAUTH_TOKEN` and print a warning.

**Metrics collection** — full detail in `docs/specs/metrics-collection.md`. Summary:
- Derive project dir: `cwd.trim_start_matches('/').replace('/', '-')`
- Scan `~/.claude/projects/<project-dir>/` for `.jsonl` files
- Filter messages by `timestamp >= started_at`
- Extract token fields from assistant messages
- Attribute roles by keyword match on first user message in each `agent-*.jsonl`
- Write `runs` + `agent_usage` rows to SQLite

---

## Success Criteria

- [ ] `claude-bros` launches with no arguments and renders the TUI correctly
- [ ] Spec list is populated from `docs/specs/` and reflects actual `.md` files
- [ ] Team list is populated from `prompts/teams/`
- [ ] Tab switches between panels; ↑↓ navigates within a panel; Space toggles headless; Enter confirms
- [ ] Confirming clears the TUI and the claude interactive session launches with no visual artifacts
- [ ] Claude session is fully interactive and unaffected by `claude-bros`
- [ ] After claude exits, a metrics row is written to `~/.claude/claude-agent-team-metrics.db`
- [ ] After claude exits, a GitLab MR is created for the feature branch
- [ ] A failed run (non-zero exit code) produces an MR with `INCOMPLETE:` title prefix
- [ ] A `.claude-agent-team.toml` with `specs_dir` override causes the TUI to load specs from the specified directory
- [ ] If no `.claude-agent-team.toml` exists, the tool works with defaults
- [ ] Metrics collection failure does not prevent MR creation

---

## Tasks

- [ ] **Config + discovery:** Implement `config.rs`: load `.claude-agent-team.toml` from cwd, fall back to defaults, expose typed config struct. Implement spec and team discovery functions (read directories, filter files). The Rust project is already scaffolded at the repo root with all dependencies in `Cargo.toml` — do not reinitialise.

- [ ] **TUI:** Implement `tui/app.rs` (state machine: which panel is focused, current selections, headless toggle) and `tui/ui.rs` (Ratatui rendering of three-panel layout, keybinding footer). TUI exits cleanly on Enter, restoring terminal state.

- [ ] **Run pipeline:** Implement `preflight.rs` (git checks, branch creation), `prompt.rs` (template load and substitution), `runner.rs` (OAuth token loading, `claude` process spawn + wait). Wire together in `main.rs` post-TUI-exit sequence.

- [ ] **Metrics collection:** Implement `metrics/parser.rs` (JSONL discovery, message-level timestamp filtering, token extraction, role attribution heuristic) and `metrics/db.rs` (SQLite schema init, `runs` and `agent_usage` inserts). Non-fatal — wrap call site in error handler that warns and continues.

- [ ] **MR creation + summary:** Implement `mr.rs` (git push with GitLab push options, title prefix and description warning on non-zero exit code). Print post-run summary to stdout. Wire full post-run sequence in `main.rs`.

---

## Considerations

- **Terminal state:** Ratatui requires entering raw mode and an alternate screen. These must be restored unconditionally before spawning `claude` — use a drop guard or explicit cleanup. A panic that skips cleanup will leave the terminal broken.
- **Preflight failure:** Preflight runs after the TUI clears and the terminal is restored to normal mode, but before `claude` is spawned. If preflight fails (dirty working tree, pull fails, branch already exists), print a clear error message to stdout and exit. Do not spawn `claude` with a dirty environment.
- **WORKFLOW_DIR:** The prompt template references `${WORKFLOW_DIR}` to locate role files. This must resolve to the repo root (the directory containing `prompts/`), not the target project cwd. Resolution strategy: check the `CLAUDE_AGENT_TEAM_DIR` env var first; if set, use it. Otherwise, derive from the binary's own location at runtime (`std::env::current_exe()`): for a development build at `<repo>/claude-bros/target/…/claude-bros`, walk up to the directory containing a `prompts/` subdirectory. If no such ancestor is found, exit with a clear error. Do NOT silently use a wrong path — a bad `WORKFLOW_DIR` produces a broken prompt that will fail silently at runtime.
- **Headless mode and log file:** Both modes write to `logs/agent-runs/<slug>-<date>.log` in the target project directory. Create the directory if it doesn't exist.
  - **Headless:** redirect `claude` stdout and stderr directly to the log file. stdin is `/dev/null`.
  - **Interactive:** stdin is inherited from the terminal so the user can interact. Stdout is piped through an external `tee -a <log>` process (same approach as the Python script) so output goes to both the terminal and the log. This is not true fd inheritance — `claude` writes to a pipe, not directly to the terminal fd. This is acceptable: the `tee` output is immediate and the user experience is indistinguishable in practice. Do NOT attempt PTY-based tee for v1.
- **Python scripts:** `scripts/run-agent-team.py` and `scripts/preflight.py` are superseded by this binary but must be preserved. Do not delete or modify them.
- **`CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS`:** Must be set in the environment before spawning `claude`. Set it on the `Command` env, not as a shell export.
