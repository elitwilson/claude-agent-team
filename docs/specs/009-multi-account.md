---
number: 009
status: complete
base_branch: main
---

# Feature: Multi-Account OAuth Token Selection

## Summary

Allows the user to store multiple Claude OAuth tokens — one per account — and select which account to use for each run directly in the TUI. Accounts are defined by a human-readable label in a global config file (`~/.claude/claude-agent-team-accounts.toml`) and their tokens are stored in the macOS Keychain. When multiple accounts are configured the TUI inserts an account picker step between team selection and the action popup (Execute Now / Schedule Later). If only one account is configured it is used automatically with no picker shown. The last-used account is remembered as the default selection. Scheduled runs encode the chosen account label in the plist so the correct token is loaded at execution time.

---

## Requirements

- Account labels and Keychain lookup keys are defined in `~/.claude/claude-agent-team-accounts.toml`
- If no accounts file exists or it is empty, the app behaves exactly as it does today (no token, no picker)
- If exactly one account is configured, it is used automatically for every run with no picker shown
- If multiple accounts are configured, the TUI shows an account picker popup after the team popup and before the action popup
- The account picker lists all configured account labels, supports up/down navigation, confirms with Enter, cancels with Esc
- Esc on the account picker returns to the team popup
- The last-used account label is saved and pre-selected on subsequent launches
- The selected account applies to both Execute Now and Schedule Later runs
- Scheduled runs store the account label in the plist; the correct token is loaded from Keychain at execution time
- If a token cannot be loaded for the selected account at run time, the app warns and proceeds without a token (same behavior as today when no token is found)

---

## Scope

### In Scope

- New `src/accounts.rs` module — account config struct, loading, and Keychain token retrieval
- `src/prefs.rs` — add `default_account: Option<String>` field
- `src/tui/app.rs` — new `PopupAction::AccountDialog` variant, updated popup chain, account state on `App`, updated `TuiResult`
- `src/tui/ui.rs` — account picker popup rendering and event handling
- `src/scheduler.rs` — encode `--account` label in plist args
- `src/run_cmd.rs` — parse `--account` flag
- `src/main.rs` — load account list and pass to `run_tui()`; use selected account label to load token for both immediate and scheduled runs
- Global accounts config file format and manual setup instructions (in Considerations)

### Out of Scope

- A CLI command for adding or managing accounts (manual Keychain entry + config edit is the setup workflow for now)
- Token validation or expiry checking
- Automatic rotation across accounts

---

## Technical Approach

- **`~/.claude/claude-agent-team-accounts.toml` format:**
  ```toml
  [[accounts]]
  label = "personal"

  [[accounts]]
  label = "work"
  ```
  Labels are arbitrary strings. The label is used as the Keychain account name.

- **Keychain convention:** Service name `com.claude-agent-team`, account name = label. One entry per account.

- **New `src/accounts.rs`:**
  ```rust
  pub struct AccountEntry {
      pub label: String,
  }
  ```
  - `load_accounts() -> Vec<AccountEntry>` — reads from `~/.claude/claude-agent-team-accounts.toml`; returns empty vec if the file does not exist.
  - `load_token_for_account(label: &str) -> Option<String>` — calls `security find-generic-password -w -s com.claude-agent-team -a <label>`; returns `None` on any error.

- **`Prefs`:** Add `default_account: Option<String>`. Serialized to the existing prefs file. Updated when the user confirms an account selection in the TUI.

- **`PopupAction::AccountDialog` (new variant in `tui/app.rs`):**
  ```rust
  AccountDialog { selected_index: usize }
  ```
  Popup chain becomes: `TeamDialog` → `AccountDialog` (if accounts.len() > 1) → `ActionDialog`.
  - `confirm_popup()` on `TeamDialog`: if accounts.len() > 1, open `AccountDialog` pre-selected at `prefs.default_account` index (fallback to 0). If accounts.len() <= 1, skip to `ActionDialog` as today.
  - `confirm_popup()` on `AccountDialog`: store `account_index`, save `prefs.default_account`, open `ActionDialog`.
  - `dismiss_popup()` on `AccountDialog`: restore `TeamDialog`.

- **`App` state changes:** Add `accounts: Vec<AccountEntry>` and `account_index: usize`. Loaded at startup and passed into `App::new()`.

- **`TuiResult`:** Add `account: Option<String>` — the selected label, or `None` if no accounts configured.

