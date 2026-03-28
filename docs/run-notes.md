# Agent Team Run Notes

A record of notable runs — outcomes, root causes, and what changed as a result. Used to identify patterns and improve the workflow over time.

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
