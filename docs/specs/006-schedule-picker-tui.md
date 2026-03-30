---
number: 006
status: ready
---

# Feature: Schedule Picker TUI

## Summary

Extends the TUI launcher so the user can choose, after selecting a Ready spec, whether to run the agent team immediately or schedule it for a future date and time. Pressing Enter on a Ready spec shows a popup asking "Execute now" or "Schedule for later." Choosing Schedule opens a full-screen date/time picker with six incremental fields — Month, Day, Year, Hour, Minute, AM/PM — defaulting to 8:00 PM today. On confirmation, the TUI hands the scheduled datetime to `scheduler::schedule_run` (spec 005) and exits. This spec covers only the TUI layer; the scheduling backend is spec 005.

---

## Requirements

- Pressing Enter on a Ready spec shows a two-option action popup: **Execute now** and **Schedule for later**
- The popup is an overlay rendered on top of the existing launcher layout, not a new screen
- Selecting **Execute now** from the popup behaves identically to the current Enter behavior (immediate run)
- Selecting **Schedule for later** opens a full-screen date/time picker (`Screen::SchedulePicker`)
- The picker has six incremental fields: Month, Day, Year, Hour, Minute, AM/PM
- Fields default to today's date at 8:00 PM
- Tab/Shift-Tab moves focus between fields; Up/Down increments/decrements the focused field
- Year is clamped to the range [current year, current year + 5]; Up/Down stops at the bounds rather than wrapping
- Day is clamped to valid values for the selected Month and Year (e.g. Feb 28/29, months with 30 days)
- Hour wraps 12→1 and 1→12; Minute wraps 59→0 and 0→59; AM/PM toggles
- Pressing Enter confirms the selection. If the resulting datetime is less than 1 minute in the future, an inline error is shown and the picker does not exit
- On valid confirmation, the TUI calls `scheduler::schedule_run` and exits, printing `Scheduled: <spec> for <datetime>` to stdout
- Pressing Escape anywhere in the popup or picker returns to the launcher with no state change
- The popup does not appear for the Requirements tab (draft runs are not schedulable)

---

## Scope

### In Scope

- Action popup overlay (two-option dialog)
- Full-screen date/time picker screen with six incremental fields
- Validation (future-time check) with inline error display
- Wiring in `main.rs`: when `TuiResult` carries a `scheduled_at`, call `scheduler::schedule_run` instead of running immediately

### Out of Scope

- Any calendar grid rendering — date entry is field-based only
- Displaying or cancelling pending scheduled runs in the TUI (future spec)
- Scheduling draft runs (Requirements tab)
- Any scheduling backend logic — that is entirely spec 005

---

## Technical Approach

**No new dependencies.** `chrono` is already present and sufficient for all date/time handling in this spec.

**`TuiResult` extension:** Add `scheduled_at: Option<chrono::DateTime<chrono::Local>>`. `None` means execute immediately. `main.rs` branches on this field.

**Action popup:** Add `popup: Option<PopupAction>` to `App`:

```rust
pub enum PopupAction {
    ActionDialog { selected: ActionChoice },
}

pub enum ActionChoice {
    ExecuteNow,
    ScheduleLater,
}
```

When `popup` is `Some`, the event loop routes keypresses to the popup handler first. Rendering draws the existing launcher layout, then a centered `Clear` + `Block` overlay on top. Escape dismisses; Enter confirms the highlighted choice.

**Schedule picker screen:** Add `Screen::SchedulePicker` and `SchedulePickerState` to `App`:

```rust
pub struct SchedulePickerState {
    pub month: u32,   // 1–12
    pub day: u32,     // 1–days_in_month
    pub year: i32,    // current year to current year + 5
    pub hour: u32,    // 1–12
    pub minute: u32,  // 0–59
    pub am_pm: AmPm,
    pub focused: PickerField,
    pub error: Option<String>,
}

pub enum PickerField { Month, Day, Year, Hour, Minute, AmPm }
```

Default values at construction: today's date from `chrono::Local::now()`, hour = 8, minute = 0, am_pm = PM.

**Rendering:** The picker screen renders a centered block with two rows of fields. Date fields on the first row, time fields on the second. Each field is a fixed-width cell showing its value (month rendered as abbreviated name: "Apr"). The focused field is highlighted with the existing yellow focus style. An optional error line renders below in red.

