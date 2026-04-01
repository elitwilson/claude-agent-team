use std::collections::HashMap;
use std::env;
use std::io;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};

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
    widgets::{Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table, TableState},
};

use super::app::{ActionChoice, App, Panel, PopupAction, Screen, SchedulePickerState, SpecRunInfo, SpecTab, TuiResult};
use super::metrics::{MetricsState, render_metrics};
use super::schedule_picker::render_schedule_picker;
use crate::accounts::AccountEntry;
use crate::config::{SpecEntry, SpecStatus};
use crate::metrics::db::{init_db, last_run_for_project};
use crate::metrics::query::fetch_runs;
use crate::prefs::Prefs;
use crate::scheduler;

#[cfg(test)]
mod tests;

/// Run the TUI, returning the user's selection or None if they quit.
pub fn run_tui(
    specs: Vec<SpecEntry>,
    teams: Vec<String>,
    default_team: &str,
    cwd: &Path,
    accounts: Vec<AccountEntry>,
) -> Result<Option<TuiResult>> {
    let prefs = Prefs::load();
    let run_info = load_run_info(cwd);
    let mut app = App::new(specs, teams, default_team, prefs, run_info, cwd.to_path_buf(), accounts);

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
            // Clear any lingering status message on the first keypress after it was set
            app.status_message = None;

            match app.screen {
                Screen::Launcher if matches!(app.popup, Some(PopupAction::CancelDialog { .. })) => {
                    match key.code {
                        KeyCode::Enter => app.confirm_cancel_dialog(),
                        KeyCode::Esc => app.dismiss_popup(),
                        _ => {}
                    }
                }
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

/// Load initial run_info from pending launchd plists and last-run metrics DB.
/// Non-fatal: returns empty map if either source fails or is absent.
fn load_run_info(cwd: &Path) -> HashMap<String, SpecRunInfo> {
    let mut run_info: HashMap<String, SpecRunInfo> = HashMap::new();

    // Load last-run data from metrics DB (lower priority)
    let db_path = env::var_os("HOME").map(|h| {
        PathBuf::from(h)
            .join(".claude")
            .join("claude-agent-team-metrics.db")
    });
    if let Some(path) = db_path.filter(|p| p.exists()) {
        if let Ok(conn) = rusqlite::Connection::open(&path) {
            if init_db(&conn).is_ok() {
                let project = cwd.to_str().unwrap_or("").replace('/', "-");
                if let Ok(last_runs) = last_run_for_project(&conn, &project) {
                    for (slug, lr) in last_runs {
                        run_info.insert(slug, SpecRunInfo::LastRun {
                            team: lr.team,
                            completed_at: lr.completed_at,
                        });
                    }
                }
            }
        }
    }

    // Load pending scheduled runs (higher priority — overwrite LastRun entries)
    if let Ok(pending) = scheduler::list_pending() {
        for sr in pending {
            run_info.insert(sr.spec.clone(), SpecRunInfo::Scheduled {
                team: sr.team.clone(),
                at: sr.scheduled_at,
                plist_path: sr.plist_path.clone(),
            });
        }
    }

    run_info
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
    let focused_style = Style::default().fg(Color::Yellow);
    let normal_style = Style::default();
    let spec_focused = app.focused_panel == Panel::Spec;
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
        .title(tab_title)
        .border_style(if spec_focused { focused_style } else { normal_style });

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
                let rows: Vec<Row> = visible.iter().map(|s| {
                    let slug = s.name.strip_suffix(".md").unwrap_or(&s.name);
                    let status_text = match s.status {
                        SpecStatus::Complete => "complete",
                        SpecStatus::Blocked => "blocked",
                        SpecStatus::Ready => "ready",
                        _ => "",
                    };
                    let (team_cell, date_cell) = match app.run_info.get(slug) {
                        Some(SpecRunInfo::Scheduled { team, at, .. }) => (
                            Cell::from(team.as_str()),
                            Cell::from(at.format("%a %b %-d %-I:%M%P").to_string()),
                        ),
                        Some(SpecRunInfo::LastRun { team, completed_at }) => {
                            let local: DateTime<Local> = DateTime::from(completed_at.clone());
                            let dim = Style::default().add_modifier(Modifier::DIM);
                            (
                                Cell::from(team.as_str()).style(dim),
                                Cell::from(local.format("%b %-d %-I:%M%P").to_string()).style(dim),
                            )
                        }
                        None => (Cell::from(""), Cell::from("")),
                    };
                    let name_style = match s.status {
                        SpecStatus::Complete => Style::default().fg(Color::Green),
                        SpecStatus::Blocked => Style::default().fg(Color::Red),
                        _ => Style::default(),
                    };
                    Row::new(vec![
                        Cell::from(s.name.as_str()).style(name_style),
                        Cell::from(status_text),
                        team_cell,
                        date_cell,
                    ])
                }).collect();

                let header = Row::new(vec![
                    Cell::from("Spec").style(Style::default().add_modifier(Modifier::BOLD)),
                    Cell::from("Status").style(Style::default().add_modifier(Modifier::BOLD)),
                    Cell::from("Team").style(Style::default().add_modifier(Modifier::BOLD)),
                    Cell::from("Date / Time").style(Style::default().add_modifier(Modifier::BOLD)),
                ]);

                let table = Table::new(rows, [
                    Constraint::Min(10),
                    Constraint::Length(10),
                    Constraint::Length(18),
                    Constraint::Length(18),
                ])
                .header(header)
                .block(spec_block)
                .highlight_symbol("> ");

                let mut table_state = TableState::default();
                table_state.select(Some(app.spec_index));
                f.render_stateful_widget(table, chunks[0], &mut table_state);
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
    let footer_widget = if let Some(ref msg) = app.status_message {
        Paragraph::new(Line::from(Span::styled(
            msg.as_str(),
            Style::default().fg(Color::Cyan),
        )))
    } else {
        let footer_text = match app.focused_panel {
            Panel::Spec => "  ↑↓ navigate  ←→ switch tab  Tab panel  Enter confirm  q quit",
            Panel::Options => "  ↑↓ navigate  Space toggle  Tab panel  q quit",
        };
        Paragraph::new(Line::from(footer_text))
    };
    f.render_widget(footer_widget, chunks[2]);

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

    // --- Account picker popup overlay ---
    if let Some(PopupAction::AccountDialog { selected_index }) = app.popup {
        let area = f.area();
        let popup_width = 30u16;
        let popup_height = (app.accounts.len() + 2).min(area.height as usize) as u16;
        let x = area.width.saturating_sub(popup_width) / 2;
        let y = area.height.saturating_sub(popup_height) / 2;
        let popup_area = Rect::new(x, y, popup_width.min(area.width), popup_height.min(area.height));

        f.render_widget(Clear, popup_area);
        f.render_widget(
            Block::default().borders(Borders::ALL).title(" Select Account "),
            popup_area,
        );

        let inner = Rect::new(
            popup_area.x + 1,
            popup_area.y + 1,
            popup_area.width.saturating_sub(2),
            popup_area.height.saturating_sub(2),
        );
        let items: Vec<ListItem> = app.accounts.iter().map(|a| ListItem::new(a.label.as_str())).collect();
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
        return;
    }

    // --- CancelDialog popup overlay ---
    if let Some(PopupAction::CancelDialog { ref spec_slug, ref team, at }) = app.popup.clone() {
        let area = f.area();
        let popup_width = 50u16;
        let popup_height = 7u16;
        let x = area.width.saturating_sub(popup_width) / 2;
        let y = area.height.saturating_sub(popup_height) / 2;
        let popup_area = Rect::new(x, y, popup_width.min(area.width), popup_height.min(area.height));

        f.render_widget(Clear, popup_area);
        f.render_widget(
            Block::default().borders(Borders::ALL).title(" Cancel Scheduled Run "),
            popup_area,
        );

        let inner = Rect::new(
            popup_area.x + 1,
            popup_area.y + 1,
            popup_area.width.saturating_sub(2),
            popup_area.height.saturating_sub(2),
        );

        let time_str = at.format("%a %b %-d %-I:%M%P").to_string();
        let lines = vec![
            Line::from(format!("  Spec:  {}", spec_slug)),
            Line::from(format!("  Team:  {}", team)),
            Line::from(format!("  Time:  {}", time_str)),
            Line::from(""),
            Line::from(Span::styled(
                "  > Cancel Scheduled Run",
                Style::default().add_modifier(Modifier::REVERSED),
            )),
        ];
        f.render_widget(Paragraph::new(lines), inner);
    }
}
