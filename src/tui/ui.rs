use std::env;
use std::io;
use std::path::PathBuf;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use super::app::{ActionChoice, App, OPTIONS_ITEMS, Panel, PopupAction, Screen, SchedulePickerState, SpecTab, TuiResult};
use super::metrics::{MetricsState, render_metrics};
use super::schedule_picker::render_schedule_picker;
use crate::config::{SpecEntry, SpecStatus};
use crate::metrics::db::init_db;
use crate::metrics::query::fetch_runs;
use crate::prefs::Prefs;

/// Run the TUI, returning the user's selection or None if they quit.
pub fn run_tui(
    specs: Vec<SpecEntry>,
    teams: Vec<String>,
    default_team: &str,
) -> Result<Option<TuiResult>> {
    let prefs = Prefs::load();
    let mut app = App::new(specs, teams, default_team, prefs);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let loop_result = run_event_loop(&mut terminal, &mut app);

    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();

    loop_result?;
    Ok(app.result())
}

fn run_event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()>
where
    B::Error: Send + Sync + 'static,
{
    loop {
        terminal.draw(|f| render(f, app))?;

        if let Event::Key(key) = event::read()? {
            match app.screen {
                Screen::Launcher if app.popup.is_some() => match key.code {
                    KeyCode::Up => app.popup_move_up(),
                    KeyCode::Down => app.popup_move_down(),
                    KeyCode::Enter => app.confirm_popup(),
                    KeyCode::Esc => app.dismiss_popup(),
                    _ => {}
                },
                Screen::Launcher => match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        app.should_quit = true;
                    }
                    KeyCode::Char('m') | KeyCode::Char('M') => {
                        load_metrics(app);
                    }
                    KeyCode::Tab => app.next_panel(),
                    KeyCode::Left | KeyCode::Right
                        if app.focused_panel == Panel::Spec =>
                    {
                        app.switch_tab()
                    }
                    KeyCode::Up => app.move_up(),
                    KeyCode::Down => app.move_down(),
                    KeyCode::Char(' ') if app.focused_panel == Panel::Options => {
                        app.toggle_option()
                    }
                    KeyCode::Enter => app.confirm(),
                    _ => {}
                },
                Screen::Metrics => match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                        app.close_metrics();
                    }
                    KeyCode::Up => app.move_up(),
                    KeyCode::Down => app.move_down(),
                    _ => {}
                },
                Screen::SchedulePicker => match key.code {
                    KeyCode::Esc => {
                        app.screen = Screen::Launcher;
                        app.picker = SchedulePickerState::default();
                    }
                    KeyCode::Tab => app.picker.next_field(),
                    KeyCode::BackTab => app.picker.prev_field(),
                    KeyCode::Up => app.picker.increment(),
                    KeyCode::Down => app.picker.decrement(),
                    KeyCode::Enter => app.confirm_picker(),
                    _ => {}
                },
            }
        }

        if app.should_quit || app.confirmed {
            break;
        }
    }
    Ok(())
}

fn load_metrics(app: &mut App) {
    if app.metrics_state.is_some() {
        app.screen = Screen::Metrics;
        return;
    }

    let db_path = env::var_os("HOME").map(|h| {
        PathBuf::from(h)
            .join(".claude")
            .join("claude-agent-team-metrics.db")
    });

    let state = match db_path {
        Some(path) if path.exists() => match rusqlite::Connection::open(&path) {
            Ok(conn) => {
                if init_db(&conn).is_err() {
                    MetricsState::with_error("Failed to initialize database schema".into())
                } else {
                    match fetch_runs(&conn) {
                        Ok(runs) => MetricsState::new(runs),
                        Err(e) => MetricsState::with_error(format!("Failed to load runs: {e}")),
                    }
                }
            }
            Err(e) => MetricsState::with_error(format!("Failed to open database: {e}")),
        },
        _ => MetricsState::new(vec![]),
    };

    app.open_metrics(state);
}

