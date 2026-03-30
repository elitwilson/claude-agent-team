use super::*;
use crate::config::{SpecEntry, SpecStatus};
use crate::prefs::Prefs;
use chrono::Timelike;

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
        Prefs::default(),
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
    assert!(!app.prefs.headless);
    assert!(!app.should_quit);
    assert!(!app.confirmed);
}

#[test]
fn test_new_selects_default_team() {
    let app = App::new(
        vec![spec("spec.md")],
        vec!["review-only".into(), "feature-dev".into()],
        "feature-dev",
        Prefs::default(),
    );
    assert_eq!(app.team_index, 1);
}

#[test]
fn test_new_defaults_to_first_team_if_default_not_found() {
    let app = App::new(
        vec![spec("spec.md")],
        vec!["review-only".into(), "feature-dev".into()],
        "nonexistent-team",
        Prefs::default(),
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
    assert_eq!(app.focused_panel, Panel::Options);
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
    app.spec_index = 2;
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

// --- Options panel navigation ---

#[test]
fn test_move_down_in_options_panel() {
    let mut app = sample_app();
    app.focused_panel = Panel::Options;
    assert_eq!(app.options_index, 0);
    app.move_down();
    assert_eq!(app.options_index, 1);
    app.move_down();
    assert_eq!(app.options_index, 2);
}

#[test]
fn test_move_down_in_options_panel_clamps_at_last_item() {
    let mut app = sample_app();
    app.focused_panel = Panel::Options;
    app.options_index = 2; // last item
    app.move_down();
    assert_eq!(app.options_index, 2);
}

#[test]
fn test_move_up_in_options_panel() {
    let mut app = sample_app();
    app.focused_panel = Panel::Options;
    app.options_index = 2;
    app.move_up();
    assert_eq!(app.options_index, 1);
}

#[test]
fn test_move_up_in_options_panel_clamps_at_zero() {
    let mut app = sample_app();
    app.focused_panel = Panel::Options;
    assert_eq!(app.options_index, 0);
    app.move_up();
    assert_eq!(app.options_index, 0);
}

// --- Options toggle ---

#[test]
fn test_toggle_option_headless_at_index_0() {
    let mut app = sample_app();
    app.options_index = 0;
    assert!(!app.prefs.headless);
    app.toggle_option();
    assert!(app.prefs.headless);
}

#[test]
fn test_toggle_option_show_complete_at_index_1() {
    let mut app = sample_app();
    app.options_index = 1;
    assert!(app.prefs.show_complete);
    app.toggle_option();
    assert!(!app.prefs.show_complete);
}

#[test]
fn test_toggle_option_show_blocked_at_index_2() {
    let mut app = sample_app();
    app.options_index = 2;
    assert!(app.prefs.show_blocked);
    app.toggle_option();
    assert!(!app.prefs.show_blocked);
}

// --- visible_specs filter ---

#[test]
fn test_visible_specs_includes_all_by_default() {
    let app = App::new(
        vec![
            SpecEntry {
                name: "a.md".into(),
                status: SpecStatus::Ready,
            },
            SpecEntry {
                name: "b.md".into(),
                status: SpecStatus::Complete,
            },
            SpecEntry {
                name: "c.md".into(),
                status: SpecStatus::Blocked,
            },
        ],
        vec!["feature-dev".into()],
        "feature-dev",
        Prefs::default(), // show_complete=true, show_blocked=true
    );
    assert_eq!(app.visible_specs().len(), 3);
}

#[test]
fn test_visible_specs_hides_complete_when_show_complete_false() {
    let mut prefs = Prefs::default();
    prefs.show_complete = false;
    let app = App::new(
        vec![
            SpecEntry {
                name: "a.md".into(),
                status: SpecStatus::Ready,
            },
            SpecEntry {
                name: "b.md".into(),
                status: SpecStatus::Complete,
            },
        ],
        vec!["feature-dev".into()],
        "feature-dev",
        prefs,
    );
    let visible = app.visible_specs();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].name, "a.md");
}

