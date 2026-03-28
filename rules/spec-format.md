---
description: How to write a feature spec for the Claude Code agent team workflow
---

# Agent Team Spec Format

When asked to write or help write a feature spec for the agent team workflow, use the template at `/Users/etwilson/workdev/claude/agent-team-workflow/docs/spec-template.md` as the source of truth.

Specs live at `docs/specs/<feature-slug>.md` in the target project.

## What each section is for

**Requirements** — the Reviewer agent gates failing tests against these. Be specific and behavioral. Vague requirements produce unenforced tests.

**Scope (In/Out)** — the Coder agent uses this as its guardrail. "Out of scope" is as important as "in scope." If it's not stated, the Coder may drift.

**Technical Approach** — integration points, existing code to touch, patterns to follow. Helps the Coder orient without guessing. No implementation code.

**Success Criteria** — testable, observable outcomes. The Reviewer checks that tests cover these. Write them as checkboxes.

**Tasks** — discrete, independently completable units of work. The Lead decomposes these into the TDD task triplet (write tests → review tests → implement). Aim for 3-5 tasks. Too coarse = Lead has to guess; too fine = coordination overhead.

**Considerations** — constraints, edge cases, gotchas. Anything that would cause a silent wrong implementation if the agent didn't know about it.

## What makes a good spec

- Requirements are behavioral, not structural ("returns paginated results" not "has a pagination function")
- Scope explicitly names what we're NOT doing — this is a hard guardrail for the Coder
- Tasks are at the right granularity: a function, a test file, an endpoint — not "implement the whole feature"
- Success criteria are independently verifiable — each one maps to a test
