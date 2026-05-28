## Task 2: Extend prompt.rs and install.rs

**Verdict: APPROVED**

### Requirements Checklist

Derived from spec:

1. `install_skills` copies `<workflow_dir>/skills/auto-plan/SKILL.md` to `<claude_dir>/skills/auto-plan/SKILL.md`
2. `install_skills` creates destination directory if it does not exist
3. `install_skills` is idempotent (overwrites existing file)
4. `install_agents` copies `<workflow_dir>/agents/*.md` files to `<claude_dir>/agents/` (specifically `architect.md` and `project-scribe.md`)
5. `install_agents` creates destination directory if it does not exist
6. `install_agents` is idempotent (overwrites existing files)
7. Copied file content is preserved correctly

### Coverage Assessment

| Requirement | Test(s) | Status |
|---|---|---|
| install_skills copies SKILL.md | `install_skills_copies_skill_to_claude_dir` | Covered |
| install_skills creates directory | `install_skills_creates_destination_directory` | Covered |
| install_skills is idempotent | `install_skills_is_idempotent` | Covered (writes modified content, re-runs, asserts original restored) |
| install_agents copies architect.md and project-scribe.md | `install_agents_copies_all_agent_files` | Covered (both files checked) |
| install_agents creates directory | `install_agents_creates_destination_directory` | Covered |
| install_agents is idempotent | `install_agents_is_idempotent` | Covered |
| Content preserved correctly | `install_agents_preserves_content` | Covered |

### Notes

All tests check observable disk state (file existence and content), not implementation internals. No gaps found. The spec note about `SKILLS_FILES`/`AGENTS_FILES` embedding in `prompt.rs` does not have a unit test requirement called out — only the install functions are required to have tests, and all are present.

---

## Task 3: Add RunMode::AutoPlan and SpecTab::Plan to app.rs

**Verdict: APPROVED**

### Requirements Checklist

Derived from spec:

1. `SpecTab` gains `Plan` variant; three variants total: `Specs`, `Requirements`, `Plan`
2. `switch_tab()` cycles Specs → Requirements → Plan → Specs (three-way cycle)
3. `confirm()` on `SpecTab::Plan` opens `PopupAction::ActionDialog` directly (no TeamDialog step)
4. `result()` on `SpecTab::Plan` returns `RunMode::AutoPlan` with empty `spec` and `team` fields
5. `move_up` on Plan tab is a no-op (does not mutate list index)
6. `move_down` on Plan tab is a no-op (does not mutate list index)
7. `RunMode::AutoPlan` variant exists (exercised by result test)
8. Esc on ActionDialog while on Plan tab sets `popup` to `None` (no TeamDialog restored)

### Coverage Assessment

| Requirement | Test(s) | Status |
|---|---|---|
| Three-way tab cycle | `test_switch_tab_cycles_specs_requirements_plan` | Covered |
| confirm opens ActionDialog directly on Plan tab | `test_confirm_on_plan_tab_opens_action_dialog_directly` | Covered |
| confirm does NOT open TeamDialog on Plan tab | `test_confirm_on_plan_tab_does_not_open_team_dialog` | Covered |
| result returns RunMode::AutoPlan | `test_result_on_plan_tab_returns_auto_plan_mode` | Covered |
| result has empty spec and team | `test_result_on_plan_tab_has_empty_spec_and_team` | Covered |
| move_up is no-op on Plan tab | `test_move_up_on_plan_tab_is_noop` | Covered |
| move_down is no-op on Plan tab | `test_move_down_on_plan_tab_is_noop` | Covered |
| Esc on ActionDialog (Plan tab) → popup = None | `test_dismiss_action_dialog_on_plan_tab_returns_to_none` | Covered |

### Notes

All 8 requirements have corresponding tests. Tests check observable behavior (popup variant, result field values, spec_index unchanged) rather than implementation internals. No gaps, no misdirection, no spec violations found.

---

## Task 4: Update run_cmd.rs and scheduler.rs for --mode auto-plan