#[test]
fn test_visible_specs_hides_blocked_when_show_blocked_false() {
    let mut prefs = Prefs::default();
    prefs.show_blocked = false;
    let app = App::new(
        vec![
            SpecEntry {
                name: "a.md".into(),
                status: SpecStatus::Ready,
            },
            SpecEntry {
                name: "b.md".into(),
                status: SpecStatus::Blocked,
            },
        ],
        vec!["feature-dev".into()],
        "feature-dev",
        prefs,
    );
    let visible = app.visible_specs();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].name, "a.md");
}

#[test]
fn test_spec_index_clamped_after_filter_hides_selected_item() {
    // 3 specs, cursor at index 2; hide blocked, which removes the last item
    let app_entries = vec![
        SpecEntry {
            name: "a.md".into(),
            status: SpecStatus::Ready,
        },
        SpecEntry {
            name: "b.md".into(),
            status: SpecStatus::Ready,
        },
        SpecEntry {
            name: "c.md".into(),
            status: SpecStatus::Blocked,
        },
    ];
    let mut app = App::new(
        app_entries,
        vec!["feature-dev".into()],
        "feature-dev",
        Prefs::default(),
    );
    app.spec_index = 2; // pointing at "c.md" (Blocked)
    app.options_index = 2; // Show Blocked toggle
    app.toggle_option(); // hides blocked -> visible list shrinks to 2
    assert!(app.spec_index < app.visible_specs().len()); // index must be in bounds
    assert_eq!(app.spec_index, 1); // clamped to last visible
}

// --- Headless toggle ---

#[test]
fn test_toggle_headless() {
    let mut app = sample_app();
    assert!(!app.prefs.headless);
    app.toggle_headless();
    assert!(app.prefs.headless);
    app.toggle_headless();
    assert!(!app.prefs.headless);
}

// --- Confirm ---

#[test]
fn test_confirm_sets_confirmed_flag() {
    let mut app = sample_app();
    app.confirm();
    app.confirm_popup(); // ExecuteNow selected by default
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
    app.prefs.headless = true;
    app.confirm();
    app.confirm_popup();

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
                status: SpecStatus::Blocked,
            },
        ],
        vec!["feature-dev".into()],
        "feature-dev",
        Prefs::default(),
    )
}

#[test]
fn test_app_carries_spec_status() {
    let app = sample_app_with_entries();
    assert_eq!(app.specs.len(), 2);
    assert_eq!(app.specs[0].status, SpecStatus::Ready);
    assert_eq!(app.specs[1].status, SpecStatus::Blocked);
}

#[test]
fn test_result_returns_spec_name_not_entry() {
    let mut app = sample_app_with_entries();
    app.spec_index = 0;
    app.confirm();
    app.confirm_popup();
    let result = app.result().unwrap();
    assert_eq!(result.spec, "004-active.md");
}

#[test]
fn test_blocked_spec_is_navigable() {
    let mut app = sample_app_with_entries();
    app.move_down();
    assert_eq!(app.spec_index, 1);
    assert_eq!(
        app.visible_specs()[app.spec_index].status,
        SpecStatus::Blocked
    );
}

// --- Requirements tab ---

fn app_with_mixed_entries() -> App {
    App::new(
        vec![
            SpecEntry {
                name: "001-spec.md".into(),
                status: SpecStatus::Ready,
            },
            SpecEntry {
                name: "email.txt".into(),
                status: SpecStatus::Raw,
            },
            SpecEntry {
                name: "002-spec.md".into(),
                status: SpecStatus::Ready,
            },
            SpecEntry {
                name: "notes.md".into(),
                status: SpecStatus::Raw,
            },
        ],
        vec!["feature-dev".into()],
        "feature-dev",
        Prefs::default(),
    )
}

#[test]
fn test_app_splits_raw_entries_into_requirements() {
    let app = app_with_mixed_entries();
    assert_eq!(app.requirements.len(), 2);
    let names: Vec<&str> = app.requirements.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"email.txt"));
    assert!(names.contains(&"notes.md"));
}

