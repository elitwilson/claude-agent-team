use super::*;
use crate::config::{SpecEntry, SpecStatus};

fn spec(name: &str) -> SpecEntry {
    SpecEntry {
        name: name.to_string(),
        status: SpecStatus::Ready,
    }
}

fn sample_app() -> App {
    App::new(
        vec![
            spec("feature-a.md"),
            spec("feature-b.md"),
            spec("feature-c.md"),
        ],
        vec!["feature-dev".into(), "review-only".into()],
        "feature-dev",
    )
}

// --- Construction ---

#[test]
fn test_new_sets_initial_state() {
    let app = sample_app();
    assert_eq!(app.specs.len(), 3);
    assert_eq!(app.teams.len(), 2);
    assert_eq!(app.spec_index, 0);
    assert_eq!(app.focused_panel, Panel::Spec);
    assert!(!app.headless);
    assert!(!app.should_quit);
    assert!(!app.confirmed);
}

#[test]
fn test_new_selects_default_team() {
    let app = App::new(
        vec![spec("spec.md")],
        vec!["review-only".into(), "feature-dev".into()],
        "feature-dev",
    );
    assert_eq!(app.team_index, 1);
}

#[test]
fn test_new_defaults_to_first_team_if_default_not_found() {
    let app = App::new(
        vec![spec("spec.md")],
        vec!["review-only".into(), "feature-dev".into()],
        "nonexistent-team",
    );
    assert_eq!(app.team_index, 0);
}

// --- Panel navigation ---

#[test]
fn test_next_panel_cycles_through_panels() {
    let mut app = sample_app();
    assert_eq!(app.focused_panel, Panel::Spec);
    app.next_panel();
    assert_eq!(app.focused_panel, Panel::Team);
    app.next_panel();
    assert_eq!(app.focused_panel, Panel::RunOptions);
    app.next_panel();
    assert_eq!(app.focused_panel, Panel::Spec);
}

// --- Spec navigation ---

#[test]
fn test_move_down_in_spec_panel() {
    let mut app = sample_app();
    assert_eq!(app.spec_index, 0);
    app.move_down();
    assert_eq!(app.spec_index, 1);
    app.move_down();
    assert_eq!(app.spec_index, 2);
}

#[test]
fn test_move_down_clamps_at_bottom() {
    let mut app = sample_app();
    app.spec_index = 2; // last spec
    app.move_down();
    assert_eq!(app.spec_index, 2);
}

#[test]
fn test_move_up_in_spec_panel() {
    let mut app = sample_app();
    app.spec_index = 2;
    app.move_up();
    assert_eq!(app.spec_index, 1);
}

#[test]
fn test_move_up_clamps_at_top() {
    let mut app = sample_app();
    assert_eq!(app.spec_index, 0);
    app.move_up();
    assert_eq!(app.spec_index, 0);
}

// --- Team navigation ---

#[test]
fn test_move_down_in_team_panel() {
    let mut app = sample_app();
    app.focused_panel = Panel::Team;
    assert_eq!(app.team_index, 0);
    app.move_down();
    assert_eq!(app.team_index, 1);
}

#[test]
fn test_move_up_in_team_panel() {
    let mut app = sample_app();
    app.focused_panel = Panel::Team;
    app.team_index = 1;
    app.move_up();
    assert_eq!(app.team_index, 0);
}

// --- Headless toggle ---

#[test]
fn test_toggle_headless() {
    let mut app = sample_app();
    assert!(!app.headless);
    app.toggle_headless();
    assert!(app.headless);
    app.toggle_headless();
    assert!(!app.headless);
}

// --- Confirm ---

#[test]
fn test_confirm_sets_confirmed_flag() {
    let mut app = sample_app();
    app.confirm();
    assert!(app.confirmed);
}

#[test]
fn test_result_returns_none_when_not_confirmed() {
    let app = sample_app();
    assert!(app.result().is_none());
}

#[test]
fn test_result_returns_selection_when_confirmed() {
    let mut app = sample_app();
    app.spec_index = 1;
    app.team_index = 0;
    app.headless = true;
    app.confirm();

    let result = app.result().unwrap();
    assert_eq!(result.spec, "feature-b.md");
    assert_eq!(result.team, "feature-dev");
    assert!(result.headless);
}

// --- Screen state ---

#[test]
fn test_app_defaults_to_launcher_screen() {
    let app = sample_app();
    assert_eq!(app.screen, Screen::Launcher);
}

#[test]
fn test_app_metrics_state_is_none_by_default() {
    let app = sample_app();
    assert!(app.metrics_state.is_none());
}

// --- Screen switching ---

use crate::metrics::query::RunSummary;
use crate::tui::metrics::MetricsState;

fn sample_metrics_state() -> MetricsState {
    MetricsState::new(vec![RunSummary {
        run_date: "2026-03-27".into(),
        feature_slug: "feat".into(),
        team: "team".into(),
        total_input: 100,
        total_output: 50,
        total_cache: 25,
        exit_code: 0,
    }])
}

#[test]
fn test_open_metrics_switches_screen() {
    let mut app = sample_app();
    app.open_metrics(sample_metrics_state());
    assert_eq!(app.screen, Screen::Metrics);
}

#[test]
fn test_open_metrics_stores_state() {
    let mut app = sample_app();
    app.open_metrics(sample_metrics_state());
    assert!(app.metrics_state.is_some());
    assert_eq!(app.metrics_state.as_ref().unwrap().runs.len(), 1);
}

