# Feature: <Name>

## Summary

One paragraph. What does this feature do, what problem does it solve, and how does the user interact with it? Write this as if explaining to someone who hasn't read the requirements.

---

## Requirements

- Each requirement is a single, testable statement
- Written from the user's perspective where possible
- No implementation detail here — what, not how
- Be explicit about edge cases that are in scope

---

## Scope

### In Scope

- Explicit list of what this spec covers
- If it touches multiple layers (storage, UI, API), name them

### Out of Scope

- Explicitly name related features that are NOT being built here
- Reference other specs by slug where applicable (e.g. covered in `other-feature` spec)
- Name anything that might be assumed but isn't included

---

## Technical Approach

- **Entry points / interfaces:** Where does this feature start? What invokes it?
- **Key modules / components:** What files or classes own what responsibility?
- **Data model:** What are the shapes of data involved?
- **Key design decisions:** Document the choices made and briefly why

Write enough here that the agent team doesn't need to make architectural decisions. The more explicit, the more reliably the Lead decomposes tasks and the Coder implements against intent.

---

## Success Criteria

- [ ] Criterion written as an observable, verifiable outcome
- [ ] Each criterion should map to at least one task below
- [ ] Avoid vague criteria like "works correctly" — be specific about what correct looks like
- [ ] Include at least one persistence or integration criterion if applicable

---

## Tasks

Ordered by dependency. Each task should be a self-contained unit of work with a clear deliverable.

- [ ] **Task name:** Description of what gets built. Name the files or modules involved. Note if it must be fully unit-tested before the next task can begin.
- [ ] **Task name:** Description. Note any dependency on a previous task explicitly.
- [ ] **Task name:** Description.

Aim for 3–5 tasks. Too few means tasks are too large for reliable agent checkpoints. Too many means coordination overhead dominates.

> **If this spec produces a runnable binary or entry point:** include an explicit final task that writes a smoke/integration test verifying the binary runs end-to-end and produces observable output. The agent team follows TDD strictly — if `main()` wiring has no test, it will not be written. A passing `cargo test` (or equivalent) is not sufficient evidence that the pieces are connected.

---

## Considerations

- Edge cases the agent team should be aware of but that aren't obvious from the requirements
- Known gotchas in the codebase or framework that are relevant here
- Constraints that affect implementation choices (e.g. "must not introduce a dependency on X")
- Anything that would cause a reasonable engineer to make a wrong assumption
