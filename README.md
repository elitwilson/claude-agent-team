# Agent Team Workflow

A workflow for autonomously implementing features overnight using Claude Code Agent Teams. A structured spec goes in, a GitLab Merge Request comes out.

```
[Spec Doc] → [Pre-flight] → [Agent Team] → [MR] → [Morning Review]
```

The agent team consists of three roles: a **Lead** that coordinates, a **Coder** that implements via TDD, and a **Reviewer** that gates tests before implementation begins.

---

## Prerequisites

- macOS
- Claude Code v2.1.32 or later (`claude --version`)
- Claude Pro account with OAuth token (optional — falls back to default auth)
- GitLab hosted repository (for MR creation)
- Python 3.12+

---

## Installation

Clone this repo, then run the install script from the project root:

```bash
git clone <this-repo> ~/workdev/claude/agent-team-workflow
cd ~/workdev/claude/agent-team-workflow
python install.py
```

This will:
- Symlink `rules/` into `~/.claude/rules/agent-workflow` (live — edits here take effect immediately)
- Register agent team hooks in `~/.claude/settings.json`

Then add the scripts to your PATH in `~/.zshrc`:

```bash
export PATH="/path/to/agent-team-workflow/scripts:$PATH"
```

### OAuth token (optional)

To use a specific Claude account for overnight runs, store your OAuth token in macOS Keychain:

```bash
security add-generic-password -s "claude-token-1" -a "claude" -w "<your-token>"
```

If no token is found, the workflow falls back to your default Claude Code authentication.

---

## Usage

### 1. Write a spec

Create a feature spec in your target project at `docs/specs/<feature-slug>.md`. See [Spec Format](#spec-format) below.

### 2. Run manually (recommended before scheduling)

From within your target project directory:

```bash
run-agent-team.py docs/specs/my-feature.md
```

This runs pre-flight checks, invokes the agent team, and creates a GitLab MR when complete.

### 3. Schedule overnight (cron)

```bash
crontab -e
```

```
# Run at 11pm every weekday
0 23 * * 1-5 cd /path/to/project && run-agent-team.py docs/specs/my-feature.md >> /path/to/project/logs/agent-runs/cron.log 2>&1
```

---

## Spec Format

> Full spec format design is a work in progress. At minimum, a spec must include:

- A title (`# Feature Name`) as the first line — used as the MR title
- A feature summary
- Requirements
- Technical design / architecture notes
- Discrete, dependency-ordered tasks
- Acceptance criteria / definition of done

The Lead agent reads the spec and decomposes it into tasks for the team. The more explicit the task breakdown in the spec, the more reliably the Lead decomposes it.

---

## Project Structure

```
agent-team-workflow/
├── install.py                  # Global installation into ~/.claude/
├── agent-workflow.md           # Full workflow design document
│
├── scripts/
│   ├── preflight.py            # Git checks and branch creation
│   └── run-agent-team.py       # Main entry point — invokes the full pipeline
│
├── prompts/
│   └── orchestration.md        # Lead agent initialization prompt (template)
│
├── docs/
│   ├── agent-roles.md          # Index of role definitions
│   └── roles/
│       ├── lead.md             # Lead: coordinates, never writes code
│       ├── coder.md            # Coder: TDD implementation
│       └── reviewer.md         # Reviewer: test review gate, one pass per task
│
├── hooks/                      # Claude Code agent team hooks (stubs — future enforcement)
│   ├── task-completed.sh
│   ├── task-created.sh
│   └── teammate-idle.sh
│
└── rules/                      # Symlinked into ~/.claude/rules/agent-workflow/
```

---

## How It Works

1. **Pre-flight** — validates the spec exists, confirms a clean git working tree, pulls latest, creates a feature branch (`feature/<slug>-<YYYYMMDD>`)
2. **Agent team** — Lead reads the spec and role definitions, spawns Coder and Reviewer, coordinates a TDD loop per task
3. **TDD loop** — Coder writes failing tests → Reviewer gates → Coder implements → commit → repeat
4. **MR creation** — branch is pushed and a GitLab MR is opened automatically with run metadata

### Runtime artifacts (written to target project)

| File | Purpose |
|------|---------|
| `docs/specs/<slug>-decisions.md` | Ambiguities and assumptions the Lead made during the run |
| `docs/specs/<slug>-review-notes.md` | Reviewer gate outcomes per task |
| `logs/agent-runs/<slug>-<date>.log` | Full terminal log of the agent run |

---

## Morning Review Checklist

```
[ ] Check GitLab for new MR
[ ] Read docs/specs/<slug>-decisions.md if present
[ ] Read docs/specs/<slug>-review-notes.md if present
[ ] git diff main -- review the diff
[ ] Run the test suite
[ ] Manually test against acceptance criteria
[ ] Merge or iterate
```

---

## Iterating on This Workflow

This repo is the source of truth. To update the global installation after making changes:

- **Prompt or role changes** — take effect immediately (scripts read files at runtime)
- **Rules changes** — take effect immediately (symlinked)
- **Hook changes** — take effect immediately (hooks registered by absolute path)
- **`install.py` changes** — re-run `python install.py` (idempotent)

---

## Open Questions / Future Work

See the [Open Questions section](agent-workflow.md#open-questions--future-improvements) in the workflow design doc.
