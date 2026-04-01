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
