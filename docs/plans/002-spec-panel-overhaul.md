# Feature: Spec Panel Overhaul

**Status:** Complete\
**Started:** 2026-03-30\
**Completed:** 2026-03-30

---

## Problem

The spec panel has three gaps:
1. Complete specs are silently filtered out at discovery time — you can't see what's done
2. `NeedsAttention` and `Blocked` are redundant statuses with no practical distinction
3. Run-related toggles (headless) live in a panel called "Run Options" even though filters will also live there — the name is wrong
4. Filter state (which statuses to show) resets every time the app launches

---

## Proposed Solution

Consolidate `NeedsAttention` into `Blocked`, surface all spec statuses in the panel with color coding, add `show_complete` / `show_blocked` filter toggles to the Options panel, and persist user preferences to `~/.claude/claude-agent-team-prefs.toml` so state survives across launches.

## Integration Points

- `src/config.rs` — `SpecStatus` enum, `parse_frontmatter_status`, `discover_specs`
- `src/tui/app.rs` — `App` state, `Panel` enum, key handlers
- `src/tui/ui.rs` — render logic, footer hints, color mapping
- New `src/prefs.rs` — `Prefs` struct with load/save

### Key Behaviors

- `discover_specs` returns all statuses including Complete (filtering moves to the TUI layer)
- `needs_attention` frontmatter value maps to `Blocked` for backwards compatibility
- Complete and Blocked specs are visible but not confirmable (Enter is a no-op)
- Filter toggles live in the Options panel; arrow keys navigate between items, Space toggles
- `show_complete` and `show_blocked` both default to `true` (show all on first run)
- Prefs are written immediately on each toggle — crash-safe
- The Options panel is renamed from "Run Options" throughout

---

## Success Criteria

- [x] Complete specs appear in the list colored green
- [x] Blocked specs appear red; NeedsAttention no longer exists as a status
- [x] `needs_attention` in frontmatter is treated as `Blocked` (backwards compat)
- [x] Toggling show_complete hides/shows Complete specs in the list
- [x] Toggling show_blocked hides/shows Blocked specs in the list
- [x] Arrow keys move between Options panel items; Space toggles the focused one
- [x] Prefs persist to `~/.claude/claude-agent-team-prefs.toml` and reload on next launch
- [x] Panel title reads "Options" everywhere (no more "Run Options")
- [x] Enter on a Complete or Blocked spec does nothing
- [x] Footer shows contextual hints per focused panel (Spec / Team / Options each have distinct hints)
- [x] All existing tests pass; new behavior is covered by tests (140 passing)

---

## Scope

### In Scope

- `SpecStatus` consolidation (remove `NeedsAttention`, map to `Blocked`)
- Surface Complete specs with green color
- `show_complete` / `show_blocked` filter toggles in Options panel
- Persistent prefs via `~/.claude/claude-agent-team-prefs.toml`
- Options panel rename
- Contextual footer hints

### Out of Scope

- Spec panel as a table with date columns (separate feature)
- Filtering on the Raw Inputs tab
- Any changes to how teams or metrics are displayed

---

## Important Considerations

- `discover_specs` currently hard-filters Complete — removing that filter changes what callers receive. The app currently shows a "All specs complete — nothing to run" empty state message; this path still needs to work (when the filtered list is empty after applying show_complete=false and show_blocked=false)
- The `spec_index` cursor could become out-of-bounds after a filter toggle if the list shrinks. Need to clamp to `len.saturating_sub(1)` after any filter change
- Prefs load is non-fatal: missing file or parse error silently falls back to defaults
- Prefs save is non-fatal: failure should log a warning, not crash

---

## High-Level Todo

- [x] 1. Remove `NeedsAttention` from `SpecStatus`; map `needs_attention` frontmatter → `Blocked`; remove Complete filter from `discover_specs`
- [x] 2. Add `src/prefs.rs` with `Prefs` struct (headless, show_complete, show_blocked), load/save
- [x] 3. Update `App`: rename `Panel::RunOptions` → `Panel::Options`; add `options_index` cursor; wire prefs into initial state; call save on each toggle
- [x] 4. Update `ui.rs`: Options panel renders as a mini-list with cursor highlight; footer is contextual per focused panel; Complete specs render green
- [x] 5. Update filter logic: `App` exposes a `visible_specs()` method that applies show_complete/show_blocked; clamp cursor after toggle
- [x] 6. Tests

---

## Key Design Decisions

### `App::new` takes `Prefs` as a parameter

Rather than loading prefs inside `App::new`, prefs are passed in from the call site (`run_tui`). This keeps tests fully isolated — if prefs were loaded internally, a developer's real `~/.claude/claude-agent-team-prefs.toml` could silently affect test behaviour. The tradeoff is a slightly noisier constructor, but there's only one real call site so it's contained.

### `visible_specs()` is a method on `App`, not a filtered field

Filtering is applied at read time rather than maintaining a separate filtered `Vec`. This means `spec_index` always indexes the filtered view, and no synchronisation is needed between the backing store and a derived list. The cost is a small allocation per call; at this scale that's irrelevant. The benefit is that toggling a filter is a one-liner with no cache invalidation.

### Options panel height is hardcoded

The layout constraint is `Constraint::Length(5)` — 3 items + 2 border rows. It is not derived from `OPTIONS_ITEMS`. If options are ever added, both the constant and the layout constraint need updating. This was a deliberate simplicity choice over premature abstraction.

### `toggle_headless()` kept as a wrapper with a side effect

The method sets `options_index = 0` then calls `toggle_option()`. It's no longer called by the UI (space now calls `toggle_option()` directly), but it's kept public for the existing test. The side effect of moving the cursor is surprising — consider removing it in a follow-on cleanup once no callers remain.

---

## Notes & Context

### 2026-03-30 — Status consolidation

`NeedsAttention` was never surfaced differently from `Blocked` in any meaningful way (both were "don't run this"). Collapsing them simplifies the enum and removes a distinction users didn't care about. The `needs_attention` frontmatter string is preserved as a parse alias for Blocked to avoid breaking any existing spec files.

### 2026-03-30 — Prefs location

UI preferences go to `~/.claude/claude-agent-team-prefs.toml`, not into `.claude-agent-team.toml`. The project config is version-controlled and team-shared; personal UI preferences are not. The `~/.claude/` directory is already used for the metrics DB so it's a natural home.

### 2026-03-30 — Filter defaults

Defaulting both filters to `true` (show all) rather than hiding Complete by default. The list is scrollable so showing everything is not a UX problem, and it gives users the full picture on first launch.

---

## Reference

- Existing prefs pattern in metrics: `~/.claude/claude-agent-team-metrics.db`
- Scroll clamping precedent: `MetricsState::scroll_down` visible_rows fix
- Follow-on: spec panel as sortable table with created/completed date columns
