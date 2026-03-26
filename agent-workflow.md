# Overnight Claude Code Agent Teams Workflow

A spec for autonomously implementing features after hours using Claude Code Agent Teams, triggered via cron, reviewed in the morning via GitLab Merge Requests.

---

## Overview

The workflow takes a structured feature spec document as input, spins up a Claude Code Agent Team to implement it autonomously overnight, and produces a GitLab Merge Request for human review in the morning.

```
[Spec Doc] → [Pre-flight] → [Cron Trigger] → [Agent Team Execution] → [MR Creation] → [Morning Review]
```

---

## Assumptions & Prerequisites

- macOS development machine
- GitLab hosted repository
- Claude Code installed and authenticated
- Two Claude Pro accounts ($20/mo each) with OAuth tokens available
- `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` enabled
- Feature spec document already written and committed to `docs/specs/`

---

## Step 1: Spec Input

**Responsibility:** Define where specs live and how agents consume them.

**Convention:**
```
docs/specs/
  └── <feature-slug>.md   ← one spec per feature
```

**Spec doc must include** (exact structure TBD in spec format design):
- Feature summary
- Requirements
- Technical design / architecture notes
- Discrete, dependency-ordered tasks
- Acceptance criteria / definition of done

**How agents consume it:**
The orchestration prompt references the spec via Claude Code's `@` file syntax:
```
implement @docs/specs/<feature-slug>.md
```

---

## Step 2: Pre-flight Checks

**Responsibility:** Ensure the environment is clean and ready before agents touch anything.

**Steps (run as part of trigger script):**

```bash
#!/bin/bash
set -e

SPEC_FILE=$1
FEATURE_SLUG=$(basename "$SPEC_FILE" .md)
BASE_BRANCH="main"  # or develop, configure as needed

# 1. Confirm spec file exists
if [ ! -f "$SPEC_FILE" ]; then
  echo "ERROR: Spec file not found: $SPEC_FILE"
  exit 1
fi

# 2. Confirm git working tree is clean
if [ -n "$(git status --porcelain)" ]; then
  echo "ERROR: Working tree is dirty. Commit or stash changes first."
  exit 1
fi

# 3. Pull latest from base branch
git checkout "$BASE_BRANCH"
git pull origin "$BASE_BRANCH"

# 4. Create and checkout feature branch
BRANCH_NAME="feature/${FEATURE_SLUG}-$(date +%Y%m%d)"
git checkout -b "$BRANCH_NAME"

echo "Pre-flight passed. Branch: $BRANCH_NAME"
echo "$BRANCH_NAME" > /tmp/current-agent-branch
```

---

## Step 3: Orchestration Trigger

**Responsibility:** Invoke Claude Code non-interactively with the right context.

**Cron entry** (`crontab -e`):
```bash
# Run at 11pm every weekday
0 23 * * 1-5 /path/to/scripts/run-agent-team.sh docs/specs/my-feature.md >> /path/to/logs/agent-run-$(date +\%Y\%m\%d).log 2>&1
```