#[test]
fn test_app_splits_non_raw_entries_into_specs() {
    let app = app_with_mixed_entries();
    assert_eq!(app.specs.len(), 2);
    let names: Vec<&str> = app.specs.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"001-spec.md"));
    assert!(names.contains(&"002-spec.md"));
}

#[test]
fn test_blocked_spec_is_not_confirmable() {
    let mut app = App::new(
        vec![SpecEntry {
            name: "003-blocked.md".into(),
            status: SpecStatus::Blocked,
        }],
        vec!["feature-dev".into()],
        "feature-dev",
        Prefs::default(),
    );
    app.confirm();
    assert!(!app.confirmed);
    assert!(app.result().is_none());
}

#[test]
fn test_complete_spec_is_not_confirmable() {
    let mut app = App::new(
        vec![SpecEntry {
            name: "done.md".into(),
            status: SpecStatus::Complete,
        }],
        vec!["feature-dev".into()],
        "feature-dev",
        Prefs::default(),
    );
    app.confirm();
    assert!(!app.confirmed);
    assert!(app.result().is_none());
}

#[test]
fn test_switch_tab_toggles_between_specs_and_requirements() {
    let mut app = app_with_mixed_entries();
    assert_eq!(app.active_tab, SpecTab::Specs);
    app.switch_tab();
    assert_eq!(app.active_tab, SpecTab::Requirements);
    app.switch_tab();
    assert_eq!(app.active_tab, SpecTab::Specs);
}

#[test]
fn test_navigation_is_independent_per_tab() {
    let mut app = app_with_mixed_entries();
    app.move_down();
    assert_eq!(app.spec_index, 1);

    app.switch_tab();
    assert_eq!(app.requirements_index, 0);
    app.move_down();
    assert_eq!(app.requirements_index, 1);

    app.switch_tab();
    assert_eq!(app.spec_index, 1);
}

#[test]
fn test_confirm_on_specs_tab_returns_team_run_mode() {
    let mut app = app_with_mixed_entries();
    assert_eq!(app.active_tab, SpecTab::Specs);
    app.confirm();
    app.confirm_popup();
    let result = app.result().unwrap();
    assert_eq!(result.mode, RunMode::TeamRun);
    assert_eq!(result.spec, "001-spec.md");
}

#[test]
fn test_confirm_on_requirements_tab_returns_draft_run_mode() {
    let mut app = app_with_mixed_entries();
    app.switch_tab();
    assert_eq!(app.active_tab, SpecTab::Requirements);
    app.confirm();
    let result = app.result().unwrap();
    assert_eq!(result.mode, RunMode::DraftRun);
    assert_eq!(result.spec, "email.txt");
}

#[test]
fn test_move_down_clamps_within_requirements_tab() {
    let mut app = app_with_mixed_entries();
    app.switch_tab();
    app.requirements_index = 1;
    app.move_down();
    assert_eq!(app.requirements_index, 1);
}

// --- Smoke tests ---

#[test]
fn test_smoke_full_navigation_m_then_esc() {
    let mut app = sample_app();
    assert_eq!(app.screen, Screen::Launcher);

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
    assert_eq!(app.metrics_state.as_ref().unwrap().runs.len(), 2);

    app.close_metrics();
    assert_eq!(app.screen, Screen::Launcher);
}

#[test]
fn test_smoke_full_navigation_m_then_q() {
    let mut app = sample_app();
    app.open_metrics(sample_metrics_state());
    assert_eq!(app.screen, Screen::Metrics);
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

    assert_eq!(app.screen, Screen::Launcher);
    assert_eq!(app.spec_index, original_spec_index);
    assert_eq!(app.team_index, original_team_index);
    assert!(!app.should_quit);
    assert!(!app.confirmed);
}

// --- TuiResult scheduled_at field ---

