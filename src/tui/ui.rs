use std::io;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};

use super::app::{App, Panel, TuiResult};

/// Run the TUI, returning the user's selection or None if they quit.
pub fn run_tui(
    specs: Vec<String>,
    teams: Vec<String>,
    default_team: &str,
) -> Result<Option<TuiResult>> {
    let mut app = App::new(specs, teams, default_team);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let loop_result = run_event_loop(&mut terminal, &mut app);

    // Restore terminal unconditionally before propagating any error.
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
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    app.should_quit = true;
                }
                KeyCode::Tab => app.next_panel(),
                KeyCode::Up => app.move_up(),
                KeyCode::Down => app.move_down(),
                KeyCode::Char(' ') => app.toggle_headless(),
                KeyCode::Enter => app.confirm(),
                _ => {}
            }
        }

        if app.should_quit || app.confirmed {
            break;
        }
    }
    Ok(())
}

fn render(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(f.area());

    let focused_style = Style::default().fg(Color::Yellow);
    let normal_style = Style::default();

    // Spec panel
    let spec_items: Vec<ListItem> = app.specs.iter().map(|s| ListItem::new(s.as_str())).collect();
    let spec_list = List::new(spec_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Spec")
                .border_style(if app.focused_panel == Panel::Spec {
                    focused_style
                } else {
                    normal_style
                }),
        )
        .highlight_symbol("> ");
    let mut spec_state = ListState::default();
    spec_state.select(Some(app.spec_index));
    f.render_stateful_widget(spec_list, chunks[0], &mut spec_state);

    // Team panel
    let team_items: Vec<ListItem> = app.teams.iter().map(|t| ListItem::new(t.as_str())).collect();
    let team_list = List::new(team_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Team")
                .border_style(if app.focused_panel == Panel::Team {
                    focused_style
                } else {
                    normal_style
                }),
        )
        .highlight_symbol("> ");
    let mut team_state = ListState::default();
    team_state.select(Some(app.team_index));
    f.render_stateful_widget(team_list, chunks[1], &mut team_state);

    // Run options panel
    let headless_label = if app.headless { "[x] Headless" } else { "[ ] Headless" };
    let opts = Paragraph::new(headless_label).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Run Options")
            .border_style(if app.focused_panel == Panel::RunOptions {
                focused_style
            } else {
                normal_style
            }),
    );
    f.render_widget(opts, chunks[2]);

    // Footer
    let footer = Paragraph::new(Line::from(
        "  \u{2191}\u{2193} navigate  Tab switch panel  Space toggle  Enter confirm  q quit",
    ));
    f.render_widget(footer, chunks[3]);
}
