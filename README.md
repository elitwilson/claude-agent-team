# claude-bros

A tool for autonomously implementing features using Claude Code agent teams. Pick a spec in the TUI, assign a team, and `claude-bros` handles the rest: pre-flight git setup, agent session, metrics collection.

```
[Spec] → [TUI] → [Pre-flight] → [Agent Team] → [Metrics]
```

The agent team consists of three roles: a **Lead** that coordinates, a **Coder** that implements via TDD, and a **Reviewer** that gates tests before implementation begins.

---

## Prerequisites

- Claude Code CLI (`claude --version`)
- Rust toolchain (`cargo --version`)
- macOS (Keychain integration for OAuth token)

---

## Building

```bash
cargo build --release
```

The binary is at `target/release/claude-bros`. Add it to your PATH or run via `cargo run` from this repo.

---

## Usage

Run from within your target project directory:

```bash
claude-bros
```

This opens the TUI where you select a spec and a team. On confirm, `claude-bros` will:

1. Run pre-flight checks (clean working tree, checkout base branch, pull, create feature branch)
2. Spawn the agent team session interactively
3. Collect token metrics and write them to `~/.claude/claude-agent-team-metrics.db`
4. Print a post-run summary

> **Important:** When the agent session finishes, exit Claude Code cleanly using `/exit` or `q` within the UI. Closing the terminal tab or killing the process prevents `claude-bros` from collecting metrics after the run.

### TUI Controls

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate list |
| `Tab` | Switch panel (Spec → Team → Run Options) |
| `Space` | Toggle headless mode |
| `Enter` | Confirm and start run |
| `m` | Open metrics viewer |
| `q` | Quit |

### Headless mode

Toggle with `Space` in the TUI. Redirects all claude output to a log file instead of the terminal. Useful for overnight runs.

Log files are written to `logs/agent-runs/<slug>-<YYYYMMDD>.log` in your target project.

---

## Configuration

`claude-bros` works with no config file — all defaults apply. To override, add a `.claude-agent-team.toml` to your target project root:

```toml
specs_dir = "docs/specs"       # default
default_team = "feature-dev"   # default
base_branch = "main"           # default
```

---

## Specs

Specs live in `docs/specs/` by default. Each spec is a Markdown file with YAML frontmatter.

### Naming convention

Specs use a three-digit zero-padded sequential prefix:

```
docs/specs/001-my-feature.md
docs/specs/002-another-feature.md
```

Numbers are assigned in the order specs are created and are never reused.

### Frontmatter

```yaml
---
number: 001
status: ready
---
```

| Status | Meaning | Shown in TUI |
|--------|---------|--------------|
| `ready` | Ready for implementation | Yes |
| `needs_attention` | Previous run did not complete | Yes (yellow) |
| `complete` | Implemented | No |

Specs with missing or unrecognized status are treated as `ready`.

The team lead updates the spec's `status` at the end of each run: `complete` if all tasks finished, `needs_attention` if any did not.

### Writing a spec

Copy `docs/spec-template.md` as your starting point. A spec should include:

- Summary
- Requirements
- Scope (in and out)
- Technical approach
- Success criteria
- Discrete, dependency-ordered tasks
- Considerations

---

## Project Structure

```
claude-bros/
├── src/
│   ├── main.rs             # Entry point and post-run sequence
│   ├── config.rs           # Config loading, spec/team discovery, frontmatter parsing
│   ├── preflight.rs        # Git checks and feature branch creation
│   ├── runner.rs           # claude process spawning, OAuth token loading
│   ├── prompt.rs           # Prompt template loading and variable substitution
│   ├── tui/                # Terminal UI (ratatui)
│   │   ├── app.rs          # TUI application state
│   │   ├── ui.rs           # Rendering and event loop
│   │   └── metrics.rs      # Metrics screen
│   └── metrics/            # Token metrics collection
│       ├── parser.rs       # JSONL discovery and token extraction
│       └── db.rs           # SQLite schema and writes
│
├── prompts/
│   └── teams/
│       └── feature-dev.md  # Lead agent initialization prompt
│
├── docs/
│   ├── spec-template.md    # Copy this when writing a new spec
│   ├── specs/              # Feature specs (00N-<slug>.md)
│   └── roles/
│       └── feature-dev/    # Role definitions
│           ├── lead.md
│           ├── coder.md
│           └── reviewer.md
│
└── rules/                  # Symlink into ~/.claude/rules/ for project-wide rules
```

---

## How It Works

1. **TUI** — select a spec and team; specs with `status: complete` are hidden
2. **Pre-flight** — validates clean git state, checks out base branch, pulls, creates `feature/<slug>-<YYYYMMDD>`
3. **Agent team** — Lead reads the spec and role definitions, spawns Coder and Reviewer, coordinates a TDD loop per task
4. **TDD loop** — Coder writes failing tests → Reviewer gates → Coder implements → commit → repeat
5. **Metrics** — token usage is parsed from Claude's JSONL logs and written to SQLite (non-fatal if it fails)
6. **Summary** — branch name and metrics status printed to stdout

### Runtime artifacts

| File | Purpose |
|------|---------|
| `docs/specs/<slug>/decisions.md` | Ambiguities and assumptions the Lead logged during the run |
| `docs/specs/<slug>/review-notes.md` | Reviewer gate outcomes per task |
| `logs/agent-runs/<slug>-<date>.log` | Full log (headless mode only) |
| `~/.claude/claude-agent-team-metrics.db` | Token usage across all runs |

---

## Review Checklist

```
[ ] Read docs/specs/<slug>/decisions.md if present
[ ] Read docs/specs/<slug>/review-notes.md if present
[ ] git diff main — review the diff
[ ] Run the test suite
[ ] Manually test against success criteria in the spec
[ ] Merge or iterate
```