#[test]
fn test_tui_result_scheduled_at_is_none_for_immediate_run() {
    let mut app = sample_app();
    app.confirm();
    app.confirm_popup(); // ExecuteNow — scheduled_at should remain None
    let result = app.result().unwrap();
    assert!(result.scheduled_at.is_none());
}

#[test]
fn test_tui_result_scheduled_at_is_none_for_draft_run() {
    let mut app = app_with_mixed_entries();
    app.switch_tab();
    app.confirm();
    let result = app.result().unwrap();
    assert!(result.scheduled_at.is_none());
}

#[test]
fn test_tui_result_can_hold_scheduled_datetime() {
    use chrono::Local;
    let mut app = sample_app();
    app.confirm();
    app.confirm_popup();
    let mut result = app.result().unwrap();
    // Verify the field exists and can be set
    let now = Local::now();
    result.scheduled_at = Some(now);
    assert!(result.scheduled_at.is_some());
}

// --- Action popup ---

#[test]
fn test_app_popup_is_none_by_default() {
    let app = sample_app();
    assert!(app.popup.is_none());
}

#[test]
fn test_open_action_popup_on_ready_spec() {
    let mut app = sample_app();
    app.open_action_popup();
    assert!(app.popup.is_some());
    match app.popup.as_ref().unwrap() {
        PopupAction::ActionDialog { selected } => {
            assert_eq!(*selected, ActionChoice::ExecuteNow);
        }
    }
}

#[test]
fn test_action_popup_not_opened_on_requirements_tab() {
    let mut app = app_with_mixed_entries();
    app.switch_tab(); // switch to Requirements
    app.open_action_popup();
    assert!(app.popup.is_none());
}

#[test]
fn test_action_popup_navigate_down_selects_schedule_later() {
    let mut app = sample_app();
    app.open_action_popup();
    app.popup_move_down();
    match app.popup.as_ref().unwrap() {
        PopupAction::ActionDialog { selected } => {
            assert_eq!(*selected, ActionChoice::ScheduleLater);
        }
    }
}

#[test]
fn test_action_popup_navigate_up_selects_execute_now() {
    let mut app = sample_app();
    app.open_action_popup();
    app.popup_move_down(); // go to ScheduleLater
    app.popup_move_up(); // back to ExecuteNow
    match app.popup.as_ref().unwrap() {
        PopupAction::ActionDialog { selected } => {
            assert_eq!(*selected, ActionChoice::ExecuteNow);
        }
    }
}

#[test]
fn test_action_popup_navigate_down_clamps_at_bottom() {
    let mut app = sample_app();
    app.open_action_popup();
    app.popup_move_down(); // ScheduleLater
    app.popup_move_down(); // should stay at ScheduleLater
    match app.popup.as_ref().unwrap() {
        PopupAction::ActionDialog { selected } => {
            assert_eq!(*selected, ActionChoice::ScheduleLater);
        }
    }
}

#[test]
fn test_action_popup_navigate_up_clamps_at_top() {
    let mut app = sample_app();
    app.open_action_popup();
    app.popup_move_up(); // should stay at ExecuteNow
    match app.popup.as_ref().unwrap() {
        PopupAction::ActionDialog { selected } => {
            assert_eq!(*selected, ActionChoice::ExecuteNow);
        }
    }
}

#[test]
fn test_action_popup_escape_dismisses() {
    let mut app = sample_app();
    app.open_action_popup();
    assert!(app.popup.is_some());
    app.dismiss_popup();
    assert!(app.popup.is_none());
    assert!(!app.confirmed);
    assert_eq!(app.screen, Screen::Launcher);
}

#[test]
fn test_action_popup_confirm_execute_now_sets_confirmed() {
    let mut app = sample_app();
    app.open_action_popup();
    // Default is ExecuteNow
    app.confirm_popup();
    assert!(app.popup.is_none());
    assert!(app.confirmed);
}

