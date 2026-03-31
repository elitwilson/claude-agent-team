# Agent Role Definitions

Role definitions live in `docs/roles/<team-type>/`. Load only what you need.

## feature-dev

- `docs/roles/feature-dev/lead.md` — coordination, task breakdown, git/MR, never writes code
- `docs/roles/feature-dev/coder.md` — TDD implementation, owns all code
- `docs/roles/feature-dev/reviewer.md` — test review gate, critical issues only, one pass per task

## investigation

- `docs/roles/investigation/coordinator.md` — decomposes the brief, spawns parallel investigators, synthesizes the final report
- `docs/roles/investigation/investigator.md` — read-only codebase explorer, answers a specific sub-question, returns findings

## solo-with-subagent-review

- `docs/roles/solo-with-subagent-review/solo-dev.md` — owns everything: task breakdown, TDD cycle, commits, git ops
- `docs/roles/solo-with-subagent-review/reviewer.md` — one-shot sub-agent, reviews failing tests against spec, writes verdict to review-notes.md
