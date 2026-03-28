---
number: 004
status: complete
---

# Feature: Spec Status and Numbering Convention

## Summary

Specs accumulate over time and become historical documents once implemented. This feature introduces a `status` frontmatter field to each spec so the TUI can filter them intelligently — showing only specs that are actionable — and establishes a sequential 00N numbering convention so specs tell a chronological story of the project. The team lead is responsible for updating spec status at the end of each run.

---

## Requirements

- Each spec file must be prefixed with a three-digit zero-padded number (e.g. `001-claude-bros.md`) that reflects the order it was created. Numbers are never reused.
- Each spec must include a `status` field in its YAML frontmatter with one of three values: `ready`, `complete`, or `needs_attention`.
- The TUI spec picker shows specs with status `ready` or `needs_attention`. Specs with status `complete` are hidden.
- `needs_attention` specs are visually distinguished in the TUI (yellow text).
- Specs with missing or unrecognized status are treated as `ready` (safe default).
- `needs_attention` specs are assignable to teams — the TUI user decides whether to re-run them.
- At the end of every agent run, the team lead updates the spec's `status` frontmatter: `complete` if all tasks finished successfully, `needs_attention` if any tasks did not.
- The spec template is updated to include the status frontmatter field and documents the numbering convention.
- Existing specs are renamed with their assigned numbers and marked `complete`.

---

## Scope

### In Scope

- Adding `status` frontmatter to all existing and future specs
- Renaming existing spec files with 00N prefix
- TUI filtering and yellow highlight for `needs_attention`
- Lead role instructions updated with final status-update step
- Spec template updated with frontmatter and numbering guidance

### Out of Scope

- Programmatic status updates by `claude-bros` itself (the lead handles this)
- Any status beyond `ready`, `complete`, `needs_attention`
- Enforcing numbering via tooling — convention is documented, not automated
- Custom `specs_dir` configurations — numbering convention applies to the default location only

---

## Technical Approach

- **Spec discovery:** `config::discover_specs` currently globs `docs/specs/*.md`. It needs to parse YAML frontmatter from each file and filter out specs where `status == "complete"`. Use the `gray_matter` or `yaml-front-matter` crate, or a simple manual parser since the frontmatter structure is minimal (just strip the `---` block and parse with `serde_yaml`).
- **TUI rendering:** The spec list in the TUI is rendered in `src/tui/`. Specs with `status: needs_attention` should render in yellow. Use `ratatui`'s `Style` with `Color::Yellow` for those items.
- **Lead role:** `docs/roles/feature-dev/lead.md` gets a new final behavior: after all tasks are complete, update the spec's `status` frontmatter to `complete` or `needs_attention` based on the outcome of the run.
- **Spec template:** `docs/spec-template.md` gets a frontmatter block at the top documenting `number` and `status` fields, plus a prose note on the numbering convention.
- **Existing specs:** Rename and mark complete as part of implementation.

**Assigned numbers for existing specs:**
- `001-claude-bros.md`
- `002-metrics-collection.md`
- `003-metrics-query.md`

---

## Success Criteria

- [ ] Running `claude-bros` with all three existing specs present shows zero specs in the picker (all are `complete`)
- [ ] A new spec with `status: ready` appears in the picker
- [ ] A spec with `status: needs_attention` appears in the picker in yellow
- [ ] A spec with no status frontmatter appears in the picker (treated as `ready`)
- [ ] A spec with `status: complete` does not appear in the picker
- [ ] The spec template includes frontmatter with `number` and `status` fields and documents the 00N convention

---

## Tasks

- [ ] **Rename and mark existing specs:** Rename `claude-bros.md` → `001-claude-bros.md`, `metrics-collection.md` → `002-metrics-collection.md`, `metrics-query.md` → `003-metrics-query.md`. Add frontmatter with `number` and `status: complete` to each.
- [ ] **Frontmatter parsing in spec discovery:** Update `config::discover_specs` to parse the `status` field from each spec's frontmatter. Filter out `complete` specs. Treat missing/unrecognized status as `ready`. Unit test the parsing and filtering logic.
- [ ] **Yellow highlight in TUI:** Update the spec list renderer in `src/tui/` to apply yellow styling to `needs_attention` specs. The spec item must carry its status through to the render layer.
- [ ] **Update lead role instructions:** Add a final step to `docs/roles/feature-dev/lead.md`: after all tasks finish, update the spec file's `status` frontmatter to `complete` or `needs_attention` based on run outcome.
- [ ] **Update spec template:** Add a frontmatter block to `docs/spec-template.md` with `number` and `status` fields. Add a note documenting the 00N sequential numbering convention and that numbers are never reused.

---

## Considerations

- The frontmatter parser must be tolerant — specs without frontmatter should not crash discovery, just default to `ready`.
- The `gray_matter` crate is the cleanest option for YAML frontmatter parsing in Rust. Check if it's already a dependency before adding a new one.
- `needs_attention` uses an underscore to be YAML-identifier-friendly. Make sure the lead role instructions and template use this exact string consistently.
- The existing spec files have no frontmatter today — the rename + frontmatter addition task must be done first so the filtering logic has real data to work against during development.
