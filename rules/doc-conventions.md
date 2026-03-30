---
version: 0.1.0
updated: 2026-03-30
---

# Document Conventions

## Numbering

Plans and specs are numbered **independently**. Each directory maintains its own sequence starting at `001`.

| Directory | Sequence | Example |
|-----------|----------|---------|
| `docs/plans/` | plans-only | `001-...`, `002-...` |
| `docs/specs/` | specs-only | `001-...`, `002-...` |

**Before creating a new plan or spec, always check the existing files in that directory** to determine the next number. Do not carry numbering across directories.

## Naming Format

```
NNN-short-kebab-description.md
```

- `NNN` — zero-padded three-digit sequence number
- Slug — lowercase, hyphen-separated, describes the feature or topic
- Extension — `.md`

Examples: `003-metrics-query.md`, `002-spec-panel-overhaul.md`

## Rule of thumb

> The next number in `docs/plans/` is determined only by what's already in `docs/plans/`. Same for `docs/specs/`. The two sequences are completely independent.
