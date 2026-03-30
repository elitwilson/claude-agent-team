use crate::metrics::query::RunSummary;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

/// State for the metrics screen.
#[derive(Debug)]
pub struct MetricsState {
    pub runs: Vec<RunSummary>,
    pub scroll_offset: usize,
    pub visible_rows: usize,
    pub error: Option<String>,
}

impl MetricsState {
    pub fn new(runs: Vec<RunSummary>) -> Self {
        Self {
            runs,
            scroll_offset: 0,
            visible_rows: 0,
            error: None,
        }
    }

    pub fn with_error(error: String) -> Self {
        Self {
            runs: vec![],
            scroll_offset: 0,
            visible_rows: 0,
            error: Some(error),
        }
    }

    /// Scroll up by one row.
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scroll down by one row, clamped so the last row stays visible.
    pub fn scroll_down(&mut self) {
        let max = self.runs.len().saturating_sub(self.visible_rows);
        if self.scroll_offset < max {
            self.scroll_offset += 1;
        }
    }
}

/// Render the metrics screen into the given frame.
pub fn render_metrics(f: &mut ratatui::Frame, state: &mut MetricsState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(f.area());

    let content_area = chunks[0];
    let footer_area = chunks[1];

    // borders (2) + header row (1)
    state.visible_rows = (content_area.height as usize).saturating_sub(3);

    // Error state
    if let Some(ref err) = state.error {
        let msg = Paragraph::new(err.as_str())
            .block(Block::default().borders(Borders::ALL).title("Metrics"));
        f.render_widget(msg, content_area);
    } else if state.runs.is_empty() {
        // Empty state
        let msg = Paragraph::new("No runs found.")
            .block(Block::default().borders(Borders::ALL).title("Metrics"));
        f.render_widget(msg, content_area);
    } else {
        // Table header
        let header = Row::new(vec![
            Cell::from("Date"),
            Cell::from("Spec"),
            Cell::from("Team"),
            Cell::from("Input"),
            Cell::from("Output"),
            Cell::from("Cache"),
            Cell::from("Status"),
        ])
        .style(Style::default().fg(Color::Yellow));

        // Data rows (sliced by scroll_offset)
        let rows: Vec<Row> = state
            .runs
            .iter()
            .skip(state.scroll_offset)
            .map(|run| {
                let status = if run.exit_code == 0 {
                    "\u{2713}" // ✓
                } else {
                    "\u{2717}" // ✗
                };
                Row::new(vec![
                    Cell::from(run.run_date.as_str()),
                    Cell::from(run.feature_slug.as_str()),
                    Cell::from(run.team.as_str()),
                    Cell::from(run.total_input.to_string()),
                    Cell::from(run.total_output.to_string()),
                    Cell::from(run.total_cache.to_string()),
                    Cell::from(status),
                ])
            })
            .collect();

        let widths = [
            Constraint::Length(12),
            Constraint::Min(15),
            Constraint::Length(15),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(8),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Metrics"));

        f.render_widget(table, content_area);
    }

    let footer = Paragraph::new(Line::from(
        "  \u{2191}\u{2193} scroll  q / Esc back",
    ));
    f.render_widget(footer, footer_area);
}

#[cfg(test)]
mod tests;
