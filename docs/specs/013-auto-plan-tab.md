---
number: 013
status: complete
base_branch: main
---

# Feature: Auto-Plan Tab

## Summary

Adds a Plan tab to the TUI launcher that lets the user trigger or schedule an autonomous spec-drafting run against the current project. The run invokes the `auto-plan` skill, which reads `vision.md`, `project-state.md`, and `backlog.md`, fans out parallel architect agents, and writes ready-or-blocked specs for all Open backlog items. This feature also moves the `auto-plan` skill and the `architect`/`project-scribe` agent definitions into this repo (as the new source of truth) and extends `install.rs` to deploy them to `~/.claude/skills/` and `~/.claude/agents/` so they are also available for direct `/auto-plan` invocation in any Claude Code session.

---

## Requirements

- `skills/auto-plan/SKILL.md` and `agents/architect.md`, `agents/project-scribe.md` exist in the repo and are the canonical source for these files
- On first run, `install.rs` copies the skill to `~/.claude/skills/auto-plan/SKILL.md` and the agent definitions to `~/.claude/agents/architect.md` and `~/.claude/agents/project-scribe.md` (idempotent)
- `/auto-plan` works as a Claude Code slash command in any project after install (skill is at the expected discovery path)
- The TUI launcher shows a Plan tab alongside Specs and Requirements
- Pressing Enter on the Plan tab opens an ActionDialog (Execute Now / Schedule Later) — no team picker, no spec picker
- If multi-account is configured, the existing AccountDialog appears after ActionDialog confirmation, same as the Specs tab flow
- Execute Now runs the auto-plan skill immediately (headless or interactive per Options pref) with no git preflight
- Schedule Later opens the existing SchedulePicker and registers a launchd plist with `--mode auto-plan`
- If a Plan run is already scheduled, Enter opens a CancelDialog identical in behavior to the Specs tab cancel flow
- The scheduled `run` subcommand accepts `--mode auto-plan`; when set, `--spec` and `--team` are not required and preflight is skipped entirely
- If `~/.claude/skills/auto-plan/SKILL.md` does not exist at execution time, the run fails with a clear error message directing the user to re-run install

---

## Scope

### In Scope
- `skills/` and `agents/` top-level directories in repo with initial file content
- `prompt.rs` embedding and extraction of `skills/` and `agents/` to `~/.claude-launch/`
- `install.rs` install functions for skills and agents (copy from `~/.claude-launch/` to `~/.claude/`)
- `SpecTab::Plan` variant and all App state changes to support it
- `RunMode::AutoPlan` variant wired through TUI result, main.rs execution, and run_cmd.rs
- Plist generation for auto-plan mode (no spec/team args, `--mode auto-plan` flag)
- `run_scheduled` branching for AutoPlan (skip preflight, load skill as prompt)

### Out of Scope
- Modifying the content of `SKILL.md`, `architect.md`, or `project-scribe.md` beyond copying them from the dotfiles source (content iteration is separate work)
- Auto-draft skill or any skill other than auto-plan
- Any Rust-side validation of `vision.md`, `project-state.md`, or `backlog.md` — the skill handles this
- `is_installed()` check extension to verify skills/agents presence (existing check for rules symlink is sufficient for now)
- Updating the README

---

## Technical Approach

- **Entry points:** `install.rs::run_install()` gains two new steps. TUI flow: `app.rs::confirm()` on `SpecTab::Plan` opens `PopupAction::ActionDialog` directly (bypasses `open_team_popup`). `main.rs::run()` branches on `TuiResult::mode == RunMode::AutoPlan`. `main.rs::run_scheduled()` branches on `RunArgs::mode == Some(RunMode::AutoPlan)`.

- **New repo directories:**
  - `skills/auto-plan/SKILL.md` — copy content from `~/mydev/dotfiles/claude/.claude/skills/auto-plan/SKILL.md`
  - `agents/architect.md` — copy content from `~/mydev/dotfiles/claude/.claude/agents/architect.md`
  - `agents/project-scribe.md` — copy content from `~/mydev/dotfiles/claude/.claude/agents/project-scribe.md`

