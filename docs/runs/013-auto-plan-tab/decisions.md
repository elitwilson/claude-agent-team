# Decisions Log: 013-auto-plan-tab

## Task 1: Add skills/ and agents/ directories

- Copying content verbatim from dotfiles sources as spec directs.
- Checking if bug-diagnostic-agent.md is needed — spec only mentions architect.md and project-scribe.md; omitting bug-diagnostic-agent.md as it is not listed in the spec scope.

## Task 3: app.rs — confirm() on Plan tab with multiple accounts

Spec says: "If multi-account is configured, the existing AccountDialog appears after ActionDialog confirmation, same as the Specs tab flow."
This means the Plan tab's ActionDialog → ScheduleLater path should also eventually allow account selection. However, the spec also says confirm() on Plan tab opens ActionDialog *directly* (bypasses open_team_popup). Looking at the existing flow:
- TeamDialog → AccountDialog (if multi-account) → ActionDialog
- Plan tab: no TeamDialog, goes directly to ActionDialog
- When ActionDialog → ExecuteNow is confirmed, the result() needs to include the account.
- The existing confirmed_popup() flow for ActionDialog just sets confirmed=true, account_index already tracks current selection.

Decision: Plan tab confirm() opens ActionDialog directly (as spec says). The account_index is already set from app initialization. result() on Plan tab uses self.account_index like other tabs. This matches the spec's intent without additional dialogs.

## Task 3: dismiss_popup on Plan tab ActionDialog

Since Plan tab goes directly to ActionDialog (no TeamDialog in front), Esc on ActionDialog while on Plan tab should return to spec list (popup = None), not TeamDialog. The existing dismiss_popup logic restores TeamDialog from ActionDialog — this needs a guard on active_tab.
Decision: When active_tab == Plan, Esc on ActionDialog dismisses to None (no team to restore).

## Task 4: --spec and --team becoming optional

The spec says: "--spec and --team become optional (return None instead of erroring when absent) unless mode is not auto-plan."
Current tests assert that missing --spec or --team returns an error. These tests must remain valid.
Decision: Keep error for missing --spec/--team when --mode is not "auto-plan". When --mode auto-plan is provided, allow both to be None. RunArgs.spec and RunArgs.team change to Option<String>.

This is a breaking change to RunArgs — all callers in main.rs that use run_args.spec and run_args.team directly need updating. Log this as a known cascade.

## Task 5: confirm_picker on Plan tab uses "auto-plan" slug

The existing confirm_picker() uses visible_specs()[self.spec_index] to get the spec name. On Plan tab this would panic (no list). Need to guard on active_tab == Plan and use "auto-plan" as slug directly, with team = "" (empty string) since auto-plan doesn't need a team.

## Task 5: dismiss_popup on ActionDialog on Plan tab

confirm_picker currently reads spec from visible_specs(). Need to add Plan tab guard in confirm_picker too.