#[test]
fn test_action_popup_confirm_schedule_later_opens_picker() {
    let mut app = sample_app();
    app.open_action_popup();
    app.popup_move_down(); // select ScheduleLater
    app.confirm_popup();
    assert!(app.popup.is_none());
    assert_eq!(app.screen, Screen::SchedulePicker);
    assert!(!app.confirmed);
}

// --- Schedule picker state and navigation ---

#[test]
fn test_schedule_picker_default_uses_today_at_8pm() {
    let state = SchedulePickerState::default();
    let today = chrono::Local::now().date_naive();
    assert_eq!(state.month, today.month());
    assert_eq!(state.day, today.day());
    assert_eq!(state.year, today.year());
    assert_eq!(state.hour, 8);
    assert_eq!(state.minute, 0);
    assert_eq!(state.am_pm, AmPm::Pm);
    assert_eq!(state.focused, PickerField::Month);
    assert!(state.error.is_none());
}

#[test]
fn test_picker_tab_cycles_forward() {
    let mut state = SchedulePickerState::default();
    assert_eq!(state.focused, PickerField::Month);
    state.next_field();
    assert_eq!(state.focused, PickerField::Day);
    state.next_field();
    assert_eq!(state.focused, PickerField::Year);
    state.next_field();
    assert_eq!(state.focused, PickerField::Hour);
    state.next_field();
    assert_eq!(state.focused, PickerField::Minute);
    state.next_field();
    assert_eq!(state.focused, PickerField::AmPm);
    state.next_field();
    assert_eq!(state.focused, PickerField::Month);
}

#[test]
fn test_picker_shift_tab_cycles_backward() {
    let mut state = SchedulePickerState::default();
    assert_eq!(state.focused, PickerField::Month);
    state.prev_field();
    assert_eq!(state.focused, PickerField::AmPm);
    state.prev_field();
    assert_eq!(state.focused, PickerField::Minute);
    state.prev_field();
    assert_eq!(state.focused, PickerField::Hour);
    state.prev_field();
    assert_eq!(state.focused, PickerField::Year);
    state.prev_field();
    assert_eq!(state.focused, PickerField::Day);
    state.prev_field();
    assert_eq!(state.focused, PickerField::Month);
}

#[test]
fn test_picker_month_increment() {
    let mut state = SchedulePickerState::default();
    state.month = 1;
    state.focused = PickerField::Month;
    state.increment();
    assert_eq!(state.month, 2);
}

#[test]
fn test_picker_month_wraps_12_to_1() {
    let mut state = SchedulePickerState::default();
    state.month = 12;
    state.focused = PickerField::Month;
    state.increment();
    assert_eq!(state.month, 1);
}

#[test]
fn test_picker_month_wraps_1_to_12() {
    let mut state = SchedulePickerState::default();
    state.month = 1;
    state.focused = PickerField::Month;
    state.decrement();
    assert_eq!(state.month, 12);
}

#[test]
fn test_picker_day_increment() {
    let mut state = SchedulePickerState::default();
    state.month = 1; // January, 31 days
    state.day = 15;
    state.focused = PickerField::Day;
    state.increment();
    assert_eq!(state.day, 16);
}

#[test]
fn test_picker_day_wraps_at_month_end() {
    let mut state = SchedulePickerState::default();
    state.month = 1; // January, 31 days
    state.day = 31;
    state.focused = PickerField::Day;
    state.increment();
    assert_eq!(state.day, 1);
}

#[test]
fn test_picker_day_wraps_1_to_month_end() {
    let mut state = SchedulePickerState::default();
    state.month = 1; // January, 31 days
    state.day = 1;
    state.focused = PickerField::Day;
    state.decrement();
    assert_eq!(state.day, 31);
}

#[test]
fn test_picker_year_increment() {
    let mut state = SchedulePickerState::default();
    let current_year = chrono::Local::now().year();
    state.year = current_year;
    state.focused = PickerField::Year;
    state.increment();
    assert_eq!(state.year, current_year + 1);
}