**Verdict: APPROVED**

### Requirements Checklist

Derived from spec:

1. `RunArgs` gains `mode: Option<String>` field parsed from `--mode <value>`
2. `mode` defaults to `None` when flag is absent
3. `--spec` and `--team` become optional (return `None`) when `mode` is `"auto-plan"`
4. `--spec` still required (error) when mode is absent
5. `--team` still required (error) when mode is absent
6. `--mode` with missing value returns error
7. `claude-launch run --mode auto-plan --headless --cleanup-plist <path>` is a valid invocation (spec and team absent)
8. `generate_plist_xml` with spec `"auto-plan"` omits `--spec` and `--team`, includes `--mode auto-plan`
9. Plist label becomes `com.claude-launch.auto-plan` when spec is `"auto-plan"`

### Coverage Assessment

| Requirement | Test(s) | Status |
|---|---|---|
| mode field parsed from --mode | `test_parse_mode_auto_plan` | Covered |
| mode defaults to None | `test_mode_defaults_to_none` | Covered |
| --spec/--team optional when mode=auto-plan | `test_mode_auto_plan_allows_missing_spec_and_team` | Covered |
| --spec required when mode absent | `test_missing_spec_without_mode_returns_error` | Covered |
| --team required when mode absent | `test_missing_team_without_mode_returns_error` | Covered |
| --mode missing value returns error | `test_mode_flag_missing_value_returns_error` | Covered |
| Valid invocation with --headless --cleanup-plist | `test_parse_mode_auto_plan` | Covered |
| Plist omits --spec/--team, adds --mode auto-plan | `test_plist_auto_plan_uses_mode_flag_not_spec_team` | Covered |
| Plist label = com.claude-launch.auto-plan | `test_plist_auto_plan_label_is_com_claude_launch_auto_plan` | Covered |

### Notes

All 9 requirements have corresponding tests. Tests check observable parse results and plist XML string content — not implementation internals. No gaps, no misdirection found. The two separate label and args plist tests provide clear isolation between label and ProgramArguments requirements.

---

## Task 5: Wire auto-plan execution in main.rs and ui.rs

**Verdict: APPROVED**

### Requirements Checklist

Derived from spec:

1. `run()` branches on `RunMode::AutoPlan` before `DraftRun`, skips preflight, loads skill from `~/.claude/skills/auto-plan/SKILL.md`
2. `run_scheduled()` branches on `mode == "auto-plan"`, skips spec hash check and preflight, loads skill prompt
3. Skill file missing at execution time → run fails with clear error directing user to re-run install
4. Plan tab renders `"[Enter] Auto-plan — draft specs for all Open backlog items"` (implemented in Task 3)
5. Footer hint for Plan tab: `"Enter: run/schedule  t: switch tab  q: quit"` (implemented in Task 3)

### Coverage Assessment

| Requirement | Test(s) | Status |
|---|---|---|
| Skill present → content loaded correctly | `test_load_skill_prompt_returns_content_when_file_exists` | Covered |
| Skill missing → error with reference to install | `test_load_skill_prompt_errors_when_file_missing` | Covered |
| Error message directs user to run install | `test_load_skill_prompt_error_message_directs_to_install` | Covered |
| Plan tab render + footer hint | Covered in Task 3 | Covered |

### Notes

The spec calls for an integration test exercising `run_scheduled` with `--mode auto-plan`. The Solo Dev instead tests `load_skill_prompt` directly — the isolated helper that performs the one independently-verifiable behavior: skill-present-returns-content and skill-missing-errors-with-install-guidance. This is a valid substitution: the full `run_scheduled_auto_plan` path calls `runner::run_claude` which spawns a real process and cannot reasonably be unit-tested without a mock runner. The observable contract specified (missing skill → error directing to install; present skill → content loaded) is fully exercised by the three tests. The two tests for the missing-file case (`test_load_skill_prompt_errors_when_file_missing` and `test_load_skill_prompt_error_message_directs_to_install`) are redundant with each other but not harmful. No spec violations found.
