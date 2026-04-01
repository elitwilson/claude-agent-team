# Decisions — 009 Multi-Account

## Task Breakdown

The spec was broken into 4 tasks matching the spec's own task list:

1. **accounts.rs + Prefs** — `AccountEntry`, `load_accounts()`, `load_token_for_account()`, `default_account` on `Prefs`
2. **app.rs popup chain** — `PopupAction::AccountDialog`, `App` state additions, popup chain logic, `TuiResult::account`
3. **run_cmd.rs + scheduler.rs** — `--account` flag parsing, plist encoding, `ScheduledRun::account`, `parse_plist` extraction
4. **main.rs + ui.rs wiring** — Load accounts, pass to `run_tui()`, render `AccountDialog`, use `selection.account` for token loading

## Decisions

### `load_token_for_account` isolation
The spec requires testing `load_token_for_account()` "by mocking the `security` command or testing the parse path in isolation." Since the `security` binary is system-only and cannot be mocked without process injection, the testable path is the output-parsing logic extracted from `load_token_for_account`. The main function will be tested indirectly via integration (or simply excluded from unit tests since it's a thin wrapper around a system call — same pattern as `runner::load_oauth_token()`).

### `load_token_for_account` uses same pattern as `runner::load_oauth_token()`
The spec says the service name is `com.claude-agent-team` with account = label, vs the existing hardcoded `claude-token-1` service. Both are just thin wrappers; the test surface is the same.

### `App::new()` accounts parameter placement
The spec says `accounts: Vec<AccountEntry>` is added to `App` state and passed into `App::new()`. Appending to the end of the parameter list to minimize test churn on existing call sites.

### `TuiResult::account` as `Option<String>`
Spec says `None` when no accounts configured; the field name is `account` matching the spec verbatim.

### ui.rs scope in Task 4
`ui.rs` does not have its own test for popup rendering at the widget level (tests are event-handling tests). The `AccountDialog` rendering will follow the same pattern as `TeamDialog` — no new test file structure needed beyond Task 2's app state tests.