#[test]
fn test_picker_year_clamps_at_upper_bound() {
    let mut state = SchedulePickerState::default();
    let current_year = chrono::Local::now().year();
    state.year = current_year + 5;
    state.focused = PickerField::Year;
    state.increment();
    assert_eq!(state.year, current_year + 5); // no wrap, stays at bound
}

#[test]
fn test_picker_year_clamps_at_lower_bound() {
    let mut state = SchedulePickerState::default();
    let current_year = chrono::Local::now().year();
    state.year = current_year;
    state.focused = PickerField::Year;
    state.decrement();
    assert_eq!(state.year, current_year); // no wrap, stays at bound
}

#[test]
fn test_picker_hour_increment() {
    let mut state = SchedulePickerState::default();
    state.hour = 8;
    state.focused = PickerField::Hour;
    state.increment();
    assert_eq!(state.hour, 9);
}

#[test]
fn test_picker_hour_wraps_12_to_1() {
    let mut state = SchedulePickerState::default();
    state.hour = 12;
    state.focused = PickerField::Hour;
    state.increment();
    assert_eq!(state.hour, 1);
}

#[test]
fn test_picker_hour_wraps_1_to_12() {
    let mut state = SchedulePickerState::default();
    state.hour = 1;
    state.focused = PickerField::Hour;
    state.decrement();
    assert_eq!(state.hour, 12);
}

#[test]
fn test_picker_minute_increment() {
    let mut state = SchedulePickerState::default();
    state.minute = 30;
    state.focused = PickerField::Minute;
    state.increment();
    assert_eq!(state.minute, 31);
}

#[test]
fn test_picker_minute_wraps_59_to_0() {
    let mut state = SchedulePickerState::default();
    state.minute = 59;
    state.focused = PickerField::Minute;
    state.increment();
    assert_eq!(state.minute, 0);
}

#[test]
fn test_picker_minute_wraps_0_to_59() {
    let mut state = SchedulePickerState::default();
    state.minute = 0;
    state.focused = PickerField::Minute;
    state.decrement();
    assert_eq!(state.minute, 59);
}

#[test]
fn test_picker_ampm_toggles_on_increment() {
    let mut state = SchedulePickerState::default();
    state.am_pm = AmPm::Pm;
    state.focused = PickerField::AmPm;
    state.increment();
    assert_eq!(state.am_pm, AmPm::Am);
}

#[test]
fn test_picker_ampm_toggles_on_decrement() {
    let mut state = SchedulePickerState::default();
    state.am_pm = AmPm::Am;
    state.focused = PickerField::AmPm;
    state.decrement();
    assert_eq!(state.am_pm, AmPm::Pm);
}

#[test]
fn test_picker_day_clamped_when_month_changes_to_shorter_month() {
    let mut state = SchedulePickerState::default();
    state.month = 1; // January
    state.day = 31;
    state.year = 2026;
    state.focused = PickerField::Month;
    state.increment(); // February
    assert_eq!(state.month, 2);
    assert_eq!(state.day, 28); // 2026 is not a leap year
}

#[test]
fn test_picker_day_clamped_feb_29_on_leap_year() {
    let mut state = SchedulePickerState::default();
    state.month = 1; // January
    state.day = 31;
    state.year = 2028; // leap year
    state.focused = PickerField::Month;
    state.increment(); // February
    assert_eq!(state.month, 2);
    assert_eq!(state.day, 29); // leap year allows 29
}

#[test]
fn test_picker_day_clamped_30_day_month() {
    let mut state = SchedulePickerState::default();
    state.month = 3; // March, 31 days
    state.day = 31;
    state.focused = PickerField::Month;
    state.increment(); // April, 30 days
    assert_eq!(state.month, 4);
    assert_eq!(state.day, 30);
}

#[test]
fn test_picker_day_clamped_when_year_changes_feb_29() {
    let mut state = SchedulePickerState::default();
    state.month = 2;
    state.day = 29;
    state.year = 2028; // leap year
    state.focused = PickerField::Year;
    state.increment(); // 2029 is not a leap year
    assert_eq!(state.year, 2029);
    assert_eq!(state.day, 28);
}