```
  Schedule Run: 005-my-feature

  [ Apr ] [ 02 ] [ 2026 ]
  [ 08 ] [ 00 ] [ PM ]

  Tab/Shift-Tab: move   ↑↓: change   Enter: confirm   Esc: cancel
```

**Day clamping:** When month or year changes, clamp day to `days_in_month(month, year)`. Use `chrono::NaiveDate::from_ymd_opt` to compute the last valid day.

**Validation and handoff:** On Enter, convert the six fields to a 24-hour `NaiveTime` using `chrono::NaiveTime::from_hms_opt` — use chrono's own 12→24hr conversion to avoid manual off-by-one errors at midnight (12:00 AM = 00:00) and noon (12:00 PM = 12:00). Construct a `NaiveDateTime`, localize with `chrono::Local`, and check it is at least 1 minute ahead of `Local::now()`. If not, set `state.error`. If valid, populate `TuiResult::scheduled_at` and confirm. In `main.rs`:

```rust
if let Some(scheduled_at) = selection.scheduled_at {
    scheduler::schedule_run(&spec, &team, headless, &cwd, scheduled_at)?;
    println!("Scheduled: {} for {}", selection.spec, scheduled_at.format("%Y-%m-%d %I:%M %p"));
    return Ok(());
}
// else: existing immediate-run path unchanged
```

---

## Success Criteria

- [ ] Pressing Enter on a Ready spec shows the two-option action popup
- [ ] Pressing Escape on the popup or picker returns to the launcher with no state change
- [ ] Selecting **Execute now** behaves identically to the pre-popup Enter behavior
- [ ] Selecting **Schedule for later** opens the picker defaulted to today at 8:00 PM
- [ ] Tab/Shift-Tab cycles through all six fields
- [ ] Up/Down increments/decrements each field with correct wrapping and day clamping
- [ ] Confirming a datetime less than 1 minute in the future shows an inline error and does not exit
- [ ] Confirming a valid future datetime calls `scheduler::schedule_run` and exits with the confirmation message

---

## Tasks

- [ ] **`TuiResult` extension and `main.rs` wiring:** Add `scheduled_at: Option<chrono::DateTime<Local>>` to `TuiResult`. Update `App::result()` to populate it. In `main.rs`, branch on `scheduled_at`: call `scheduler::schedule_run` or proceed with the existing immediate-run path. No visible behavior change until the picker exists.

- [ ] **Action popup:** Add `PopupAction`, `ActionChoice` types and `popup` field to `App`. Implement popup rendering (centered overlay using `Clear` + `Block`). Wire keyboard handling: Up/Down navigate choices, Enter confirms, Escape dismisses. Transition to `Screen::SchedulePicker` when Schedule is chosen. Unit test popup state transitions.

- [ ] **Schedule picker — state and field navigation:** Add `Screen::SchedulePicker`, `SchedulePickerState`, `PickerField`, `AmPm` types. Implement Tab/Shift-Tab focus cycling and Up/Down increment/decrement for all six fields, including hour/minute wrapping and day clamping on month/year change. Unit test all field transitions and day clamping edge cases (Feb 29 on leap/non-leap year, 30-day months).

- [ ] **Schedule picker — rendering:** Implement the picker screen renderer: spec name header, two rows of fixed-width field cells with focus highlight, footer with key hints, optional error line in red.

- [ ] **Validation:** On Enter in the picker, construct `chrono::DateTime<Local>`, validate it is at least 1 minute in the future, set `state.error` or confirm. Unit test the validation boundary.

---

## Considerations

- Day clamping must fire whenever month or year changes, not just when the Day field is edited. If the user is on Feb 29 and increments month to March, day must immediately clamp to 28 or 29 before the next render.
- The default of 8:00 PM today may already be in the past if the user launches at 8:01 PM. The validation handles this — the error message should be clear ("Scheduled time must be in the future") rather than generic.
- Month display as abbreviated name ("Jan"–"Dec") rather than a number makes the layout easier to read, but the internal representation stays as `u32` (1–12) to simplify date construction.
- 12→24hr conversion must go through `chrono` rather than manual arithmetic to correctly handle 12:00 AM (midnight = 00:00) and 12:00 PM (noon = 12:00), which are the two classic off-by-one traps in 12-hour time.
- Escape from the picker goes directly to the launcher, not back to the action popup. The popup is transient and does not need a back-stack.