fn render(f: &mut ratatui::Frame, app: &mut App) {
    match app.screen {
        Screen::Metrics => {
            if let Some(ref mut state) = app.metrics_state {
                render_metrics(f, state);
            }
            return;
        }
        Screen::Launcher => {}
        Screen::SchedulePicker => {
            let spec_name = app.visible_specs()
                .get(app.spec_index)
                .map(|s| s.name.as_str())
                .unwrap_or("");
            render_schedule_picker(f, &app.picker, spec_name);
            return;
        }
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(5), // Options panel: 3 items + 2 borders
            Constraint::Length(1),
        ])
        .split(f.area());

    // --- Spec panel ---
    let tab_title = {
        let specs_label = if app.active_tab == SpecTab::Specs {
            Span::styled(" Specs ", Style::default().add_modifier(Modifier::BOLD))
        } else {
            Span::raw(" Specs ")
        };
        let reqs_label = if app.active_tab == SpecTab::Requirements {
            Span::styled(
                " Raw Inputs ",
                Style::default().add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw(" Raw Inputs ")
        };
        Line::from(vec![specs_label, Span::raw("|"), reqs_label])
    };
    let spec_block = Block::default()
        .borders(Borders::ALL)
        .title(tab_title);

    match app.active_tab {
        SpecTab::Specs => {
            let visible = app.visible_specs();
            if visible.is_empty() {
                let msg = if app.specs.is_empty() {
                    "No specs found"
                } else {
                    "All specs filtered — adjust Options to show more"
                };
                let p = Paragraph::new(msg)
                    .style(Style::default().fg(Color::DarkGray))
                    .block(spec_block);
                f.render_widget(p, chunks[0]);
            } else {
                let spec_items: Vec<ListItem> = visible
                    .iter()
                    .map(|s| {
                        let item = ListItem::new(s.name.as_str());
                        match s.status {
                            SpecStatus::Complete => item.style(Style::default().fg(Color::Green)),
                            SpecStatus::Blocked => item.style(Style::default().fg(Color::Red)),
                            _ => item,
                        }
                    })
                    .collect();
                let spec_list = List::new(spec_items)
                    .block(spec_block)
                    .highlight_symbol("> ");
                let mut spec_state = ListState::default();
                spec_state.select(Some(app.spec_index));
                f.render_stateful_widget(spec_list, chunks[0], &mut spec_state);
            }
        }
        SpecTab::Requirements => {
            if app.requirements.is_empty() {
                let msg = Paragraph::new("No requirements files found")
                    .style(Style::default().fg(Color::DarkGray))
                    .block(spec_block);
                f.render_widget(msg, chunks[0]);
            } else {
                let req_items: Vec<ListItem> = app
                    .requirements
                    .iter()
                    .map(|r| ListItem::new(r.name.as_str()))
                    .collect();
                let req_list = List::new(req_items)
                    .block(spec_block)
                    .highlight_symbol("> ");
                let mut req_state = ListState::default();
                req_state.select(Some(app.requirements_index));
                f.render_stateful_widget(req_list, chunks[0], &mut req_state);
            }
        }
    }

    // --- Options panel ---
    let options_focused = app.focused_panel == Panel::Options;
    let focused_style = Style::default().fg(Color::Yellow);
    let normal_style = Style::default();
    let option_data = [
        ("Headless", app.prefs.headless),
        ("Show Complete", app.prefs.show_complete),
        ("Show Blocked", app.prefs.show_blocked),
    ];
    let option_items: Vec<ListItem> = option_data
        .iter()
        .map(|(label, checked)| {
            let checkbox = if *checked { "[x]" } else { "[ ]" };
            ListItem::new(format!("{checkbox} {label}"))
        })
        .collect();
    let options_list = List::new(option_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Options")
                .border_style(if options_focused { focused_style } else { normal_style }),
        )
        .highlight_symbol("> ")
        .highlight_style(if options_focused {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        });
    let mut options_state = ListState::default();
    if options_focused {
        options_state.select(Some(app.options_index));
    }
    f.render_stateful_widget(options_list, chunks[1], &mut options_state);

    // --- Footer ---
    let footer_text = match app.focused_panel {
        Panel::Spec => "  ↑↓ navigate  ←→ switch tab  Tab panel  Enter confirm  q quit",
        Panel::Options => "  ↑↓ navigate  Space toggle  Tab panel  q quit",
    };
    f.render_widget(Paragraph::new(Line::from(footer_text)), chunks[2]);

    // --- Team popup overlay ---
    if let Some(PopupAction::TeamDialog { selected_index }) = app.popup {
        let area = f.area();
        let popup_width = 30u16;
        let popup_height = (app.teams.len() + 2).min(area.height as usize) as u16;
        let x = area.width.saturating_sub(popup_width) / 2;
        let y = area.height.saturating_sub(popup_height) / 2;
        let popup_area = Rect::new(x, y, popup_width.min(area.width), popup_height.min(area.height));

        f.render_widget(Clear, popup_area);
        f.render_widget(
            Block::default().borders(Borders::ALL).title(" Select Team "),
            popup_area,
        );

        let inner = Rect::new(
            popup_area.x + 1,
            popup_area.y + 1,
            popup_area.width.saturating_sub(2),
            popup_area.height.saturating_sub(2),
        );
        let items: Vec<ListItem> = app.teams.iter().map(|t| ListItem::new(t.as_str())).collect();
        let list = List::new(items)
            .highlight_symbol("> ")
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        let mut list_state = ListState::default();
        list_state.select(Some(selected_index));
        f.render_stateful_widget(list, inner, &mut list_state);
        return;
    }

    // --- Action popup overlay ---
    if let Some(PopupAction::ActionDialog { ref selected }) = app.popup {
        let area = f.area();
        let popup_width = 24u16;
        let popup_height = 6u16;
        let x = area.width.saturating_sub(popup_width) / 2;
        let y = area.height.saturating_sub(popup_height) / 2;
        let popup_area = Rect::new(x, y, popup_width.min(area.width), popup_height.min(area.height));

        f.render_widget(Clear, popup_area);
        f.render_widget(
            Block::default().borders(Borders::ALL).title(" Action "),
            popup_area,
        );

        let inner = Rect::new(popup_area.x + 1, popup_area.y + 1, popup_area.width.saturating_sub(2), popup_area.height.saturating_sub(2));
        let items: Vec<ListItem> = vec![
            ListItem::new("  Execute now"),
            ListItem::new("  Schedule for later"),
        ];
        let selected_index = match selected {
            ActionChoice::ExecuteNow => 0,
            ActionChoice::ScheduleLater => 1,
        };
        let list = List::new(items)
            .highlight_symbol("> ")
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        let mut list_state = ListState::default();
        list_state.select(Some(selected_index));
        f.render_stateful_widget(list, inner, &mut list_state);
    }
}
