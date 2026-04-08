## Task 1: TeamEntry + discover_teams() rewrite

**Verdict: APPROVED**

### Requirements checklist vs. tests

| Requirement | Test(s) | Status |
|---|---|---|
| `discover_teams()` three-source signature | all tests | covered |
| Missing user dir silently skipped | `test_discover_teams_missing_user_dir_silently_skipped` | covered |
| Configured-but-missing project dir returns error | `test_discover_teams_configured_project_dir_missing_errors` | covered |
| Error message names the conflicting team(s) | `test_discover_teams_collision_*`, `test_discover_teams_collision_error_lists_all_conflicting_names` | covered |
| Collisions across all three source pairs (B/U, B/P, U/P) | three dedicated collision tests | covered |
| Entries sorted by name | `test_discover_teams_clean_merge_all_sources_sorted` | covered |
| Source assigned correctly per origin | `test_discover_teams_builtin_only`, `test_discover_teams_clean_merge_all_sources_sorted` | covered |
| Path stored correctly (absolute) | `test_discover_teams_entry_path_is_absolute`, `test_discover_teams_builtin_only` | covered |
| Only `.md` files included | `test_discover_teams_skips_non_md_files` | covered |
| Empty dirs return empty result | `test_discover_teams_returns_empty_for_empty_dirs` | covered |

### Notes

- `test_discover_teams_missing_builtin_dir_errors` is not explicitly required by the spec but is consistent with it (builtin dir is never optional) and is not a misalignment.
- All tests assert on observable behavior (return type, error message content, entry fields) — no implementation detail testing detected.
- No spec requirement for Task 1 is left without coverage.

---

## Task 2: Config custom_dir field

**Verdict: APPROVED**

### Requirements checklist vs. tests

| Requirement | Test(s) | Status |
|---|---|---|
| `custom_dir: Option<String>` field exists on `Config` | all three tests (field access would fail to compile otherwise) | covered |
| Absent from TOML defaults to `None` | `test_load_custom_dir_absent_defaults_to_none` | covered |
| Present in TOML parses as `Some(String)` | `test_load_custom_dir_parses_when_present` | covered |
| No-config-file case defaults to `None` | `test_load_default_config_has_no_custom_dir` | covered |

### Notes

- Runtime resolution relative to project root (spec requirement) is a `main.rs` concern (Task 4), not a `Config` struct concern. No test is required here for that.
- All three tests assert on observable behavior via `Config::load()` return value — no implementation detail testing detected.
- No spec requirement for Task 2 is left without coverage.

---

## Task 3: render_prompt() + user dir creation

**Verdict: FLAGGED**

### Requirements checklist vs. tests

| Requirement | Test(s) | Status |
|---|---|---|
| New `user_dir` and `project_dir` params on `render_prompt()` | All tests pass 7 args; existing tests updated | covered |
| `${USER_DIR}` substituted with `user_dir` value | `test_render_prompt_substitutes_user_dir` | covered |
| `${PROJECT_DIR}` substituted with `project_dir` value | `test_render_prompt_substitutes_project_dir` | covered |
| Empty `project_dir` substitutes `${PROJECT_DIR}` to empty string | `test_render_prompt_project_dir_empty_string_substitutes_nothing` | covered |
| `resolve_workflow_dir()` creates `user/teams/` | `test_resolve_workflow_dir_creates_user_teams_dir` | covered |
| `resolve_workflow_dir()` creates `user/agents/` | `test_resolve_workflow_dir_creates_user_teams_dir` (asserts both dirs) | covered |
| `run_install()` creates `user/teams/` and `user/agents/` | none | **MISSING** |
| Existing call sites updated to pass new params | implicit — compile-time enforcement | covered |

### Gap

The spec explicitly assigns to Task 3: "In `src/install.rs` `run_install()`, do the same two directories explicitly so they exist after first install." There are no tests for this requirement. The Solo Dev acknowledged the omission but called it out of scope — however, the spec does not draw that boundary. A failing test for `run_install()` creating the two user dirs is required before Task 3 can be considered complete.

### Notes

- The three `render_prompt` substitution tests assert on observable output — no implementation detail issues.
- The `resolve_workflow_dir` test correctly asserts both dirs in a single test — no issue with that approach.
- All other Task 3 requirements are covered.