- **`prompt.rs`:** Add two new `include_dir!` statics: `static SKILLS_FILES: Dir = include_dir!("$CARGO_MANIFEST_DIR/skills")` and `static AGENTS_FILES: Dir = include_dir!("$CARGO_MANIFEST_DIR/agents")`. `resolve_workflow_dir()` extracts both to `~/.claude-launch/skills/` and `~/.claude-launch/agents/` alongside the existing extractions.

- **`install.rs`:** Add `install_skills(workflow_dir, claude_dir)` — copies `<workflow_dir>/skills/auto-plan/SKILL.md` to `~/.claude/skills/auto-plan/SKILL.md`, creating the directory if needed. Add `install_agents(workflow_dir, claude_dir)` — copies `<workflow_dir>/agents/*.md` to `~/.claude/agents/`. Both are idempotent (overwrite existing). `run_install()` calls both after the existing steps.

- **`app.rs`:**
  - `SpecTab` gains `Plan` variant: `{ Specs, Requirements, Plan }`
  - `switch_tab()` cycles Specs → Requirements → Plan → Specs
  - `App::confirm()` on `SpecTab::Plan`: opens `PopupAction::ActionDialog { selected: ActionChoice::ExecuteNow }` directly
  - `App::result()` on `SpecTab::Plan`: returns `TuiResult { spec: String::new(), team: String::new(), headless: self.prefs.headless, mode: RunMode::AutoPlan, account }`
  - Plan tab pending run uses the fixed slug `"auto-plan"` in the existing `run_info: HashMap<String, SpecRunInfo>` — no new state needed
  - `cancel_dialog` and `confirm_cancel_dialog` already key on slug + plist_path lookup and require no changes

- **`run_cmd.rs`:** `RunArgs` gains `mode: Option<String>` field parsed from `--mode <value>`. `--spec` and `--team` become optional (return `None` instead of erroring when absent) unless `mode` is not `auto-plan`.

- **`scheduler.rs` / `generate_plist_xml`:** When `spec` is `"auto-plan"`, omit `--spec` and `--team` from ProgramArguments, add `--mode auto-plan`. The plist label becomes `com.claude-launch.auto-plan`. `plist_path_for_spec("auto-plan")` produces the correct path without changes.

- **`main.rs`:**
  - `run()`: when `selection.mode == RunMode::AutoPlan`, load prompt from `~/.claude/skills/auto-plan/SKILL.md` (fail fast with clear error if missing), skip preflight, skip team lookup, call `runner::run_claude` with no branch creation
  - `run_scheduled()`: when `run_args.mode == Some("auto-plan")`, skip spec hash check, skip `run_preflight`, load prompt from `~/.claude/skills/auto-plan/SKILL.md`, call `runner::run_claude`
  - `TuiResult` confirm_picker path: when `active_tab == Plan`, use `"auto-plan"` as the slug passed to `scheduler::schedule_run`; `schedule_run` receives spec `"auto-plan"` which routes through the modified plist generator

- **`ui.rs`:** Plan tab renders a single line: `"[Enter] Auto-plan — draft specs for all Open backlog items"`. Footer hint for Plan tab: `"Enter: run/schedule  t: switch tab  q: quit"`. No list, no cursor.

---

## Success Criteria

- [ ] `skills/auto-plan/SKILL.md`, `agents/architect.md`, `agents/project-scribe.md` exist in repo with content from dotfiles sources
- [ ] After `install.rs` runs, `~/.claude/skills/auto-plan/SKILL.md`, `~/.claude/agents/architect.md`, `~/.claude/agents/project-scribe.md` exist on disk
- [ ] Install is idempotent — running it twice does not error
- [ ] TUI shows three tabs: Specs, Requirements, Plan; `t` cycles between them
- [ ] Pressing Enter on Plan tab opens ActionDialog (Execute Now / Schedule Later) with no team picker
- [ ] Execute Now on Plan tab runs `claude` with the auto-plan skill content as prompt, no git preflight
- [ ] Schedule Later on Plan tab registers a launchd plist with `--mode auto-plan` (no `--spec`/`--team`)
- [ ] `claude-launch run --mode auto-plan --headless --cleanup-plist <path>` executes the auto-plan skill without preflight
- [ ] If a Plan run is pending, Enter on Plan tab shows CancelDialog; confirming removes the plist
- [ ] If `~/.claude/skills/auto-plan/SKILL.md` is missing at run time, a clear error is printed and the process exits non-zero