- **`scheduler.rs`:** `schedule_run` and `generate_plist_xml` accept an `account: Option<&str>`. If `Some`, append `--account <label>` to the plist `ProgramArguments`. `parse_plist` extracts it alongside `--spec` and `--team`. `ScheduledRun` gains `account: Option<String>`.

- **`run_cmd.rs`:** `RunArgs` gains `account: Option<String>`. `parse_run_args` handles `--account <label>`.

- **`main.rs`:**
  - Load accounts via `accounts::load_accounts()` before calling `run_tui()`; pass them in.
  - For immediate runs: `accounts::load_token_for_account(&label)` using `selection.account`.
  - Remove the now-unused `runner::load_oauth_token()` call from the immediate-run path (or leave it as a fallback if `selection.account.is_none()`).
  - For scheduled runs (`run_scheduled`): use `run_args.account` to load the token.

---

## Success Criteria

- [ ] `~/.claude/claude-agent-team-accounts.toml` with zero accounts: app behaves identically to today
- [ ] With one account: no picker shown; that account's token is loaded automatically for every run
- [ ] With multiple accounts: account picker appears after team popup, before action popup; up/down navigates; Enter confirms; Esc returns to team popup
- [ ] The last confirmed account is pre-selected on the next launch
- [ ] A scheduled run plist contains `--account <label>`; at execution time the token is loaded from Keychain using that label
- [ ] If `security find-generic-password` fails for the selected label, a warning is printed and the run proceeds without a token
- [ ] `TuiResult` carries the selected account label
- [ ] All existing tests pass or are updated to reflect new `TuiResult` and `RunArgs` fields

---

## Tasks

- [ ] **Add `accounts.rs` module:** `AccountEntry` struct, `load_accounts()`, `load_token_for_account()`. Add `default_account: Option<String>` to `Prefs`. Unit test `load_accounts()` against a temp file; unit test `load_token_for_account()` by mocking the `security` command or testing the parse path in isolation.

- [ ] **Update `app.rs` popup chain:** Add `accounts: Vec<AccountEntry>` and `account_index: usize` to `App`. Add `PopupAction::AccountDialog`. Update `confirm_popup()`, `dismiss_popup()`, `popup_move_up/down()` for the new variant. Update `App::new()` signature to accept accounts. Add `account: Option<String>` to `TuiResult`. Update all affected tests. Depends on Task 1.

- [ ] **Update `run_cmd.rs` and `scheduler.rs`:** Add `account: Option<String>` to `RunArgs` and parse `--account` flag. Add `account` param to `schedule_run()` and `generate_plist_xml()`; encode in plist args. Add `account` field to `ScheduledRun`; update `parse_plist()` to extract it. Update all affected tests.

- [ ] **Wire everything in `main.rs` and `ui.rs`:** Load accounts before `run_tui()`; pass into `App::new()`. Render `AccountDialog` popup in `ui.rs`. Use `selection.account` to load the OAuth token for immediate runs. Use `run_args.account` in `run_scheduled`. Update `run_tui()` signature to accept accounts. Depends on Tasks 1–3.

---

## Considerations

- **Manual setup instructions** — to add an account:
  1. Add a `[[accounts]]` entry with a `label` to `~/.claude/claude-agent-team-accounts.toml` (create the file if it doesn't exist).
  2. Store the token in Keychain: `security add-generic-password -s com.claude-agent-team -a <label> -w <token>`
  To update a token: `security add-generic-password -U -s com.claude-agent-team -a <label> -w <new-token>`

- **Interaction with spec 008** — spec 008 removes `TuiResult::scheduled_at` and modifies `App` state and the popup chain. This spec adds `TuiResult::account` and inserts `AccountDialog` into the same popup chain. If implemented after 008, the popup chain is `TeamDialog` → `AccountDialog` → `ActionDialog`. If implemented before 008, the same insertion applies to the current chain. The two specs are independent but touch overlapping files; implement 008 first to avoid merge conflicts.

- **`runner::load_oauth_token()`** — the existing function is hardcoded to `claude-token-1`. Once this feature is in place it becomes dead code. It can be deleted or left as a legacy fallback; the spec does not require deletion but the agent should not call it from the main run paths.

- **Single-account auto-select** — when `accounts.len() == 1`, `account_index` is set to 0 and `prefs.default_account` is updated silently (no picker). This keeps the single-account case zero-friction.

- **Plist encoding note** — `ScheduledRun::account` and `--account` in the plist args follow the same pattern as `--spec` and `--team`. `parse_plist()` should treat a missing `--account` as `None` (not an error) for backwards compatibility with plists created before this feature.
