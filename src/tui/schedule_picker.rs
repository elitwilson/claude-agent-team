use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use super::app::{AmPm, PickerField, SchedulePickerState};

const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

fn month_abbrev(month: u32) -> &'static str {
    MONTH_NAMES[(month - 1) as usize]
}

fn field_span(label: &str, is_focused: bool) -> Span<'_> {
    let style = if is_focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    Span::styled(format!("[ {label} ]"), style)
}

pub fn render_schedule_picker(
    f: &mut ratatui::Frame,
    state: &SchedulePickerState,
    spec_name: &str,
) {
    let area = f.area();
    f.render_widget(Clear, area);

    // Center a block
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Schedule Run ");
    let inner = centered_rect(70, 12, area);
    f.render_widget(block, inner);

    let content_area = Rect {
        x: inner.x + 2,
        y: inner.y + 1,
        width: inner.width.saturating_sub(4),
        height: inner.height.saturating_sub(2),
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Length(1), // blank
            Constraint::Length(1), // date row
            Constraint::Length(1), // time row
            Constraint::Length(1), // blank
            Constraint::Length(1), // error or blank
            Constraint::Length(1), // footer
        ])
        .split(content_area);

    // Header
    let header = Line::from(format!("Schedule Run: {spec_name}"));
    f.render_widget(Paragraph::new(header), chunks[0]);

    // Date row
    let month_str = format!("{:>3}", month_abbrev(state.month));
    let day_str = format!("{:02}", state.day);
    let year_str = format!("{}", state.year);

    let date_line = Line::from(vec![
        field_span(&month_str, state.focused == PickerField::Month),
        Span::raw("  "),
        field_span(&day_str, state.focused == PickerField::Day),
        Span::raw("  "),
        field_span(&year_str, state.focused == PickerField::Year),
    ]);
    f.render_widget(Paragraph::new(date_line), chunks[2]);

    // Time row
    let hour_str = format!("{:02}", state.hour);
    let minute_str = format!("{:02}", state.minute);
    let ampm_str = match state.am_pm {
        AmPm::Am => "AM",
        AmPm::Pm => "PM",
    };

    let time_line = Line::from(vec![
        field_span(&hour_str, state.focused == PickerField::Hour),
        Span::raw("  "),
        field_span(&minute_str, state.focused == PickerField::Minute),
        Span::raw("  "),
        field_span(ampm_str, state.focused == PickerField::AmPm),
    ]);
    f.render_widget(Paragraph::new(time_line), chunks[3]);

    // Error line
    if let Some(ref err) = state.error {
        let err_line = Line::from(Span::styled(err.as_str(), Style::default().fg(Color::Red)));
        f.render_widget(Paragraph::new(err_line), chunks[5]);
    }

    // Footer
    let footer =
        Line::from("Tab/Shift-Tab: move  \u{2191}\u{2193}: change  Enter: confirm  Esc: cancel");
    f.render_widget(Paragraph::new(footer), chunks[6]);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}

#[cfg(test)]
mod tests;
