# Feature: Drafter

**Status:** In Progress\
**Started:** 2026-03-29\
**Completed:**

---

## Problem

The agent team workflow requires a well-formed spec to execute against. Currently, that spec has to be written manually — there's no path from raw requirements (a client email, rough notes, a bug report) to a ready spec. This creates friction and limits the workflow to users who already know how to write specs.

---

## Proposed Solution

A new **Drafter** agent that takes a raw requirements file, explores the codebase, and produces a spec file in `docs/specs/`. The spec always follows the existing template format. If the Drafter determines the feature is implementable with what it finds, the spec is marked `status: ready` and the harness automatically chains to the default team run. If it hits blockers, the spec is marked `status: blocked` and includes a `## Blockers` section describing what's missing and what needs to happen to proceed. The user can resolve blockers and re-feed the same spec back to the Drafter.

The entry point is a new **Requirements tab** in the TUI. Raw files in the specs directory (files without valid frontmatter) appear in this tab and can be selected to kick off a draft run.

## Integration Points

- `src/config.rs` — `discover_specs` extended to recognize raw files (no frontmatter) as a separate `SpecEntry` kind
- `src/tui/` — new tab in the spec panel; app state tracks active tab
- `src/main.rs` — draft flow branch: run Drafter agent, read resulting spec status, chain to team run if `ready`
- `src/runner.rs` — reused as-is for both the Drafter run and the subsequent team run
- `docs/spec-template.md` — updated with `blocked`/`draft` status values and `## Blockers` section
- `prompts/agents/drafter.md` — new Drafter agent prompt

### Key Behaviors

- Raw files (no frontmatter) discovered in specs dir appear in Requirements tab
- Selecting a raw file and confirming runs the Drafter agent against it
- Drafter writes a new numbered spec to `docs/specs/`
- Harness reads the resulting spec's frontmatter status
- If `ready`: runs preflight and chains to default team run automatically
- If `blocked`: exits cleanly; blocked spec is visible in the Specs tab for review
- Re-feeding a blocked spec to the Drafter is supported (same flow, spec file as input)

---

## Success Criteria

- [ ] Raw files in specs dir appear in a Requirements tab in the TUI
- [ ] Selecting a raw file kicks off a Drafter agent run
- [ ] Drafter produces a valid, numbered spec file
- [ ] A `ready` spec automatically chains to the default team run
- [ ] A `blocked` spec surfaces in the Specs tab with visible blocked status
- [ ] Re-feeding a blocked spec to the Drafter works end-to-end

---

## Scope

### In Scope

- Drafter agent prompt (initial working version — quality iterated separately)
- Raw file discovery and Requirements tab in TUI
- Harness chaining logic (draft → team run)
- Spec template updates (`blocked` status, `## Blockers` section)

### Out of Scope

- Redesign of Team and Run Options panels (deferred)
- File system browser / file picker in TUI
- Drafter prompt quality iteration (happens after wiring is proven)
- Any input format other than files in the specs dir

---

## Important Considerations

- The Team and Run Options panels remain visible but are effectively inactive during a draft run — this is a known gap, deferred to a later panel redesign
- Spec numbering: Drafter determines the next available number by reading the specs directory — this should be specified explicitly in the prompt
- The Drafter runs against the project that invokes the tool, so it needs broad read access to the target codebase

---

## High-Level Todo

- [ ] Update spec template with `blocked` status and `## Blockers` section
- [ ] Extend `discover_specs` to return raw files as a distinct type
- [ ] Add Requirements tab to TUI (app state + rendering)
- [ ] Write initial `prompts/agents/drafter.md`
- [ ] Add draft flow to harness (`main.rs`): run Drafter, read status, chain if ready
- [ ] End-to-end smoke test: raw file → Drafter → blocked or ready → team run

---

## Notes & Context

### 2026-03-29 — Core design decisions

Single output format: the Drafter always writes a spec file. Blocked state is expressed via `status: blocked` frontmatter and a `## Blockers` section in the spec body — not a separate report file. This keeps the workflow uniform and makes re-feeding simple (same file, same agent, same invocation).

Auto-chaining to the team run is handled by the Rust harness, not the agent. The Drafter just writes the spec and exits. The harness reads the status and decides whether to proceed.

### 2026-03-29 — Blocker criteria (open question)

What threshold should the Drafter use to decide between "block and ask" vs. "make a reasonable assumption and document it in Considerations"? Working heuristic: block when the agent would have to choose an architecture the human hasn't implicitly endorsed; assume when there's a reasonable default that follows existing codebase patterns. Needs to be encoded explicitly in the prompt before the Drafter is considered production-quality.

### 2026-03-29 — TUI input approach

Considered a file system browser (no ratatui built-in, would require external crate) and a CLI subcommand. Settled on raw files in the specs dir appearing in a Requirements tab — no new directories, no new CLI surface, extends the existing discovery pattern naturally.
