use super::*;
use crate::metrics::query::RunSummary;
use ratatui::{Terminal, backend::TestBackend};

fn sample_run(slug: &str) -> RunSummary {
    RunSummary {
        run_date: "2026-03-27".into(),
        feature_slug: slug.into(),
        team: "team".into(),
        total_input: 100,
        total_output: 50,
        total_cache: 25,
        exit_code: 0,
    }
}

/// Render the metrics screen to a test backend and return the buffer content as a string.
fn render_to_string(state: &mut MetricsState, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| render_metrics(f, state)).unwrap();
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

// --- MetricsState construction ---

#[test]
fn test_new_sets_runs_and_zero_scroll() {
    let runs = vec![sample_run("a"), sample_run("b")];
    let state = MetricsState::new(runs.clone());
    assert_eq!(state.runs.len(), 2);
    assert_eq!(state.scroll_offset, 0);
}

// --- Scroll down ---

#[test]
fn test_scroll_down_increments_offset() {
    // 3 runs, 1 visible row -> can scroll up to offset 2
    let runs = vec![sample_run("a"), sample_run("b"), sample_run("c")];
    let mut state = MetricsState::new(runs);
    state.visible_rows = 1;
    state.scroll_down();
    assert_eq!(state.scroll_offset, 1);
}

#[test]
fn test_scroll_down_noop_when_all_rows_fit() {
    // 2 runs, 2 visible rows -> nothing to scroll
    let runs = vec![sample_run("a"), sample_run("b")];
    let mut state = MetricsState::new(runs);
    state.visible_rows = 2;
    state.scroll_down();
    assert_eq!(state.scroll_offset, 0);
}

#[test]
fn test_scroll_down_clamps_to_last_visible_window() {
    // 5 runs, 3 visible rows -> max offset = 5 - 3 = 2
    let runs = vec![
        sample_run("a"),
        sample_run("b"),
        sample_run("c"),
        sample_run("d"),
        sample_run("e"),
    ];
    let mut state = MetricsState::new(runs);
    state.visible_rows = 3;
    for _ in 0..10 {
        state.scroll_down();
    }
    assert_eq!(state.scroll_offset, 2);
}

#[test]
fn test_scroll_down_noop_when_empty() {
    let mut state = MetricsState::new(vec![]);
    state.scroll_down();
    assert_eq!(state.scroll_offset, 0);
}

// --- Scroll up ---

#[test]
fn test_scroll_up_decrements_offset() {
    let runs = vec![sample_run("a"), sample_run("b"), sample_run("c")];
    let mut state = MetricsState::new(runs);
    state.scroll_offset = 2;
    state.scroll_up();
    assert_eq!(state.scroll_offset, 1);
}

#[test]
fn test_scroll_up_clamps_at_zero() {
    let runs = vec![sample_run("a")];
    let mut state = MetricsState::new(runs);
    assert_eq!(state.scroll_offset, 0);
    state.scroll_up();
    assert_eq!(state.scroll_offset, 0);
}

// --- Render: column headers ---

#[test]
fn test_render_shows_column_headers() {
    let mut state = MetricsState::new(vec![sample_run("feat")]);
    let output = render_to_string(&mut state, 100, 10);
    assert!(output.contains("Date"), "missing Date header");
    assert!(output.contains("Spec"), "missing Spec header");
    assert!(output.contains("Team"), "missing Team header");
    assert!(output.contains("Input"), "missing Input header");
    assert!(output.contains("Output"), "missing Output header");
    assert!(output.contains("Cache"), "missing Cache header");
    assert!(output.contains("Status"), "missing Status header");
}

// --- Render: data row ---

#[test]
fn test_render_shows_run_data() {
    let run = RunSummary {
        run_date: "2026-03-27".into(),
        feature_slug: "auth-fix".into(),
        team: "platform".into(),
        total_input: 3500,
        total_output: 1750,
        total_cache: 800,
        exit_code: 0,
    };
    let mut state = MetricsState::new(vec![run]);
    let output = render_to_string(&mut state, 100, 10);
    assert!(output.contains("2026-03-27"), "missing run date");
    assert!(output.contains("auth-fix"), "missing feature slug");
    assert!(output.contains("platform"), "missing team");
    assert!(output.contains("3500"), "missing input tokens");
    assert!(output.contains("1750"), "missing output tokens");
    assert!(output.contains("800"), "missing cache tokens");
}

// --- Render: exit code symbols ---

#[test]
fn test_render_exit_code_zero_shows_check() {
    let mut run = sample_run("ok-run");
    run.exit_code = 0;
    let mut state = MetricsState::new(vec![run]);
    let output = render_to_string(&mut state, 100, 10);
    assert!(output.contains("✓"), "exit code 0 should render as ✓");
}

#[test]
fn test_render_exit_code_nonzero_shows_cross() {
    let mut run = sample_run("bad-run");
    run.exit_code = 1;
    let mut state = MetricsState::new(vec![run]);
    let output = render_to_string(&mut state, 100, 10);
    assert!(
        output.contains("✗"),
        "non-zero exit code should render as ✗"
    );
}

// --- Render: empty state ---

#[test]
fn test_render_empty_state_shows_message() {
    let mut state = MetricsState::new(vec![]);
    let output = render_to_string(&mut state, 100, 10);
    assert!(
        output.contains("No runs") || output.contains("no runs") || output.contains("empty"),
        "empty state should show a friendly message"
    );
}

// --- Render: error state ---

#[test]
fn test_render_error_state_shows_error_message() {
    let mut state = MetricsState::with_error("Database connection failed".into());
    let output = render_to_string(&mut state, 100, 10);
    assert!(
        output.contains("Database connection failed"),
        "error state should display the error message"
    );
}
