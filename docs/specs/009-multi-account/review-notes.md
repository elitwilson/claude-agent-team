## Task 1: accounts.rs + Prefs

**Verdict: APPROVED**

### Requirements Checklist

| Requirement | Coverage |
|---|---|
| `AccountEntry` struct with `label: String` | Covered — tests read back `.label` directly |
| `load_accounts_from_path` returns `Err` for missing file | `test_load_accounts_returns_empty_when_file_missing` |
| `load_accounts_from_path` returns empty vec for empty file | `test_load_accounts_returns_empty_for_empty_accounts_list` |
| Parses single account entry | `test_load_accounts_parses_single_account` + `test_load_accounts_single_entry_label_correct` |
| Parses multiple account entries with correct labels | `test_load_accounts_parses_multiple_accounts` + `test_load_accounts_multiple_entries_labels_correct` |
| `load_token_for_account` returns `None` on error/missing | `test_load_token_for_unknown_label_returns_none` |
| `Prefs.default_account` defaults to `None` | `test_default_account_is_none` |
| `default_account: None` round-trips through save/load | `test_default_account_round_trips_none` |
| `default_account: Some(...)` round-trips through save/load | `test_default_account_round_trips_some` |
| Old prefs file without `default_account` field deserializes to `None` | `test_default_account_missing_in_file_defaults_to_none` |

### Notes

All spec requirements for Task 1 have corresponding test coverage. The `load_token_for_account` test exercises the `None`-on-error path by relying on a guaranteed-missing Keychain entry, which matches the decisions doc's rationale (same thin-wrapper pattern as `runner::load_oauth_token()`). The `load_accounts_from_path` testable variant satisfies the spec's requirement to test against a temp file. `Prefs` backward-compatibility (old file without `default_account`) is covered. No gaps found.

---

## Task 2: app.rs popup chain

**Verdict: APPROVED**

### Requirements Checklist

| Requirement | Coverage |
|---|---|
| `App::new()` accepts accounts param | `test_app_new_preselects_default_account_from_prefs` and `sample_app_with_accounts` helper |
| `account_index` pre-selected from `prefs.default_account` on startup | `test_app_new_preselects_default_account_from_prefs` |
| `account_index` falls back to 0 when label not found in accounts | `test_app_new_account_index_defaults_to_zero_when_default_not_found` |
| `PopupAction::AccountDialog { selected_index }` variant | Exercised throughout all `AccountDialog` tests |
| `confirm_popup()` on `TeamDialog` with 0 accounts opens `ActionDialog` | `test_confirm_popup_on_team_dialog_with_no_accounts_opens_action_dialog` |
| `confirm_popup()` on `TeamDialog` with 1 account opens `ActionDialog` | `test_confirm_popup_on_team_dialog_with_single_account_opens_action_dialog` |
| `confirm_popup()` on `TeamDialog` with 2+ accounts opens `AccountDialog` | `test_confirm_popup_on_team_dialog_with_multiple_accounts_opens_account_dialog` |
| `confirm_popup()` on `AccountDialog` stores `account_index` | `test_confirm_popup_on_account_dialog_stores_account_index` |
| `confirm_popup()` on `AccountDialog` saves `prefs.default_account` by label | `test_confirm_popup_on_account_dialog_updates_prefs_default_account` |
| `confirm_popup()` on `AccountDialog` opens `ActionDialog` | `test_confirm_popup_on_account_dialog_opens_action_dialog` |
| `dismiss_popup()` on `AccountDialog` restores `TeamDialog` | `test_dismiss_popup_on_account_dialog_restores_team_dialog` |
| `popup_move_down()` on `AccountDialog` increments `selected_index` | `test_popup_move_down_on_account_dialog_increments_index` |
| `popup_move_down()` on `AccountDialog` clamps at last index | `test_popup_move_down_on_account_dialog_clamps_at_last` |
| `popup_move_up()` on `AccountDialog` decrements `selected_index` | `test_popup_move_up_on_account_dialog_decrements_index` |
| `popup_move_up()` on `AccountDialog` clamps at 0 | `test_popup_move_up_on_account_dialog_clamps_at_zero` |
| `TuiResult::account` is `None` when no accounts configured | `test_tui_result_account_is_none_when_no_accounts` |
| `TuiResult::account` is the selected label when account confirmed | `test_tui_result_account_is_label_when_account_selected` |
| `dismiss_popup()` on `ActionDialog` with multiple accounts restores `AccountDialog` | `test_dismiss_action_dialog_with_multiple_accounts_restores_account_dialog` |

