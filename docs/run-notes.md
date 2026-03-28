# Agent Team Run Notes

A record of notable runs — outcomes, root causes, and what changed as a result. Used to identify patterns and improve the workflow over time.

---

## 2026-03-27 — metrics-query (feature-dev team)

**Spec:** `docs/specs/metrics-query.md`
**Team:** feature-dev
**Outcome:** Complete — 15/15 tasks, 94 tests passing, all reviews approved

### What happened

Clean run. All five areas implemented and approved without escalation. Smoke test task was included this time — the main.rs wiring lesson from the previous run paid off immediately. Two minor decisions logged to `docs/specs/metrics-query/decisions.md` (LEFT JOIN for zero-token runs, SUBSTR for date formatting) — both defensive, low-risk choices.

**Incident:** The Reviewer crashed mid-run after ~104k tokens, likely context exhaustion from accumulating review context across multiple tasks. Human intervention was required to prompt the Lead to spawn a replacement Reviewer. The replacement completed the remaining 4 reviews without issue.

### Root cause (Reviewer crash)

The Reviewer's context window fills up across a multi-task run because it holds the full conversation history of every review pass. Unlike the Coder (who commits and starts relatively fresh on each task), the Reviewer accumulates spec analysis, expected test lists, coder tests, and review decisions for every task it completes. On a 5-task spec this can hit ~100k+ tokens.

### Key insight

The Lead may not be able to reliably detect a crashed teammate — "unresponsive" and "slow" look the same. Human monitoring is the current mitigation. This is an accepted failure mode for now.

Spawning a fresh Reviewer per task would eliminate the context exhaustion risk but multiplies the token cost (spec reload, expected list regeneration per task). Not worth it at current scale.

### What to change

- Reviewer role now explicitly says to go idle after preparing expected lists — this reduces unnecessary turns and may reduce context growth slightly.
- No structural change for now. Monitor whether context exhaustion recurs on longer specs.

---

## 2026-03-27 — claude-bros (feature-dev team)

**Spec:** `docs/specs/claude-bros.md`
**Team:** feature-dev
**Outcome:** Incomplete — binary not wired up

### What happened

65 tests passing across all modules. Every module implemented correctly in isolation. `main.rs` was never wired up — still a hello world with module declarations. The binary does nothing.

### Root cause

The spec did not include an explicit task for wiring `main.rs` or a smoke/integration test that would have forced it to exist. The task descriptions for run-pipeline and mr+summary mentioned "wire together in main.rs" as a line item, but without a failing test demanding it, the Coder had no forcing function to actually connect the pieces. `cargo test` passed — so the Coder was technically correct.

### Key insight

**This is the TDD guardrails working exactly as intended.** The team did not make assumptions or write untested code. They implemented precisely what the tests required and nothing more. This is the right behavior — tight, disciplined, no freelancing.

The failure was in the spec, not the team. A spec that produces a runnable binary must include an explicit wiring task with a testable deliverable (a smoke/integration test that verifies the binary runs end-to-end). Without that test, the wiring has no forcing function and will not get written.

### What to change

- Specs that produce a binary need an explicit final task: write a smoke/integration test that runs the binary and verifies observable output. This forces `main.rs` wiring to exist.
- Add a note to the spec template Tasks section flagging this requirement.
- The Lead's termination checklist should include verifying `cargo run` (or equivalent) produces expected output, not just that `cargo test` passes.

### Follow-up

All the pieces exist. `main.rs` wiring is the only remaining work — a focused follow-up spec or manual wiring session.
