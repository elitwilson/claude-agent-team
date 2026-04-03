---
number: ToDo
status: blocked
base_branch: main
---

# Spec Dependencies — Notes / Pre-planning

Rough idea: add a `depends_on` frontmatter field to spec documents so that the system can enforce ordering — don't schedule or run spec `003` if spec `002` is required first and isn't complete.

---

## The Core Idea

Add optional frontmatter to specs:

```yaml
---
depends_on:
  - "002-metrics-collection"
---
```

This makes dependencies explicit and machine-readable, living directly in the spec file alongside the rest of its metadata.

---

## Open Questions / Things to Think Through

**What counts as "complete"?**
- Does a spec need `status: complete` in its frontmatter?
- Or do we track completion state elsewhere (e.g., the metrics DB)?

**How strict should enforcement be?**
- Hard block: refuse to schedule/run if deps aren't met
- Soft warning: allow it but warn the user
- Informational only: just display deps, no enforcement

**Dependency resolution**
- What if deps have their own deps? Do we need transitive resolution?
- What about circular deps? Probably rare but worth catching.

**Multiple deps**
- `depends_on` should support a list, not just a single value
- Do ALL deps need to be complete, or just ANY?

---

## Potential TUI Implications

These are just possibilities — nothing decided yet:

- **New column in the spec table** showing dep status (e.g., blocked/ready)
- **Visual indicator** on rows that are blocked by incomplete deps
- **Pre-execution check** that warns or blocks if deps aren't met before running
- **Pre-scheduling check** same but at schedule time
- **Dependency view** — maybe a way to see a spec's full dep chain

The table column might get noisy if most specs have no deps. Could be opt-in or only shown when deps exist.

---

## Notes

- This is similar to how CI pipelines handle job dependencies (needs: in GitHub Actions)
- Keep it simple first — a flat list of dep slugs is probably enough before adding DAG resolution
- The `depends_on` field should be optional; specs without it behave exactly as today
