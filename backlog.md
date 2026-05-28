# Backlog

> This backlog is maintained by the Planning Agent and Retro Agent. Do not edit manually unless necessary.

---

## Items

### [BLI-001] Auto-Plan Tab
- **Type:** Feature
- **Priority:** High
- **Status:** Ready
- **Source:** Vision gap
- **Spec:** `docs/specs/013-auto-plan-tab.md`
- **Notes:** Adds a Plan tab to the TUI that triggers the `auto-plan` skill (execute now or schedule). Also deploys `skills/auto-plan/SKILL.md`, `agents/architect.md`, and `agents/project-scribe.md` via `install.rs`. All implementation details are fully specified — ready to run.

---

### [BLI-002] Spec Panel Sortable Table with Date Columns
- **Type:** Feature
- **Priority:** Low
- **Status:** Blocked
- **Source:** Vision (v2 milestone)
- **Notes:** Vision calls for the spec panel to be a sortable table with created/completed date columns. Current implementation is a simple list. Blocked on: date metadata (created/completed) is not currently stored in spec frontmatter or any persistent store — a storage/frontmatter convention decision is needed before this can be specced.

---

### [BLI-003] Raw Inputs Tab for Drafter
- **Type:** Feature
- **Priority:** Medium
- **Status:** Blocked
- **Source:** Vision (v2 milestone)
- **Notes:** *MAY BE DEPRECATED BY AUTO-PLAN* A Requirements/Raw Inputs tab in the TUI that surfaces raw files (no valid frontmatter) so the user can select them for a Drafter run. Depends on BLI-002 (Drafter agent) being specced and implemented first.

---

### [BLI-004] Auto-Chain Drafter → Team Run
- **Type:** Feature
- **Priority:** Medium
- **Status:** Blocked
- **Source:** Vision (v2 milestone)
- **Notes:** *MAY BE DEPRECATED BY AUTO-PLAN* When the Drafter produces a `ready` spec, automatically chain to the default team run without user intervention. Depends on BLI-002 and BLI-004.

---

### [BLI-005] Spec Dependency Tracking
- **Type:** Feature
- **Priority:** Low
- **Status:** Blocked
- **Source:** Tech debt / `docs/specs/TODO-spec-dependencies.md`
- **Notes:** Tracked as a known gap. No resolution path or requirements defined. Needs a requirements discussion before it can become a spec.