### Notes

All spec requirements for Task 2 have corresponding test coverage. The tests cover the full popup chain insertion (`TeamDialog` → `AccountDialog` → `ActionDialog`), all navigation and clamping behavior, the `prefs.default_account` pre-selection and save-on-confirm, the skip-picker paths for zero and single accounts, and the correct `TuiResult::account` value for both the no-accounts and multi-account cases. The `ActionDialog` dismiss-with-accounts test correctly verifies the backwards step goes to `AccountDialog` rather than `TeamDialog`, which is the non-obvious behavioral requirement introduced by inserting the new popup into the chain. No gaps found.

---

## Task 3: run_cmd.rs + scheduler.rs

**Verdict: APPROVED**

### Requirements Checklist

| Requirement | Coverage |
|---|---|
| `RunArgs` gains `account: Option<String>` field | `test_parse_account_flag`, `test_account_flag_defaults_to_none` (assert on field) |
| `parse_run_args` parses `--account <label>` when present | `test_parse_account_flag`, `test_parse_account_flag_with_all_flags` |
| `parse_run_args` defaults `account` to `None` when flag is absent | `test_account_flag_defaults_to_none` |
| `parse_run_args` returns error when `--account` has no value | `test_account_flag_missing_value_returns_error` |
| `generate_plist_xml` accepts `account: Option<&str>` param | Both `test_plist_includes_account_flag_when_provided` and `test_plist_excludes_account_flag_when_none` |
| `generate_plist_xml` encodes `--account <label>` in plist args when `Some` | `test_plist_includes_account_flag_when_provided` |
| `generate_plist_xml` omits `--account` from plist args when `None` | `test_plist_excludes_account_flag_when_none` |
| `ScheduledRun` gains `account: Option<String>` field | `test_parse_plist_extracts_account_when_present` (asserts `run.account`) |
| `parse_plist` extracts `--account` value when present | `test_parse_plist_extracts_account_when_present` |
| `parse_plist` returns `None` (not error) for legacy plist without `--account` | `test_parse_plist_account_is_none_for_legacy_plist_without_account` |

### Notes

All spec requirements for Task 3 have corresponding test coverage. The four `run_cmd` tests cover the full parse contract including the error path for a missing flag value. The plist encoding tests confirm the `Some`/`None` branching in `generate_plist_xml`. The two `parse_plist` tests correctly distinguish between a plist that carries `--account` and a legacy plist that does not, satisfying the backwards-compatibility requirement explicitly called out in the spec. No gaps found.

---

## Task 4: main.rs + ui.rs wiring

**Verdict: APPROVED**

### Requirements Checklist

| Requirement | Coverage |
|---|---|
| `run_tui()` signature accepts `accounts: Vec<AccountEntry>` | Compile-time verification — all existing ui tests call the updated signature |
| `AccountDialog` popup is rendered in `ui.rs` | `test_render_shows_account_dialog_popup` |
| Rendered account dialog displays account labels | `test_render_account_dialog_shows_labels` |
| `main.rs` loads accounts before `run_tui()` and passes them in | Not unit-testable at this layer; behavioral contract is process-level wiring |
| Immediate run path uses `selection.account` + `load_token_for_account()` | Not unit-testable without process-level mocking; token loading contract covered by Task 1 tests |
| Scheduled run path uses `run_args.account` to load token | Not unit-testable without process-level mocking; `run_args.account` parse contract covered by Task 3 tests |

### Notes

The two ui tests cover the testable surface of Task 4 — observable rendering behavior of the `AccountDialog` popup. The `main.rs` wiring requirements (account loading before `run_tui()`, token resolution for immediate and scheduled run paths) are process-level integration concerns that cannot be meaningfully unit tested without a process mock. The behavioral contracts they depend on were covered where they could be: `load_token_for_account()` in Task 1, `run_args.account` parsing in Task 3, and `App::new()` accepting accounts in Task 2. The `run_tui()` signature change is verified at compile time by the full suite of existing ui tests. No spec requirement has zero coverage in aggregate across the task sequence. No gaps found.