**Trigger script** (`scripts/run-agent-team.sh`):
```bash
#!/bin/bash
set -e

SPEC_FILE=$1
FEATURE_SLUG=$(basename "$SPEC_FILE" .md)
LOG_DIR="logs/agent-runs"
mkdir -p "$LOG_DIR"

# Load OAuth token for account selection
# Toggle between accounts to distribute rate limit usage
export CLAUDE_OAUTH_TOKEN="<your-token-here>"  # or load from secure store
export CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1

# Run pre-flight
source scripts/preflight.sh "$SPEC_FILE"
BRANCH_NAME=$(cat /tmp/current-agent-branch)

# Build the orchestration prompt
PROMPT="You are the Lead agent. Your job is to coordinate, not to write code.

Read the feature spec at @${SPEC_FILE} and implement it using an agent team as follows:

## Your team
- **Coder** (claude-sonnet-4-6): owns all implementation. Follows strict TDD — writes failing tests first, implements against them, iterates until green. Owns backend and frontend.
- **Reviewer** (claude-sonnet-4-6): reviews Coder's failing tests before implementation begins. One pass, critical issues only.

## Task lifecycle
Break the spec into discrete tasks (aim for 5-6 per teammate). Assign tasks explicitly to teammates. Teammates self-claim next available unblocked task when they finish. Tasks have dependencies — order them correctly so Reviewer gates happen before Coder goes GREEN.

## TDD flow per feature task
1. Coder writes failing tests (RED) and signals Reviewer
2. Reviewer reads spec + failing tests — flags CRITICAL issues only (tests that don't match spec requirements, tests testing implementation logic rather than behavior, obvious misdirection). One pass. One message back to Coder if issues found.
3. Coder fixes if needed, goes GREEN, refactors, marks task complete
4. Repeat for next task

## Reviewer guardrails
Reviewer does NOT flag: code style, naming, edge cases not in spec, anything not a spec violation.
Reviewer writes outcome to docs/specs/${FEATURE_SLUG}-review-notes.md regardless of result.
Reviewer gets one pass per task. If issues remain after Coder's fix cycle, they go in review-notes.md for human review — Reviewer does not re-engage.

## Rules
- Do not write code yourself. Delegate everything to teammates.
- Do not modify files outside the scope of this spec.
- If blocked on a decision, log it to docs/specs/${FEATURE_SLUG}-decisions.md with your assumption and proceed. Never halt.
- Teammates commit after each completed task.
- Teammates must not push to remote.
- When all tasks are complete: shut down teammates, clean up the team, then signal done.
- Max turns: 200"

# Invoke Claude Code non-interactively
claude --print \
  --max-turns 200 \
  "$PROMPT" \
  >> "${LOG_DIR}/${FEATURE_SLUG}-$(date +%Y%m%d).log" 2>&1

echo "Agent run complete. Branch: $BRANCH_NAME"
```

---

## Step 4: Agent Team Configuration

**Responsibility:** Define the shape of the agent team and how roles map to the spec.

**Configuration approach:** Defined in the orchestration prompt (Step 3), not a separate config file. The lead agent reads the spec and determines how many teammates to spawn and what to assign each.

