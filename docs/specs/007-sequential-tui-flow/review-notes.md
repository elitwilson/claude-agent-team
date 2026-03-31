## Task 1: app.rs state machine

**Verdict: APPROVED**

### Requirements checklist vs. test coverage

| Requirement | Test(s) | Status |
|---|---|---|
| `PopupAction::TeamDialog { selected_index }` exists | `test_confirm_on_ready_spec_opens_team_dialog`, all `TeamDialog` match tests | Covered |
| `open_team_popup()` sets popup to TeamDialog at current team_index | `test_open_team_popup_uses_current_team_index` | Covered |
| `confirm()` on ready spec calls `open_team_popup()` | `test_confirm_on_ready_spec_opens_team_dialog`, `test_confirm_opens_team_dialog_with_current_team_index` | Covered |
| `confirm()` on blocked/complete spec does nothing | `test_blocked_spec_is_not_confirmable`, `test_complete_spec_is_not_confirmable` | Covered |
| `confirm_popup()` on TeamDialog stores selected_index → team_index, opens ActionDialog | `test_confirm_popup_on_team_dialog_stores_team_and_opens_action_dialog` | Covered |
| `confirm_popup()` on ActionDialog (execute now) → confirmed | `test_full_flow_confirm_team_then_execute_now_sets_confirmed` | Covered |
| `confirm_popup()` on ActionDialog (schedule later) → SchedulePicker screen | `test_full_flow_confirm_team_then_schedule_later_sets_screen` | Covered |
| `dismiss_popup()` on TeamDialog → popup = None | `test_dismiss_popup_on_team_dialog_returns_to_spec_list` | Covered |
| `dismiss_popup()` on ActionDialog → restores TeamDialog | `test_dismiss_popup_on_action_dialog_restores_team_dialog` | Covered |
| `toggle_headless()` toggles prefs.headless | `test_toggle_headless_toggles_pref` | Covered |
| `toggle_show_complete()` toggles prefs.show_complete | `test_toggle_show_complete_toggles_pref`, `test_toggle_show_complete_clamps_spec_index` | Covered |
| `toggle_show_blocked()` toggles prefs.show_blocked | `test_toggle_show_blocked_toggles_pref`, `test_toggle_show_blocked_clamps_spec_index` | Covered |
| TeamDialog up/down navigation with clamping | `test_popup_move_down/up_on_team_dialog_*` (4 tests) | Covered |
| `result()` returns correct TuiResult after full flow | `test_result_returns_correct_selection_after_full_flow`, `test_result_returns_none_when_not_confirmed` | Covered |
| Panel enum / focused_panel removed | Structural (compile-time enforcement) | Acceptable |
| `next_panel()` removed | Structural (compile-time enforcement) | Acceptable |

### Notes

- Tests verify observable behavior throughout; no implementation-detail testing observed.
- `prefs.save()` side effect is not tested, but this is a file I/O concern outside the scope of unit tests and not called out in the spec's success criteria.
- Bonus coverage: `visible_specs()` filter behavior and spec-index clamping on pref toggle are tested, which directly supports the toggle requirements.

## Task 3: Integration smoke test

**Verdict: APPROVED**

### Requirements checklist vs. test coverage

| Requirement | Test(s) | Status |
|---|---|---|
| Full flow (spec → team → execute now) produces correct `TuiResult` (spec and team set) | `test_integration_full_flow_spec_to_team_to_execute_produces_correct_result` | Covered |
| Full flow (spec → team → schedule later → schedule picker) works end-to-end | `test_full_flow_confirm_team_then_schedule_later_sets_screen` (Task 1) | Covered |

### Notes

- The execute-now integration test exercises the complete sequential event chain at the observable level: navigate spec list, open team popup, navigate team popup, confirm team, confirm Execute Now, verify `TuiResult` fields. No implementation internals are tested.
- The schedule-later path is satisfied by cross-reference to the Task 1 test. This is valid: the spec's out-of-scope section explicitly excludes schedule picker behavior, so the end-to-end requirement for that path is met by verifying the screen transition to SchedulePicker.
- The integration test also verifies `result.headless`, `result.mode`, and `result.scheduled_at` beyond what the spec strictly requires — this is additive coverage, not a flag.
