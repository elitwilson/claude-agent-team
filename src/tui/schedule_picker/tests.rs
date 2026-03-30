use super::*;
use crate::tui::app::{AmPm, PickerField, SchedulePickerState};
use ratatui::{Terminal, backend::TestBackend};

fn render_to_string(
    state: &SchedulePickerState,
    spec_name: &str,
    width: u16,
    height: u16,
) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| render_schedule_picker(f, state, spec_name))
        .unwrap();
    let buffer = terminal.backend().buffer().clone();
    let mut output = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            output.push_str(buffer.cell((x, y)).unwrap().symbol());
        }
        output.push('\n');
    }
    output
}

fn sample_state() -> SchedulePickerState {
    SchedulePickerState {
        month: 4, // April
        day: 2,
        year: 2026,
        hour: 8,
        minute: 0,
        am_pm: AmPm::Pm,
        focused: PickerField::Month,
        error: None,
    }
}

// --- Spec name header ---

#[test]
fn test_render_contains_spec_name() {
    let state = sample_state();
    let output = render_to_string(&state, "005-my-feature", 80, 14);
    assert!(
        output.contains("005-my-feature"),
        "should display the spec name"
    );
}

// --- Date row: month as abbreviated name ---

#[test]
fn test_render_month_as_abbreviated_name() {
    let state = sample_state(); // month = 4 = April
    let output = render_to_string(&state, "spec", 80, 14);
    assert!(output.contains("Apr"), "month 4 should render as 'Apr'");
}

#[test]
fn test_render_january() {
    let mut state = sample_state();
    state.month = 1;
    let output = render_to_string(&state, "spec", 80, 14);
    assert!(output.contains("Jan"), "month 1 should render as 'Jan'");
}

#[test]
fn test_render_december() {
    let mut state = sample_state();
    state.month = 12;
    let output = render_to_string(&state, "spec", 80, 14);
    assert!(output.contains("Dec"), "month 12 should render as 'Dec'");
}

// --- Date row: day and year ---

#[test]
fn test_render_day_zero_padded() {
    let state = sample_state(); // day = 2
    let output = render_to_string(&state, "spec", 80, 14);
    assert!(output.contains("02"), "day 2 should render as '02'");
}

#[test]
fn test_render_year() {
    let state = sample_state(); // year = 2026
    let output = render_to_string(&state, "spec", 80, 14);
    assert!(output.contains("2026"), "should display the year");
}

// --- Time row ---

#[test]
fn test_render_hour_zero_padded() {
    let state = sample_state(); // hour = 8
    let output = render_to_string(&state, "spec", 80, 14);
    assert!(output.contains("08"), "hour 8 should render as '08'");
}

#[test]
fn test_render_minute_zero_padded() {
    let state = sample_state(); // minute = 0
    let output = render_to_string(&state, "spec", 80, 14);
    assert!(output.contains("00"), "minute 0 should render as '00'");
}

#[test]
fn test_render_am_pm() {
    let state = sample_state(); // PM
    let output = render_to_string(&state, "spec", 80, 14);
    assert!(output.contains("PM"), "should display 'PM'");
}

#[test]
fn test_render_am() {
    let mut state = sample_state();
    state.am_pm = AmPm::Am;
    let output = render_to_string(&state, "spec", 80, 14);
    assert!(output.contains("AM"), "should display 'AM'");
}

// --- Error display ---

#[test]
fn test_render_error_message_when_present() {
    let mut state = sample_state();
    state.error = Some("Scheduled time must be in the future".to_string());
    let output = render_to_string(&state, "spec", 80, 14);
    assert!(
        output.contains("Scheduled time must be in the future"),
        "should display error message"
    );
}

#[test]
fn test_render_no_error_line_when_none() {
    let state = sample_state();
    let output = render_to_string(&state, "spec", 80, 14);
    assert!(
        !output.contains("Scheduled time must be in the future"),
        "should not display error when none"
    );
}

// --- Key hints footer ---

#[test]
fn test_render_key_hints() {
    let state = sample_state();
    let output = render_to_string(&state, "spec", 80, 14);
    assert!(output.contains("Enter"), "should show Enter hint");
    assert!(output.contains("Esc"), "should show Esc hint");
}