### Defining teammates
There is currently no native way to pre-define agent team members as reusable config files. Teammates are spawned as general-purpose agents — the only way to specialize them is through the natural language prompt the Lead writes at spawn time. (An open GitHub issue requests `.claude/agents/` integration with agent teams, but it doesn't exist yet.)

This means role definitions — Coder guardrails, Reviewer constraints, TDD rules — have to live somewhere accessible to the Lead at runtime. Three options:

| Option | Approach | Tradeoff |
|--------|----------|----------|
| Inline in orchestration prompt | Embed full role definitions in the bash script | Works, but verbose and hard to maintain |
| In CLAUDE.md | Lead reads roles from CLAUDE.md automatically | Clean, but adds to CLAUDE.md token cost for every session — not just agent team runs |
| Separate roles file | Lead reads `docs/agent-roles.md` at spawn time | Lean prompt, lean CLAUDE.md, roles are versioned and reusable across specs |

**Recommended approach: `docs/agent-roles.md`**

Keep role definitions in a dedicated file in the repo. The orchestration prompt tells the Lead to read it before spawning teammates:
```
Before spawning teammates, read @docs/agent-roles.md for role definitions and guardrails.
Spawn a Coder and a Reviewer per those definitions.
```

This keeps the orchestration prompt lean, keeps CLAUDE.md lean, and makes roles independently editable and version-controlled without touching the trigger script.

### Team size
Start with 3 teammates max. Token usage scales linearly with team size — 3 teammates costs roughly 3-4x a single session. Beyond 4-5 teammates, coordination overhead starts eating the parallelism gains.

### Role pattern

All roles run claude-sonnet-4-6. Three contexts total — lean by design.

| Role | Responsibility |
|------|---------------|
| **Lead** | Reads spec, breaks into tasks, assigns work, stays in delegate mode (does not write code), handles git and MR at the end |
| **Coder** | Owns all implementation. Follows strict TDD: writes failing tests first, implements against them, iterates until green. Owns backend and frontend — no split. |
| **Reviewer** | After Coder signals done, reads the spec and the diff independently. Checks implementation against acceptance criteria. Flags issues to `docs/specs/<feature-slug>-review-notes.md`. Sends one coarse-grained feedback message to Coder. |

**Full TDD flow:**
```
Lead assigns tasks to Coder

Coder: writes failing tests (RED)
  → signals Reviewer

Reviewer: reads spec + failing tests (one pass, critical issues only)
  → "approved, proceed" OR one message flagging critical issues

Coder: fixes if flagged → goes GREEN → refactors → signals Lead done

Lead: synthesizes, commits, pushes, creates MR
```

**Reviewer guardrails — critical issues only:**
The Reviewer's job is narrow and explicitly bounded. It is NOT a general code reviewer. It checks for one thing: do the tests faithfully represent what the spec requires?

Flag only if:
- Tests don't cover requirements stated in the spec
- Tests are testing implementation logic rather than observable behavior
- Obvious misdirection — testing irrelevant edge cases not in the spec, testing things that don't matter

Do NOT flag:
- Code style or formatting
- Minor test naming issues
- Edge cases the Reviewer thinks would be nice to have
- Anything not explicitly a spec violation

**Hard limits in prompt:**
- One review pass. No second opinions.
- One fix cycle from Coder. If issues remain after that, they go to `review-notes.md` for human review in the morning — Reviewer does not re-engage.
- Reviewer writes outcome (approved or flagged + notes) to `docs/specs/<feature-slug>-review-notes.md` regardless.

The goal is catching genuinely wrong tests before wasting tokens on a GREEN phase that implements against bad requirements. Not perfection.

### Task sizing
Each teammate should have 5-6 discrete tasks. Too small = coordination overhead exceeds benefit. Too large = agent works too long without checkpoints, risk of wasted effort if it goes sideways. Tasks should be self-contained units with a clear deliverable — a function, a test file, a component.

> **Context overflow is an accepted risk.** No matter how carefully tasks are sized, a Coder agent will eventually hit its context limit mid-task. When it does, auto-compact kicks in (lossy but better than halting), or the agent halts entirely. Both are detectable in morning review via incomplete commits, failing tests, or weird diffs. Treat it as signal to resize that task smaller next time. Task sizing is calibrated iteratively — you won't get it right immediately and that's fine.

### Token cost: CLAUDE.md and rules

Every teammate loads CLAUDE.md automatically. Keep it lean — everything in CLAUDE.md is multiplied by team size.

Your `.claude/rules/*.md` files behave as follows:
- Rules **without** `paths:` frontmatter load for every teammate, every time. Same multiplier problem as CLAUDE.md.
- Rules **with** `paths:` frontmatter (e.g. `paths: src/api/**/*.ts`) only load when an agent is working on matching files. This is the right approach for agent teams — path-scoped rules mean the backend agent doesn't pay for frontend rules and vice versa.

**Action:** Audit your rules before running agent teams. Add `paths:` frontmatter to any rule that isn't truly universal.

### Pre-approve permissions
Unattended agents will halt if they hit an unapproved permission prompt with nobody at the keyboard. Before running overnight, pre-approve common operations in Claude Code's permission settings — file writes, git commands, test execution, anything the agents will routinely need.

### Rate limit strategy (two Pro accounts)
- Run primary agents under Account 1 token
- If hitting limits mid-run, re-invoke under Account 2 token
- Manual today — token rotation automation is a future improvement

---

## Step 5: Branch Strategy

**Responsibility:** Isolate agent work so each run is reviewable and mergeable independently.

**Convention:**
```
main (or develop)
 └── feature/<feature-slug>-<YYYYMMDD>
```

One branch per overnight run. Agents commit incrementally as tasks complete. Branch is created in pre-flight (Step 2) before agents start.

**Commit convention agents should follow** (include in orchestration prompt):
```
feat(<area>): <short description>

- Task: <task name from spec>
- Agent: <agent role>
```

---

## Step 6: Execution / Runtime

**Responsibility:** The agents doing the actual work. This is largely Claude's domain, but guardrails matter.

**Guardrails to include in orchestration prompt:**
- `--max-turns 200` flag on CLI invocation (hard ceiling)
- Agents must not modify files outside the spec's stated scope
- Agents commit after each discrete task (not one giant commit at the end)
- Agents must not push to remote — push happens in Step 8

### Task lifecycle
Tasks have three states: `pending`, `in_progress`, `completed`. Tasks can declare dependencies — a task with unresolved dependencies cannot be claimed until those are done. This is how the Reviewer gate is enforced structurally: the GREEN task depends on the Reviewer task completing first.

Teammates self-claim the next available unblocked task when they finish their current one. File locking prevents race conditions. The Lead can also assign tasks explicitly.

**Termination:** When all tasks are completed, the Lead shuts down teammates one by one (teammates can approve or reject shutdown), then runs team cleanup. Cleanup fails if any teammates are still running — so shutdown order matters. This is the natural termination condition for the overnight run.

### Hooks (quality gates)
Claude Code exposes three hook points for agent teams that can enforce rules without relying on prompt instructions alone:

| Hook | Fires when | Use case |
|------|-----------|----------|
| `TeammateIdle` | A teammate is about to go idle | Keep teammate working if tasks remain |
| `TaskCreated` | A task is being created | Reject malformed or out-of-scope tasks |
| `TaskCompleted` | A task is being marked complete | Block completion if quality bar not met |

**Most relevant for this workflow:** `TaskCompleted` — can be used to enforce that Reviewer has written to `review-notes.md` before a task is allowed to complete, providing a hard gate rather than relying purely on prompt instructions. Hooks exit with code `2` to block and send feedback, or `0` to allow.

> Hook implementation for the Reviewer gate is a future improvement — documenting here because it's the right long-term solution to enforcing the review cycle without prompt-only guardrails.

**Runtime artifacts agents produce:**
- Code changes (committed to feature branch)
- `docs/specs/<feature-slug>-decisions.md` — log of ambiguities and assumptions made
- `docs/specs/<feature-slug>-review-notes.md` — Reviewer outcome per task
- Terminal log captured to `logs/agent-runs/`

---

## Step 7: Error / Stuck Handling

**Responsibility:** Define what happens when agents hit a wall, rather than silently hanging or burning tokens.

**Strategy: Log and assume, don't halt.**

Include in orchestration prompt:
```
If you encounter ambiguity or a blocking decision:
1. Log it to docs/specs/<feature-slug>-decisions.md with your assumption
2. Proceed with the assumption
3. Never halt waiting for human input
```

**What to watch for in morning log review:**
- Exit code of the `claude` invocation (non-zero = something went wrong)
- Presence of `decisions.md` (indicates assumptions were made — review carefully)
- Incomplete tasks (check git log for commits — are all task areas represented?)

**Future improvement:** Add a post-run health check script that scans the log for error patterns and sends a notification (email, Slack, etc.).

---

## Step 8: MR Creation

**Responsibility:** Push the feature branch and open a GitLab Merge Request automatically after agent run completes.

**Script** (appended to `run-agent-team.sh` after claude invocation):
```bash
# Push feature branch to remote
git push origin "$BRANCH_NAME"

# Create GitLab MR via push options
SPEC_TITLE=$(head -1 "$SPEC_FILE" | sed 's/# //')
git push origin "$BRANCH_NAME" \
  -o merge_request.create \
  -o merge_request.target="$BASE_BRANCH" \
  -o merge_request.title="feat: ${SPEC_TITLE} (agent run $(date +%Y-%m-%d))" \
  -o merge_request.description="Automated implementation by Claude Code Agent Teams.

**Spec:** \`${SPEC_FILE}\`
**Branch:** \`${BRANCH_NAME}\`
**Run date:** $(date)

Review \`docs/specs/${FEATURE_SLUG}-decisions.md\` for assumptions made during the run.
Review \`docs/specs/${FEATURE_SLUG}-review-notes.md\` for Reviewer gate outcomes.

Log: \`logs/agent-runs/${FEATURE_SLUG}-$(date +%Y%m%d).log\`" \
  -o merge_request.remove_source_branch

echo "MR created for branch: $BRANCH_NAME"
```

---

## Step 9: Morning Review (Human Step)

**Responsibility:** You. Review what the agents did, verify it works, and merge if satisfied.

**Checklist:**

```
[ ] Check GitLab for new MR
[ ] Review MR description for run metadata
[ ] Read docs/specs/<feature-slug>-decisions.md if present (assumptions made)
[ ] Read docs/specs/<feature-slug>-review-notes.md if present (Reviewer gate outcomes)
[ ] git checkout feature/<feature-slug>-<date>
[ ] git diff main -- review the diff
[ ] Start local dev environment
[ ] Manually test the feature against acceptance criteria in spec
[ ] Run test suite
[ ] If satisfied: merge MR into main/develop
[ ] If not satisfied: leave MR comments, re-run or iterate manually
```

---

---

## Appendix: Metrics & Observability (Post-MVP)

Not part of the POC. But as a data-oriented workflow, blind iteration is bad. Future work should instrument this pipeline so runs produce evidence, not just vibes.

**Quantitative signals worth capturing per run (automatable from logs):**
- Exit code, turns used vs max, wall clock time
- Whether auto-compact fired and how many times
- Number of commits, files touched vs files in spec scope (scope creep detection)
- Whether `decisions.md` and `review-notes.md` were written, and how many entries

**Qualitative signals captured manually in the morning:**
- Merge outcome: `merged_clean` / `merged_with_fixes` / `rejected_rerun` / `rejected_manual`
- Estimated manual intervention time
- Free-text notes

**The metric that actually matters:** merge rate with minimal intervention. Everything else is a proxy for that.

A simple `logs/agent-runs/<feature-slug>-metrics.json` per run is enough to build a dataset. After 10+ runs patterns will emerge — which spec structures produce clean runs, what turn counts correlate with good outcomes, whether Reviewer flags predict morning rejections.
---

## Appendix: Security Considerations (Pre-Production Checklist)

Not blockers for prototyping, but things to nail down before running this on anything real.

**Blast radius** — Agents run as you and can touch your entire filesystem, CLI tools, SSH keys, and env vars. Eventually: run agents as a dedicated OS user with scoped access. For now: don't run on a machine with production credentials in the environment.

**Credential storage** — Don't put OAuth tokens in plaintext scripts. Load from macOS Keychain: `security find-generic-password -w -s "claude-token-1" -a "claude"`. Never commit tokens to the repo.

**Runaway cost control** — `--max-turns 200` helps but isn't a guarantee. Consider a watchdog cron that kills any `claude` process still running 2 hours after the agent cron fires.

**Prompt injection** — If spec docs reference external URLs or pull in web content, a malicious page could inject instructions. Keep specs self-contained. Disable web search in Claude Code settings for overnight runs.

**Git safety** — Enable GitLab branch protection on `main`/`develop` so agents can't push there regardless. Eventually: use a scoped deploy key with push access to `feature/*` only, not your personal credentials.

**Scope creep** — Agents may helpfully modify files outside the spec. Lock down allowed write paths in Claude Code settings (`src/`, `tests/`, `docs/specs/`). The morning diff review is your last line of defense.

---

## Open Questions / Future Improvements

- [ ] Hooks: implement `TaskCompleted` hook to enforce Reviewer gate as a hard quality gate rather than prompt-only instruction
- [ ] Permissions: replace `--dangerously-skip-permissions` with a curated set of pre-approved operations (file writes, git commands, test execution). See Claude Code permission settings. Reduces blast radius for unattended runs.
- [ ] Rate limit rotation: script automatic token switching between two Pro accounts mid-run
- [ ] Notification: post-run ping (email or Slack) with MR link and run summary
- [ ] Health check script: parse log for errors, flag to reviewer before morning
- [ ] Multi-spec runs: queue multiple specs to run sequentially overnight
- [ ] Spec validation: pre-flight check that spec doc has required sections before invoking agents