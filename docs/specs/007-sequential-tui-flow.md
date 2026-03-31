---
number: 007
status: ready
---

# Feature: Sequential TUI Selection Flow

## Summary

Replaces the current three-panel side-by-side TUI layout (Spec | Team | Options) with a sequential popup-driven flow. The user selects a spec from a full-width list, then a team popup appears, then the existing action popup (Execute Now / Schedule Later). Options are removed as a navigable panel and replaced with direct keybind toggles shown in the footer. The result is a linear funnel that makes the dependency between choices explicit and removes panel-switching overhead.

---

## Requirements

- The main launcher screen shows only the spec list, full width, with no Team or Options panels alongside it
- Pressing Enter on a ready spec opens a Team selection popup (not the action popup directly)
- The team popup lists all available teams, supports up/down navigation, confirms with Enter, cancels with Esc
- Confirming a team opens the existing action popup (Execute Now / Schedule Later)
- Cancelling the team popup returns to the spec list with no changes
- The Options panel is removed; headless, show_complete, and show_blocked are toggled via direct keybinds from the spec list
- Current state of each pref is shown in the footer
- All existing flows that follow the action popup (schedule picker, confirm) are unchanged
- The Requirements tab and its draft flow are unchanged
- Esc dismisses whichever popup is active and returns to the previous step (team popup → spec list, action popup → team popup)

---

## Scope

### In Scope

- `src/tui/app.rs` — state machine, Panel enum, PopupAction enum, event handlers
- `src/tui/ui.rs` — layout and rendering

### Out of Scope

- Schedule picker behavior (unchanged)
- Metrics screen (unchanged)
- `TuiResult` shape (unchanged — team is still resolved before exit)
- Any changes to how teams are discovered or loaded

---

## Technical Approach

- **`Panel` enum:** Remove `Team` and `Options` variants. The only panel state remaining is the spec list; `focused_panel` field can be removed entirely or replaced with a simpler `active_tab` check. The `Panel` enum itself may be deleted.

- **`PopupAction` enum:** Add a new variant `TeamDialog { selected_index: usize }`. The flow through popup variants is now: `TeamDialog` → (on confirm) → `ActionDialog`. Esc on `ActionDialog` should restore `TeamDialog` (re-open team popup) rather than dismissing entirely.

- **`App::confirm()`:** Instead of calling `open_action_popup()`, call a new `open_team_popup()` method that sets `popup = Some(PopupAction::TeamDialog { selected_index: self.team_index })`.

- **`App::confirm_popup()`:** When confirming `TeamDialog`, store the selected index into `self.team_index` and open the action dialog. When confirming `ActionDialog`, behavior is unchanged.

- **`App::dismiss_popup()`:** Esc on `TeamDialog` → `popup = None`. Esc on `ActionDialog` → restore `TeamDialog` (re-open team popup at current `team_index`).

- **Options keybinds:** Replace the Options panel with direct key handlers in the event loop: `h` toggles headless, `c` toggles show_complete, `b` toggles show_blocked. All call `self.prefs.save()` after toggling, same as `toggle_option()` does today.

- **Layout (`ui.rs`):** Replace the three-chunk vertical layout with a single full-width spec list. Remove Team panel and Options panel rendering blocks. Add `TeamDialog` popup rendering (same overlay style as current action popup — centered, narrow, list of team names with highlight). Footer updated to show pref state: e.g. `h:headless[on]  c:complete[off]  b:blocked[off]  Tab switch tab  Enter select  q quit`.

- **`next_panel()` / tab navigation:** Tab key currently cycles panels. With only one panel, Tab switches between Specs and Requirements tabs (same as Left/Right does today). Remove the panel-cycling behavior.

---

## Success Criteria

- [ ] Main launcher screen is a single full-width spec list with no Team or Options panels
- [ ] Pressing Enter on a ready spec opens a Team popup listing all available teams
- [ ] Up/down navigates the team popup; Enter confirms and opens the action popup; Esc returns to spec list
- [ ] Esc on the action popup restores the team popup (does not collapse all the way to spec list)
- [ ] `h`, `c`, `b` keybinds toggle the respective prefs from the spec list; footer reflects current state
- [ ] Full flow (spec → team → execute now) produces the same `TuiResult` as before
- [ ] Full flow (spec → team → schedule later → schedule picker) works end-to-end
- [ ] Requirements tab draft flow is unaffected
- [ ] All existing `app.rs` unit tests pass or are updated to reflect removed Panel variants

---

## Tasks

- [ ] **Update `app.rs` state machine:** Remove `Panel` enum and `focused_panel` field. Add `PopupAction::TeamDialog { selected_index: usize }`. Implement `open_team_popup()`. Update `confirm()` to call it. Update `confirm_popup()` to chain `TeamDialog` → `ActionDialog`. Update `dismiss_popup()` so Esc on `ActionDialog` restores `TeamDialog`. Add `h`/`c`/`b` toggle methods for prefs. Remove `next_panel()`, `move_up/down` panel dispatch for Team/Options, and `toggle_option()`/`toggle_headless()`. Update or remove tests in `app/tests.rs` that reference removed variants.

- [ ] **Update `ui.rs` layout and rendering:** Replace three-panel layout with single full-width spec list. Remove Team and Options panel rendering. Add `TeamDialog` popup rendering block (same overlay approach as `ActionDialog`). Update footer text to show pref state and new keybinds. Update Tab key handler to switch tabs instead of cycling panels.

- [ ] **Integration smoke test:** Add or update a test that simulates the full event sequence — select spec, confirm team popup, confirm action popup — and verifies the resulting `TuiResult` has the correct spec and team set.

---

## Considerations

- `toggle_headless()` is currently called directly by the `h` key in the event loop as well as via `toggle_option()`. After this change, `h` should call a direct headless toggle on `prefs` — the `toggle_option()` indirection can be removed.
- The `OPTIONS_ITEMS` constant and `options_index` field can be deleted.
- `next_panel()` currently cycles Spec → Team → Options → Spec. After removal of Team and Options, Tab's only remaining job is switching spec tabs — consolidate with the existing Left/Right tab-switch handler.
- The `PopupAction::TeamDialog` selected index should be initialized to `self.team_index` so the popup opens with the last-used team pre-selected.