#[test]
fn test_close_metrics_returns_to_launcher() {
    let mut app = sample_app();
    app.open_metrics(sample_metrics_state());
    app.close_metrics();
    assert_eq!(app.screen, Screen::Launcher);
}

#[test]
fn test_move_up_on_metrics_screen_scrolls() {
    let mut app = sample_app();
    let mut state = sample_metrics_state();
    // Add more runs so we can scroll
    state.runs.push(RunSummary {
        run_date: "2026-03-26".into(),
        feature_slug: "feat2".into(),
        team: "team".into(),
        total_input: 200,
        total_output: 100,
        total_cache: 50,
        exit_code: 0,
    });
    state.scroll_offset = 1;
    app.open_metrics(state);
    app.move_up();
    assert_eq!(app.metrics_state.as_ref().unwrap().scroll_offset, 0);
}

#[test]
fn test_move_down_on_metrics_screen_scrolls() {
    let mut app = sample_app();
    let mut state = sample_metrics_state();
    state.runs.push(RunSummary {
        run_date: "2026-03-26".into(),
        feature_slug: "feat2".into(),
        team: "team".into(),
        total_input: 200,
        total_output: 100,
        total_cache: 50,
        exit_code: 0,
    });
    app.open_metrics(state);
    app.move_down();
    assert_eq!(app.metrics_state.as_ref().unwrap().scroll_offset, 1);
}

// --- Spec status in TUI ---

fn sample_app_with_entries() -> App {
    App::new(
        vec![
            SpecEntry {
                name: "004-active.md".into(),
                status: SpecStatus::Ready,
            },
            SpecEntry {
                name: "005-broken.md".into(),
                status: SpecStatus::NeedsAttention,
            },
        ],
        vec!["feature-dev".into()],
        "feature-dev",
    )
}

#[test]
fn test_app_carries_spec_status() {
    let app = sample_app_with_entries();
    assert_eq!(app.specs.len(), 2);
    assert_eq!(app.specs[0].status, SpecStatus::Ready);
    assert_eq!(app.specs[1].status, SpecStatus::NeedsAttention);
}

#[test]
fn test_result_returns_spec_name_not_entry() {
    let mut app = sample_app_with_entries();
    app.spec_index = 1;
    app.confirm();
    let result = app.result().unwrap();
    assert_eq!(result.spec, "005-broken.md");
}

#[test]
fn test_needs_attention_spec_is_navigable() {
    let mut app = sample_app_with_entries();
    app.move_down();
    assert_eq!(app.spec_index, 1);
    assert_eq!(app.specs[app.spec_index].status, SpecStatus::NeedsAttention);
}

// --- Smoke tests: full navigation flow ---

#[test]
fn test_smoke_full_navigation_m_then_esc() {
    let mut app = sample_app();
    assert_eq!(app.screen, Screen::Launcher);

    // Simulate 'm' → open metrics with seeded data
    let state = MetricsState::new(vec![
        RunSummary {
            run_date: "2026-03-27".into(),
            feature_slug: "auth-fix".into(),
            team: "platform".into(),
            total_input: 3500,
            total_output: 1750,
            total_cache: 800,
            exit_code: 0,
        },
        RunSummary {
            run_date: "2026-03-26".into(),
            feature_slug: "metrics-query".into(),
            team: "feature-dev".into(),
            total_input: 5000,
            total_output: 2500,
            total_cache: 1200,
            exit_code: 1,
        },
    ]);
    app.open_metrics(state);
    assert_eq!(app.screen, Screen::Metrics);
    assert!(app.metrics_state.is_some());
    assert_eq!(app.metrics_state.as_ref().unwrap().runs.len(), 2);

    // Simulate Esc → back to launcher
    app.close_metrics();
    assert_eq!(app.screen, Screen::Launcher);
}

#[test]
fn test_smoke_full_navigation_m_then_q() {
    let mut app = sample_app();
    app.open_metrics(sample_metrics_state());
    assert_eq!(app.screen, Screen::Metrics);

    // Simulate 'q' on metrics → back to launcher
    app.close_metrics();
    assert_eq!(app.screen, Screen::Launcher);
}

#[test]
fn test_smoke_metrics_scroll_navigation() {
    let mut app = sample_app();
    let state = MetricsState::new(vec![
        RunSummary {
            run_date: "2026-03-27".into(),
            feature_slug: "a".into(),
            team: "team".into(),
            total_input: 100,
            total_output: 50,
            total_cache: 25,
            exit_code: 0,
        },
        RunSummary {
            run_date: "2026-03-26".into(),
            feature_slug: "b".into(),
            team: "team".into(),
            total_input: 200,
            total_output: 100,
            total_cache: 50,
            exit_code: 0,
        },
    ]);
    app.open_metrics(state);

    // Scroll down then up
    app.move_down();
    assert_eq!(app.metrics_state.as_ref().unwrap().scroll_offset, 1);
    app.move_up();
    assert_eq!(app.metrics_state.as_ref().unwrap().scroll_offset, 0);
}

#[test]
fn test_smoke_launcher_unchanged_after_metrics_roundtrip() {
    let mut app = sample_app();
    let original_spec_index = app.spec_index;
    let original_team_index = app.team_index;

    app.open_metrics(sample_metrics_state());
    app.close_metrics();

    // Launcher state preserved
    assert_eq!(app.screen, Screen::Launcher);
    assert_eq!(app.spec_index, original_spec_index);
    assert_eq!(app.team_index, original_team_index);
    assert!(!app.should_quit);
    assert!(!app.confirmed);
}
