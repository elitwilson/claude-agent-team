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