---

## Tasks

- [ ] **Add skills/ and agents/ directories:** Create `skills/auto-plan/SKILL.md`, `agents/architect.md`, `agents/project-scribe.md` by copying content from `~/mydev/dotfiles/claude/.claude/skills/auto-plan/SKILL.md`, `~/mydev/dotfiles/claude/.claude/agents/architect.md`, and `~/mydev/dotfiles/claude/.claude/agents/project-scribe.md` respectively. Must be fully in place before the next task — `include_dir!` compile-time embedding will fail otherwise.

- [ ] **Extend prompt.rs and install.rs:** Add `SKILLS_FILES` and `AGENTS_FILES` static embedded dirs in `prompt.rs`. Extend `resolve_workflow_dir()` to extract both. Add `install_skills` and `install_agents` to `install.rs` and call them from `run_install()`. Write unit tests for both new install functions (idempotency, directory creation). Depends on previous task.

- [ ] **Add RunMode::AutoPlan and SpecTab::Plan to app.rs:** Add `Plan` to `SpecTab`; update `switch_tab()` to cycle three-way. Update `confirm()` to open ActionDialog directly on Plan tab. Update `result()` to return AutoPlan mode with empty spec/team. Update move_up/move_down to no-op on Plan tab. Write tests covering tab cycling, confirm routing, and result shape for Plan tab. Depends on previous task.

- [ ] **Update run_cmd.rs and scheduler.rs for --mode auto-plan:** Make `--spec` and `--team` optional in `RunArgs`; add `mode: Option<String>` field parsed from `--mode`. Update `generate_plist_xml` (or add a branch) to emit auto-plan args when spec is `"auto-plan"`. Write tests for parse round-trip and plist generation for auto-plan mode. Depends on previous task.

- [ ] **Wire auto-plan execution in main.rs and ui.rs:** In `run()`, branch on `RunMode::AutoPlan` to skip preflight and load skill prompt from `~/.claude/skills/auto-plan/SKILL.md`. In `run_scheduled()`, branch on `mode == "auto-plan"` to skip spec hash check, skip preflight, load skill prompt. Update `ui.rs` Plan tab render and footer hints. Write an integration test that calls `run_scheduled` with `--mode auto-plan` against a temp dir containing the skill file and verifies it reaches `runner::run_claude` (can mock the claude binary or assert on the rendered prompt string without actually spawning). Depends on all previous tasks.

---

## Considerations

- `include_dir!` is a compile-time macro — the `skills/` and `agents/` directories must exist before the first `cargo build` after this change. The agent team must create those directories in the first task before any Rust changes.
- The existing spec hash integrity check in `run_scheduled` must be skipped entirely for auto-plan (there is no spec file to hash). Guard the hash check on `run_args.mode != Some("auto-plan")` rather than on presence of `--spec-hash`.
- `schedule_run()` currently takes `spec` and `team` as required `&str` params and builds the plist from them. For auto-plan, pass `spec = "auto-plan"` and `team = ""` (or add an explicit auto-plan variant) — the plist generator branches on the spec value. Whichever approach is chosen, the existing scheduled-run tests must continue to pass.
- `confirm_picker` in app.rs uses `visible_specs()[self.spec_index]` to get the spec name. For Plan tab scheduling, it should use `"auto-plan"` as the slug directly — guard on `active_tab == SpecTab::Plan` before indexing visible_specs.
- The `move_up` / `move_down` handlers currently match on `focused_panel` then `active_tab`. Plan tab has no list; both should be no-ops when `active_tab == Plan`.
- `is_installed()` only checks for the rules symlink. This is intentional — do not change the install detection logic in this spec.
- After install, `~/.claude/skills/auto-plan/SKILL.md` is a copy, not a symlink. The user can edit it independently of the repo. Re-running install overwrites it with the repo version — this is acceptable and consistent with how hooks are handled.