// --- Validation and 12→24hr conversion ---

#[test]
fn test_validate_12_to_24hr_12am_is_midnight() {
    // 12:00 AM = 00:00
    let mut state = SchedulePickerState::default();
    state.hour = 12;
    state.minute = 0;
    state.am_pm = AmPm::Am;
    let time = state.to_naive_time();
    assert_eq!(time.hour(), 0);
    assert_eq!(time.minute(), 0);
}

#[test]
fn test_validate_12_to_24hr_12pm_is_noon() {
    // 12:00 PM = 12:00
    let mut state = SchedulePickerState::default();
    state.hour = 12;
    state.minute = 0;
    state.am_pm = AmPm::Pm;
    let time = state.to_naive_time();
    assert_eq!(time.hour(), 12);
    assert_eq!(time.minute(), 0);
}

#[test]
fn test_validate_12_to_24hr_1159pm_is_2359() {
    // 11:59 PM = 23:59
    let mut state = SchedulePickerState::default();
    state.hour = 11;
    state.minute = 59;
    state.am_pm = AmPm::Pm;
    let time = state.to_naive_time();
    assert_eq!(time.hour(), 23);
    assert_eq!(time.minute(), 59);
}

#[test]
fn test_validate_12_to_24hr_1am_is_0100() {
    // 1:00 AM = 01:00
    let mut state = SchedulePickerState::default();
    state.hour = 1;
    state.minute = 0;
    state.am_pm = AmPm::Am;
    let time = state.to_naive_time();
    assert_eq!(time.hour(), 1);
    assert_eq!(time.minute(), 0);
}

#[test]
fn test_validate_past_datetime_sets_error() {
    use chrono::{Duration, Local};
    let past = Local::now() - Duration::hours(1);
    let mut state = SchedulePickerState::default();
    state.month = past.month();
    state.day = past.day();
    state.year = past.year();
    // Set time to 1 hour ago via 12hr fields
    let hour_24 = past.hour();
    state.am_pm = if hour_24 < 12 { AmPm::Am } else { AmPm::Pm };
    state.hour = if hour_24 == 0 {
        12
    } else if hour_24 > 12 {
        hour_24 - 12
    } else {
        hour_24
    };
    state.minute = past.minute();

    let result = state.confirm();
    assert!(result.is_none());
    assert!(state.error.is_some());
    assert!(state.error.as_ref().unwrap().contains("future"));
}

#[test]
fn test_validate_less_than_1_minute_future_sets_error() {
    use chrono::{Duration, Local};
    let soon = Local::now() + Duration::seconds(30);
    let mut state = SchedulePickerState::default();
    state.month = soon.month();
    state.day = soon.day();
    state.year = soon.year();
    let hour_24 = soon.hour();
    state.am_pm = if hour_24 < 12 { AmPm::Am } else { AmPm::Pm };
    state.hour = if hour_24 == 0 {
        12
    } else if hour_24 > 12 {
        hour_24 - 12
    } else {
        hour_24
    };
    state.minute = soon.minute();

    let result = state.confirm();
    assert!(result.is_none());
    assert!(state.error.is_some());
}

#[test]
fn test_validate_valid_future_datetime_returns_scheduled_at() {
    use chrono::{Duration, Local};
    let future = Local::now() + Duration::hours(2);
    let mut state = SchedulePickerState::default();
    state.month = future.month();
    state.day = future.day();
    state.year = future.year();
    let hour_24 = future.hour();
    state.am_pm = if hour_24 < 12 { AmPm::Am } else { AmPm::Pm };
    state.hour = if hour_24 == 0 {
        12
    } else if hour_24 > 12 {
        hour_24 - 12
    } else {
        hour_24
    };
    state.minute = future.minute();

    let result = state.confirm();
    assert!(result.is_some());
    assert!(state.error.is_none());
}
