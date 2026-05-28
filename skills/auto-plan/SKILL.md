---
name: auto-plan
description: Autonomously draft specs for all Open backlog items. Creates an agent team with a Scribe and parallel Architect teammates — one per Open BLI. Each Architect produces a spec that is either ready or blocked, then the Scribe updates the backlog.
---

You are the team lead for an autonomous planning run. Your job is to stand up a team, assign work, and report results. You do not write specs or touch the backlog yourself — that's what your teammates are for.

---

## Step 1: Preflight

Check that the following files exist at the project root:
- `vision.md`
- `project-state.md`
- `backlog.md`

If any are missing, stop and tell the human which files need to be created first (e.g. "Run `/draft-vision` then `/init` before running `/auto-plan`").

---

## Step 2: Load Backlog

Read `backlog.md`. Collect all BLIs where `Status: Open`.

If no Open items exist, tell the human:

> "No Open backlog items found. Update backlog item statuses to Open before running `/auto-plan`."

And stop.

---

## Step 3: Pre-assign Spec Numbers

Determine the spec directory (check for a `doc-conventions` rule or CLAUDE.md override — default to `docs/specs/`).

List that directory now and find the highest existing sequence number. Assign the next sequential numbers to each Open BLI in backlog order:

- BLI-001 → `003`
- BLI-003 → `004`
- BLI-005 → `005`

These numbers are fixed. Do not re-derive them inside the agents.

---

## Step 4: Create the Team

Create an agent team named `auto-plan`. Tell the human:

> "Starting planning run for: BLI-001 (→ 003), BLI-003 (→ 004), BLI-005 (→ 005)"

Create tasks in the shared task list — one per Open BLI:
- Title: `Spec BLI-NNN`
- One task per BLI, in backlog order

---

## Step 5: Spawn Scribe

Spawn a teammate using the `project-scribe` agent type. Name it `scribe`.

Wait for the scribe to confirm it is ready before proceeding.

---

## Step 6: Spawn Architects

Spawn one teammate per Open BLI **in parallel** using the `architect` agent type. Name each one `architect-NNN` (e.g. `architect-001`).

Assign each architect its corresponding task from the shared task list.

Each architect receives this prompt:

```
Draft a spec for <BLI-ID>.

Context files are at the project root: vision.md, project-state.md, backlog.md.
Explore the codebase as needed to ground your technical approach.
Always produce a spec — ready or blocked.

Write the spec to: <spec-dir>/<NNN>-<slug>.md
Your assigned spec number is <NNN> — do not list the spec directory or derive a number yourself.

When your spec is written, message the scribe teammate with:
  "Update BLI-<ID> status to Specced. Spec written at <path>."
  or if blocked:
  "Update BLI-<ID> status to Blocked. Spec written at <path>."

Then mark your task complete.
```

---

## Step 7: Wait and Report

Wait for all architect tasks to complete.

Once all tasks are done, summarize results to the human:

```
/auto-plan complete

  ✓ BLI-001 → docs/specs/003-slug.md (specced)
  ✓ BLI-003 → docs/specs/004-slug.md (blocked — see spec for details)
  ✓ BLI-005 → docs/specs/005-slug.md (specced)
```

If any architect failed to produce a spec or mark its task complete, call it out:

> "BLI-NNN: architect did not complete. Check that all context files exist and are well-formed."

---

## Step 8: Shut Down

Ask each teammate to shut down gracefully via SendMessage. Once all teammates have shut down, clean up the team.
