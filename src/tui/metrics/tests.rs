use super::*;
use crate::metrics::query::RunSummary;

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
    let runs = vec![sample_run("a"), sample_run("b"), sample_run("c")];
    let mut state = MetricsState::new(runs);
    state.scroll_down();
    assert_eq!(state.scroll_offset, 1);
}

#[test]
fn test_scroll_down_clamps_at_last_row() {
    let runs = vec![sample_run("a"), sample_run("b")];
    let mut state = MetricsState::new(runs);
    state.scroll_down();
    state.scroll_down();
    state.scroll_down(); // should clamp
    assert_eq!(state.scroll_offset, 1); // max is len - 1
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
