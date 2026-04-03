use chrono::{DateTime, Local};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

use super::app::ActionChoice;
use crate::accounts::AccountEntry;

fn popup_frame(area: Rect, width: u16, height: u16, title: &str, buf: &mut Buffer) -> Rect {
    let clamped_w = width.min(area.width);
    let clamped_h = height.min(area.height);
    let x = area.width.saturating_sub(clamped_w) / 2;
    let y = area.height.saturating_sub(clamped_h) / 2;
    let popup_area = Rect::new(x, y, clamped_w, clamped_h);
    Clear.render(popup_area, buf);
    Block::default().borders(Borders::ALL).title(title).render(popup_area, buf);
    Rect::new(
        popup_area.x + 1,
        popup_area.y + 1,
        popup_area.width.saturating_sub(2),
        popup_area.height.saturating_sub(2),
    )
}

pub struct TeamDialog<'a> {
    pub teams: &'a [String],
    pub selected_index: usize,
}

impl Widget for TeamDialog<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let height = (self.teams.len() + 2).min(area.height as usize) as u16;
        let inner = popup_frame(area, 30, height, " Select Team ", buf);
        let items: Vec<ListItem> = self.teams.iter().map(|t| ListItem::new(t.as_str())).collect();
        let list = List::new(items)
            .highlight_symbol("> ")
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        let mut state = ListState::default();
        state.select(Some(self.selected_index));
        StatefulWidget::render(list, inner, buf, &mut state);
    }
}

pub struct AccountDialog<'a> {
    pub accounts: &'a [AccountEntry],
    pub selected_index: usize,
}

impl Widget for AccountDialog<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let height = (self.accounts.len() + 2).min(area.height as usize) as u16;
        let inner = popup_frame(area, 30, height, " Select Account ", buf);
        let items: Vec<ListItem> = self.accounts.iter().map(|a| ListItem::new(a.label.as_str())).collect();
        let list = List::new(items)
            .highlight_symbol("> ")
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        let mut state = ListState::default();
        state.select(Some(self.selected_index));
        StatefulWidget::render(list, inner, buf, &mut state);
    }
}

pub struct ActionDialog {
    pub selected: ActionChoice,
}

impl Widget for ActionDialog {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = popup_frame(area, 24, 6, " Action ", buf);
        let items = vec![
            ListItem::new("  Execute now"),
            ListItem::new("  Schedule for later"),
        ];
        let selected_index = match self.selected {
            ActionChoice::ExecuteNow => 0,
            ActionChoice::ScheduleLater => 1,
        };
        let list = List::new(items)
            .highlight_symbol("> ")
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        let mut state = ListState::default();
        state.select(Some(selected_index));
        StatefulWidget::render(list, inner, buf, &mut state);
    }
}

pub struct BlockedReasonDialog<'a> {
    pub spec_name: &'a str,
    pub reason: &'a str,
}

impl Widget for BlockedReasonDialog<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = popup_frame(area, 60, 9, " Blocked ", buf);
        let lines = vec![
            Line::from(format!("  {}", self.spec_name)),
            Line::from(""),
            Line::from(format!("  {}", self.reason)),
            Line::from(""),
            Line::from(Span::styled(
                "          [Esc] Dismiss",
                Style::default().add_modifier(Modifier::DIM),
            )),
        ];
        Paragraph::new(lines).render(inner, buf);
    }
}

pub struct CancelDialog<'a> {
    pub spec_slug: &'a str,
    pub team: &'a str,
    pub at: DateTime<Local>,
}

impl Widget for CancelDialog<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = popup_frame(area, 50, 7, " Cancel Scheduled Run ", buf);
        let time_str = self.at.format("%a %b %-d %-I:%M%P").to_string();
        let lines = vec![
            Line::from(format!("  Spec:  {}", self.spec_slug)),
            Line::from(format!("  Team:  {}", self.team)),
            Line::from(format!("  Time:  {}", time_str)),
            Line::from(""),
            Line::from(Span::styled(
                "  > Cancel Scheduled Run",
                Style::default().add_modifier(Modifier::REVERSED),
            )),
        ];
        Paragraph::new(lines).render(inner, buf);
    }
}
