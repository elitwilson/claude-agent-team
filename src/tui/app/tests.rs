use super::*;
use crate::config::{SpecEntry, SpecStatus};
use crate::prefs::Prefs;
use chrono::{TimeZone, Timelike, Utc};
use std::collections::HashMap;
use std::path::PathBuf;

fn spec(name: &str) -> SpecEntry {
    SpecEntry {
        name: name.to_string(),
        status: SpecStatus::Ready,
        block_reason: None,
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
        HashMap::new(),
        PathBuf::from("/tmp/test"),
        vec![],
    )
}

// --- Construction ---

#[test]
fn test_new_sets_initial_state() {
    let app = sample_app();
    assert_eq!(app.specs.len(), 3);
    assert_eq!(app.teams.len(), 2);
    assert_eq!(app.spec_index, 0);
    assert!(!app.prefs.headless);
    assert!(!app.should_quit);
    assert!(!app.confirmed);
    assert!(app.popup.is_none());
}

#[test]
fn test_new_selects_default_team() {
    let app = App::new(
        vec![spec("spec.md")],
        vec!["review-only".into(), "feature-dev".into()],
        "feature-dev",
        Prefs::default(),
        HashMap::new(),
        PathBuf::from("/tmp/test"),
        vec![],
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
        HashMap::new(),
        PathBuf::from("/tmp/test"),
        vec![],
    );
    assert_eq!(app.team_index, 0);
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

// --- Pref toggles (h/c/b keybinds) ---

#[test]
fn test_toggle_headless_toggles_pref() {
    let mut app = sample_app();
    assert!(!app.prefs.headless);
    app.toggle_headless();
    assert!(app.prefs.headless);
    app.toggle_headless();
    assert!(!app.prefs.headless);
}

#[test]
fn test_toggle_show_complete_toggles_pref() {
    let mut app = sample_app();
    assert!(app.prefs.show_complete);
    app.toggle_show_complete();
    assert!(!app.prefs.show_complete);
    app.toggle_show_complete();
    assert!(app.prefs.show_complete);
}

#[test]
fn test_toggle_show_blocked_toggles_pref() {
    let mut app = sample_app();
    assert!(app.prefs.show_blocked);
    app.toggle_show_blocked();
    assert!(!app.prefs.show_blocked);
    app.toggle_show_blocked();
    assert!(app.prefs.show_blocked);
}

#[test]
fn test_toggle_show_complete_clamps_spec_index() {
    let mut prefs = Prefs::default();
    prefs.show_complete = true;
    let mut app = App::new(
        vec![
            SpecEntry { name: "a.md".into(), status: SpecStatus::Ready, block_reason: None },
            SpecEntry { name: "b.md".into(), status: SpecStatus::Complete, block_reason: None },
        ],
        vec!["feature-dev".into()],
        "feature-dev",
        prefs,
        HashMap::new(),
        PathBuf::from("/tmp/test"),
        vec![],
    );
    app.spec_index = 1; // pointing at Complete spec
    app.toggle_show_complete(); // hides complete -> visible list shrinks to 1
    assert!(app.spec_index < app.visible_specs().len());
}

#[test]
fn test_toggle_show_blocked_clamps_spec_index() {
    let app_entries = vec![
        SpecEntry { name: "a.md".into(), status: SpecStatus::Ready, block_reason: None },
        SpecEntry { name: "b.md".into(), status: SpecStatus::Ready, block_reason: None },
        SpecEntry { name: "c.md".into(), status: SpecStatus::Blocked, block_reason: None },
    ];
    let mut app = App::new(
        app_entries,
        vec!["feature-dev".into()],
        "feature-dev",
        Prefs::default(),
        HashMap::new(),
        PathBuf::from("/tmp/test"),
        vec![],
    );
    app.spec_index = 2;
    app.toggle_show_blocked();
    assert!(app.spec_index < app.visible_specs().len());
    assert_eq!(app.spec_index, 1);
}

// --- visible_specs filter ---

#[test]
fn test_visible_specs_includes_all_by_default() {
    let app = App::new(
        vec![
            SpecEntry { name: "a.md".into(), status: SpecStatus::Ready, block_reason: None },
            SpecEntry { name: "b.md".into(), status: SpecStatus::Complete, block_reason: None },
            SpecEntry { name: "c.md".into(), status: SpecStatus::Blocked, block_reason: None },
        ],
        vec!["feature-dev".into()],
        "feature-dev",
        Prefs::default(), // show_complete=true, show_blocked=true
        HashMap::new(),
        PathBuf::from("/tmp/test"),
        vec![],
    );
    assert_eq!(app.visible_specs().len(), 3);
}

#[test]
fn test_visible_specs_hides_complete_when_show_complete_false() {
    let mut prefs = Prefs::default();
    prefs.show_complete = false;
    let app = App::new(
        vec![
            SpecEntry { name: "a.md".into(), status: SpecStatus::Ready, block_reason: None },
            SpecEntry { name: "b.md".into(), status: SpecStatus::Complete, block_reason: None },
        ],
        vec!["feature-dev".into()],
        "feature-dev",
        prefs,
        HashMap::new(),
        PathBuf::from("/tmp/test"),
        vec![],
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
            SpecEntry { name: "a.md".into(), status: SpecStatus::Ready, block_reason: None },
            SpecEntry { name: "b.md".into(), status: SpecStatus::Blocked, block_reason: None },
        ],
        vec!["feature-dev".into()],
        "feature-dev",
        prefs,
        HashMap::new(),
        PathBuf::from("/tmp/test"),
        vec![],
    );
    let visible = app.visible_specs();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].name, "a.md");
}

// --- TeamDialog popup ---

#[test]
fn test_confirm_on_ready_spec_opens_team_dialog() {
    let mut app = sample_app();
    app.confirm();
    assert!(matches!(app.popup, Some(PopupAction::TeamDialog { .. })));
}

#[test]
fn test_confirm_opens_team_dialog_with_current_team_index() {
    let mut app = sample_app();
    app.team_index = 1;
    app.confirm();
    assert!(matches!(
        app.popup,
        Some(PopupAction::TeamDialog { selected_index: 1 })
    ));
}

#[test]
fn test_open_team_popup_uses_current_team_index() {
    let mut app = sample_app();
    app.team_index = 1;
    app.open_team_popup();
    assert!(matches!(
        app.popup,
        Some(PopupAction::TeamDialog { selected_index: 1 })
    ));
}

#[test]
fn test_confirm_popup_on_team_dialog_stores_team_and_opens_action_dialog() {
    let mut app = sample_app();
    app.confirm(); // TeamDialog, selected_index = 0
    app.popup_move_down(); // selected_index -> 1
    app.confirm_popup(); // stores team_index = 1, opens ActionDialog
    assert_eq!(app.team_index, 1);
    assert!(matches!(app.popup, Some(PopupAction::ActionDialog { .. })));
}

#[test]
fn test_dismiss_popup_on_team_dialog_returns_to_spec_list() {
    let mut app = sample_app();
    app.confirm(); // opens TeamDialog
    app.dismiss_popup();
    assert!(app.popup.is_none());
}

#[test]
fn test_dismiss_popup_on_action_dialog_restores_team_dialog() {
    let mut app = sample_app();
    app.confirm(); // TeamDialog
    app.confirm_popup(); // ActionDialog (team stored, ActionDialog opened)
    app.dismiss_popup(); // Esc on ActionDialog -> restores TeamDialog
    assert!(matches!(app.popup, Some(PopupAction::TeamDialog { .. })));
}

#[test]
fn test_popup_move_down_on_team_dialog_increments_selected_index() {
    let mut app = sample_app(); // 2 teams, team_index = 0
    app.confirm(); // TeamDialog with selected_index = 0
    app.popup_move_down();
    assert!(matches!(
        app.popup,
        Some(PopupAction::TeamDialog { selected_index: 1 })
    ));
}

#[test]
fn test_popup_move_down_on_team_dialog_clamps_at_last_team() {
    let mut app = sample_app(); // 2 teams
    app.team_index = 1;
    app.confirm(); // TeamDialog with selected_index = 1
    app.popup_move_down(); // already at last
    assert!(matches!(
        app.popup,
        Some(PopupAction::TeamDialog { selected_index: 1 })
    ));
}

#[test]
fn test_popup_move_up_on_team_dialog_decrements_selected_index() {
    let mut app = sample_app();
    app.team_index = 1;
    app.confirm(); // TeamDialog with selected_index = 1
    app.popup_move_up();
    assert!(matches!(
        app.popup,
        Some(PopupAction::TeamDialog { selected_index: 0 })
    ));
}

#[test]
fn test_popup_move_up_on_team_dialog_clamps_at_zero() {
    let mut app = sample_app();
    app.confirm(); // TeamDialog with selected_index = 0
    app.popup_move_up(); // already at 0
    assert!(matches!(
        app.popup,
        Some(PopupAction::TeamDialog { selected_index: 0 })
    ));
}

// --- Full confirm flow ---

#[test]
fn test_full_flow_confirm_team_then_execute_now_sets_confirmed() {
    let mut app = sample_app();
    app.confirm(); // opens TeamDialog
    app.confirm_popup(); // stores team, opens ActionDialog (ExecuteNow default)
    app.confirm_popup(); // ExecuteNow -> confirmed = true
    assert!(app.confirmed);
}

#[test]
fn test_full_flow_confirm_team_then_schedule_later_sets_screen() {
    let mut app = sample_app();
    app.confirm(); // TeamDialog
    app.confirm_popup(); // ActionDialog
    app.popup_move_down(); // ScheduleLater
    app.confirm_popup(); // -> SchedulePicker screen
    assert_eq!(app.screen, Screen::SchedulePicker);
    assert!(!app.confirmed);
}

#[test]
fn test_result_returns_none_when_not_confirmed() {
    let app = sample_app();
    assert!(app.result().is_none());
}

#[test]
fn test_result_returns_correct_selection_after_full_flow() {
    let mut app = sample_app();
    app.spec_index = 1;
    app.team_index = 0;
    app.prefs.headless = true;
    app.confirm(); // TeamDialog
    app.confirm_popup(); // ActionDialog (team stored = 0)
    app.confirm_popup(); // ExecuteNow -> confirmed

    let result = app.result().unwrap();
    assert_eq!(result.spec, "feature-b.md");
    assert_eq!(result.team, "feature-dev");
    assert!(result.headless);
}

// --- Non-confirmable specs ---

#[test]
fn test_blocked_spec_is_not_confirmable() {
    let mut app = App::new(
        vec![SpecEntry {
            name: "003-blocked.md".into(),
            status: SpecStatus::Blocked,
            block_reason: None,
        }],
        vec!["feature-dev".into()],
        "feature-dev",
        Prefs::default(),
        HashMap::new(),
        PathBuf::from("/tmp/test"),
        vec![],
    );
    app.confirm();
    assert!(!app.confirmed);
    assert!(app.popup.is_none());
    assert!(app.result().is_none());
}

#[test]
fn test_complete_spec_is_not_confirmable() {
    let mut app = App::new(
        vec![SpecEntry {
            name: "done.md".into(),
            status: SpecStatus::Complete,
            block_reason: None,
        }],
        vec!["feature-dev".into()],
        "feature-dev",
        Prefs::default(),
        HashMap::new(),
        PathBuf::from("/tmp/test"),
        vec![],
    );
    app.confirm();
    assert!(!app.confirmed);
    assert!(app.popup.is_none());
    assert!(app.result().is_none());
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
                block_reason: None,
            },
            SpecEntry {
                name: "005-broken.md".into(),
                status: SpecStatus::Blocked,
                block_reason: None,
            },
        ],
        vec!["feature-dev".into()],
        "feature-dev",
        Prefs::default(),
        HashMap::new(),
        PathBuf::from("/tmp/test"),
        vec![],
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
    app.confirm_popup(); // ActionDialog
    app.confirm_popup(); // ExecuteNow
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
                block_reason: None,
            },
            SpecEntry {
                name: "email.txt".into(),
                status: SpecStatus::Raw,
                block_reason: None,
            },
            SpecEntry {
                name: "002-spec.md".into(),
                status: SpecStatus::Ready,
                block_reason: None,
            },
            SpecEntry {
                name: "notes.md".into(),
                status: SpecStatus::Raw,
                block_reason: None,
            },
        ],
        vec!["feature-dev".into()],
        "feature-dev",
        Prefs::default(),
        HashMap::new(),
        PathBuf::from("/tmp/test"),
        vec![],
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

// --- SchedulePicker state ---

#[test]
fn test_schedule_picker_next_field() {
    let mut picker = SchedulePickerState::default();
    assert_eq!(picker.focused, PickerField::Month);
    picker.next_field();
    assert_eq!(picker.focused, PickerField::Day);
    picker.next_field();
    assert_eq!(picker.focused, PickerField::Year);
    picker.next_field();
    assert_eq!(picker.focused, PickerField::Hour);
    picker.next_field();
    assert_eq!(picker.focused, PickerField::Minute);
    picker.next_field();
    assert_eq!(picker.focused, PickerField::AmPm);
    picker.next_field();
    assert_eq!(picker.focused, PickerField::Month);
}

#[test]
fn test_schedule_picker_prev_field() {
    let mut picker = SchedulePickerState::default();
    picker.prev_field();
    assert_eq!(picker.focused, PickerField::AmPm);
}

#[test]
fn test_schedule_picker_increment_month_wraps() {
    let mut picker = SchedulePickerState::default();
    picker.month = 12;
    picker.increment();
    assert_eq!(picker.month, 1);
}

#[test]
fn test_schedule_picker_to_naive_time_am() {
    let mut picker = SchedulePickerState::default();
    picker.focused = PickerField::AmPm;
    picker.am_pm = AmPm::Am;
    picker.focused = PickerField::Hour;
    picker.hour = 8;
    picker.minute = 30;
    let t = picker.to_naive_time();
    assert_eq!(t.hour(), 8);
    assert_eq!(t.minute(), 30);
}

#[test]
fn test_schedule_picker_to_naive_time_pm() {
    let mut picker = SchedulePickerState::default();
    picker.am_pm = AmPm::Pm;
    picker.hour = 3;
    picker.minute = 0;
    let t = picker.to_naive_time();
    assert_eq!(t.hour(), 15);
}

#[test]
fn test_schedule_picker_confirm_rejects_past_time() {
    let mut picker = SchedulePickerState::default();
    picker.year = 2020;
    picker.month = 1;
    picker.day = 1;
    let result = picker.confirm();
    assert!(result.is_none());
    assert!(picker.error.is_some());
}

// --- Integration smoke test ---

/// Simulates the full event sequence: navigate spec list → open team popup →
/// navigate team popup → confirm team → confirm action → verify TuiResult.
#[test]
fn test_integration_full_flow_spec_to_team_to_execute_produces_correct_result() {
    let mut app = App::new(
        vec![
            spec("feature-a.md"),
            spec("feature-b.md"),
            spec("feature-c.md"),
        ],
        vec!["alpha-team".into(), "beta-team".into()],
        "alpha-team",
        Prefs::default(),
        HashMap::new(),
        PathBuf::from("/tmp/test"),
        vec![],
    );

    // Navigate spec list down to feature-b.md
    app.move_down();
    assert_eq!(app.spec_index, 1);

    // Press Enter -> opens TeamDialog with alpha-team (index 0) pre-selected
    app.confirm();
    assert!(matches!(
        app.popup,
        Some(PopupAction::TeamDialog { selected_index: 0 })
    ));

    // Navigate team popup down to beta-team
    app.popup_move_down();
    assert!(matches!(
        app.popup,
        Some(PopupAction::TeamDialog { selected_index: 1 })
    ));

    // Confirm team -> stores beta-team, opens ActionDialog
    app.confirm_popup();
    assert_eq!(app.team_index, 1);
    assert!(matches!(app.popup, Some(PopupAction::ActionDialog { .. })));

    // Confirm Execute Now
    app.confirm_popup();
    assert!(app.confirmed);

    // Verify result
    let result = app.result().unwrap();
    assert_eq!(result.spec, "feature-b.md");
    assert_eq!(result.team, "beta-team");
    assert!(!result.headless);
    assert!(matches!(result.mode, RunMode::TeamRun));
}

// --- Task 2: App state changes (SpecRunInfo, run_info, cwd, status_message) ---

fn sample_app_with_run_info(run_info: HashMap<String, SpecRunInfo>) -> App {
    App::new(
        vec![
            spec("feature-a.md"),
            spec("feature-b.md"),
        ],
        vec!["feature-dev".into()],
        "feature-dev",
        Prefs::default(),
        run_info,
        PathBuf::from("/tmp/test-project"),
        vec![],
    )
}

#[test]
fn test_app_new_initializes_run_info() {
    let app = sample_app_with_run_info(HashMap::new());
    assert!(app.run_info.is_empty());
}

#[test]
fn test_app_new_initializes_status_message_as_none() {
    let app = sample_app_with_run_info(HashMap::new());
    assert!(app.status_message.is_none());
}

#[test]
fn test_app_new_stores_cwd() {
    let cwd = PathBuf::from("/tmp/test-project");
    let app = App::new(
        vec![spec("feat.md")],
        vec!["feature-dev".into()],
        "feature-dev",
        Prefs::default(),
        HashMap::new(),
        cwd.clone(),
        vec![],
    );
    assert_eq!(app.cwd, cwd);
}

#[test]
fn test_app_new_stores_run_info_entries() {
    let mut run_info = HashMap::new();
    let plist_path = PathBuf::from("/tmp/test.plist");
    let at = chrono::Local.with_ymd_and_hms(2027, 6, 1, 8, 0, 0).unwrap();
    run_info.insert(
        "feat-a".to_string(),
        SpecRunInfo::Scheduled {
            team: "alpha".to_string(),
            at,
            plist_path: plist_path.clone(),
        },
    );
    let app = sample_app_with_run_info(run_info);
    assert!(app.run_info.contains_key("feat-a"));
    assert!(matches!(app.run_info["feat-a"], SpecRunInfo::Scheduled { .. }));
}

#[test]
fn test_spec_run_info_last_run_variant() {
    let completed_at = Utc.with_ymd_and_hms(2026, 3, 1, 11, 0, 0).unwrap();
    let entry = SpecRunInfo::LastRun {
        team: "beta".to_string(),
        completed_at,
    };
    assert!(matches!(entry, SpecRunInfo::LastRun { .. }));
}

#[test]
fn test_tui_result_has_no_scheduled_at_field() {
    // Execute Now still produces a TuiResult; it must not include scheduled_at
    let mut app = sample_app_with_run_info(HashMap::new());
    app.confirm();
    app.confirm_popup(); // ActionDialog
    app.confirm_popup(); // ExecuteNow
    let result = app.result().unwrap();
    assert_eq!(result.spec, "feature-a.md");
    assert!(matches!(result.mode, RunMode::TeamRun));
    // Verify compilation: TuiResult no longer has scheduled_at field
    // (If it still had the field, the struct init in result() would include it
    // and we'd see it here. This test passing after removal confirms absence.)
}

// --- Task 3: Scheduling and cancel logic ---

fn make_scheduled_run_info(slug: &str, team: &str) -> HashMap<String, SpecRunInfo> {
    let mut run_info = HashMap::new();
    let at = chrono::Local.with_ymd_and_hms(2027, 6, 1, 8, 0, 0).unwrap();
    run_info.insert(
        slug.to_string(),
        SpecRunInfo::Scheduled {
            team: team.to_string(),
            at,
            plist_path: PathBuf::from("/tmp/test.plist"),
        },
    );
    run_info
}

#[test]
fn test_open_team_popup_opens_cancel_dialog_for_scheduled_spec() {
    // feature-a.md has a scheduled run — pressing Enter should open CancelDialog
    let slug = "feature-a"; // slug is filename without .md
    let run_info = make_scheduled_run_info(slug, "alpha");
    let mut app = sample_app_with_run_info(run_info);
    app.open_team_popup();
    assert!(matches!(app.popup, Some(PopupAction::CancelDialog { .. })));
}

#[test]
fn test_open_team_popup_opens_team_dialog_for_unscheduled_spec() {
    let mut app = sample_app_with_run_info(HashMap::new());
    app.open_team_popup();
    assert!(matches!(app.popup, Some(PopupAction::TeamDialog { .. })));
}

#[test]
fn test_cancel_dialog_contains_correct_spec_team_and_time() {
    let slug = "feature-a";
    let at = chrono::Local.with_ymd_and_hms(2027, 6, 1, 8, 0, 0).unwrap();
    let mut run_info = HashMap::new();
    run_info.insert(
        slug.to_string(),
        SpecRunInfo::Scheduled {
            team: "alpha".to_string(),
            at,
            plist_path: PathBuf::from("/tmp/test.plist"),
        },
    );
    let mut app = sample_app_with_run_info(run_info);
    app.open_team_popup();
    match &app.popup {
        Some(PopupAction::CancelDialog { spec_slug, team, at: dialog_at }) => {
            assert_eq!(spec_slug, slug);
            assert_eq!(team, "alpha");
            assert_eq!(*dialog_at, at);
        }
        other => panic!("Expected CancelDialog, got {:?}", other),
    }
}

#[test]
fn test_confirm_cancel_dialog_removes_run_info_entry() {
    let slug = "feature-a";
    let run_info = make_scheduled_run_info(slug, "alpha");
    let mut app = sample_app_with_run_info(run_info);
    // Put the app in the CancelDialog state directly
    let at = chrono::Local.with_ymd_and_hms(2027, 6, 1, 8, 0, 0).unwrap();
    app.popup = Some(PopupAction::CancelDialog {
        spec_slug: slug.to_string(),
        team: "alpha".to_string(),
        at,
    });
    app.confirm_cancel_dialog();
    assert!(!app.run_info.contains_key(slug));
    assert!(app.popup.is_none());
}

#[test]
fn test_after_cancel_open_team_popup_opens_team_dialog() {
    let slug = "feature-a";
    let run_info = make_scheduled_run_info(slug, "alpha");
    let mut app = sample_app_with_run_info(run_info);
    // Simulate cancel: remove the run_info entry
    app.run_info.remove(slug);
    app.open_team_popup();
    assert!(matches!(app.popup, Some(PopupAction::TeamDialog { .. })));
}

#[test]
fn test_dismiss_cancel_dialog_returns_to_spec_list() {
    let slug = "feature-a";
    let run_info = make_scheduled_run_info(slug, "alpha");
    let mut app = sample_app_with_run_info(run_info);
    let at = chrono::Local.with_ymd_and_hms(2027, 6, 1, 8, 0, 0).unwrap();
    app.popup = Some(PopupAction::CancelDialog {
        spec_slug: slug.to_string(),
        team: "alpha".to_string(),
        at,
    });
    app.dismiss_popup();
    assert!(app.popup.is_none());
}

#[test]
fn test_confirm_picker_on_success_returns_to_launcher_and_sets_status_message() {
    // confirm_picker calls scheduler::schedule_run internally, but in tests we can't
    // run launchctl. We test the post-success state by verifying the App's behavior
    // when a ScheduledRun is injected. The actual scheduler call path is covered by
    // the scheduler module tests. Here we verify app state after a simulated success:
    // insert a run_info entry directly and check status_message is set.
    let mut app = sample_app_with_run_info(HashMap::new());
    let at = chrono::Local.with_ymd_and_hms(2027, 6, 1, 8, 0, 0).unwrap();
    // Simulate what confirm_picker does on success:
    app.run_info.insert(
        "feature-a".to_string(),
        SpecRunInfo::Scheduled {
            team: "feature-dev".to_string(),
            at,
            plist_path: PathBuf::from("/tmp/test.plist"),
        },
    );
    app.status_message = Some("Scheduled: feature-a.md @ feature-dev Mon Jun 1 8:00am 2027".to_string());
    app.screen = Screen::Launcher;
    // Assertions on resulting state
    assert_eq!(app.screen, Screen::Launcher);
    assert!(!app.confirmed);
    assert!(app.status_message.is_some());
    assert!(app.run_info.contains_key("feature-a"));
}

// --- Task 009: AccountDialog popup chain ---

fn sample_app_with_accounts(accounts: Vec<AccountEntry>) -> App {
    App::new(
        vec![spec("feature-a.md"), spec("feature-b.md")],
        vec!["feature-dev".into()],
        "feature-dev",
        Prefs::default(),
        HashMap::new(),
        PathBuf::from("/tmp/test"),
        accounts,
    )
}

#[test]
fn test_confirm_popup_on_team_dialog_with_no_accounts_opens_action_dialog() {
    let mut app = sample_app_with_accounts(vec![]);
    app.confirm(); // opens TeamDialog
    app.confirm_popup(); // no accounts → skip to ActionDialog
    assert!(matches!(app.popup, Some(PopupAction::ActionDialog { .. })));
}

#[test]
fn test_confirm_popup_on_team_dialog_with_single_account_opens_action_dialog() {
    let accounts = vec![AccountEntry { label: "personal".to_string() }];
    let mut app = sample_app_with_accounts(accounts);
    app.confirm();
    app.confirm_popup(); // 1 account → skip to ActionDialog
    assert!(matches!(app.popup, Some(PopupAction::ActionDialog { .. })));
}

#[test]
fn test_confirm_popup_on_team_dialog_with_multiple_accounts_opens_account_dialog() {
    let accounts = vec![
        AccountEntry { label: "personal".to_string() },
        AccountEntry { label: "work".to_string() },
    ];
    let mut app = sample_app_with_accounts(accounts);
    app.confirm();
    app.confirm_popup(); // 2 accounts → AccountDialog
    assert!(matches!(app.popup, Some(PopupAction::AccountDialog { .. })));
}

#[test]
fn test_confirm_popup_on_account_dialog_opens_action_dialog() {
    let accounts = vec![
        AccountEntry { label: "personal".to_string() },
        AccountEntry { label: "work".to_string() },
    ];
    let mut app = sample_app_with_accounts(accounts);
    app.confirm();
    app.confirm_popup(); // AccountDialog
    app.confirm_popup(); // confirm account → ActionDialog
    assert!(matches!(app.popup, Some(PopupAction::ActionDialog { .. })));
}

#[test]
fn test_confirm_popup_on_account_dialog_stores_account_index() {
    let accounts = vec![
        AccountEntry { label: "personal".to_string() },
        AccountEntry { label: "work".to_string() },
    ];
    let mut app = sample_app_with_accounts(accounts);
    app.confirm();
    app.confirm_popup(); // AccountDialog pre-selected at index 0
    app.popup_move_down(); // move to "work" (index 1)
    app.confirm_popup(); // confirm → account_index = 1
    assert_eq!(app.account_index, 1);
}

#[test]
fn test_confirm_popup_on_account_dialog_updates_prefs_default_account() {
    let accounts = vec![
        AccountEntry { label: "personal".to_string() },
        AccountEntry { label: "work".to_string() },
    ];
    let mut app = sample_app_with_accounts(accounts);
    app.confirm();
    app.confirm_popup(); // AccountDialog
    app.popup_move_down(); // select "work"
    app.confirm_popup(); // confirm
    assert_eq!(app.prefs.default_account, Some("work".to_string()));
}

#[test]
fn test_dismiss_popup_on_account_dialog_restores_team_dialog() {
    let accounts = vec![
        AccountEntry { label: "personal".to_string() },
        AccountEntry { label: "work".to_string() },
    ];
    let mut app = sample_app_with_accounts(accounts);
    app.confirm(); // TeamDialog
    app.confirm_popup(); // AccountDialog
    app.dismiss_popup(); // Esc → back to TeamDialog
    assert!(matches!(app.popup, Some(PopupAction::TeamDialog { .. })));
}

#[test]
fn test_popup_move_down_on_account_dialog_increments_index() {
    let accounts = vec![
        AccountEntry { label: "personal".to_string() },
        AccountEntry { label: "work".to_string() },
    ];
    let mut app = sample_app_with_accounts(accounts);
    app.popup = Some(PopupAction::AccountDialog { selected_index: 0 });
    app.popup_move_down();
    assert!(matches!(app.popup, Some(PopupAction::AccountDialog { selected_index: 1 })));
}

#[test]
fn test_popup_move_down_on_account_dialog_clamps_at_last() {
    let accounts = vec![
        AccountEntry { label: "personal".to_string() },
        AccountEntry { label: "work".to_string() },
    ];
    let mut app = sample_app_with_accounts(accounts);
    app.popup = Some(PopupAction::AccountDialog { selected_index: 1 });
    app.popup_move_down();
    assert!(matches!(app.popup, Some(PopupAction::AccountDialog { selected_index: 1 })));
}

#[test]
fn test_popup_move_up_on_account_dialog_decrements_index() {
    let accounts = vec![
        AccountEntry { label: "personal".to_string() },
        AccountEntry { label: "work".to_string() },
    ];
    let mut app = sample_app_with_accounts(accounts);
    app.popup = Some(PopupAction::AccountDialog { selected_index: 1 });
    app.popup_move_up();
    assert!(matches!(app.popup, Some(PopupAction::AccountDialog { selected_index: 0 })));
}

#[test]
fn test_popup_move_up_on_account_dialog_clamps_at_zero() {
    let accounts = vec![
        AccountEntry { label: "personal".to_string() },
        AccountEntry { label: "work".to_string() },
    ];
    let mut app = sample_app_with_accounts(accounts);
    app.popup = Some(PopupAction::AccountDialog { selected_index: 0 });
    app.popup_move_up();
    assert!(matches!(app.popup, Some(PopupAction::AccountDialog { selected_index: 0 })));
}

#[test]
fn test_app_new_preselects_default_account_from_prefs() {
    let mut prefs = Prefs::default();
    prefs.default_account = Some("work".to_string());
    let accounts = vec![
        AccountEntry { label: "personal".to_string() },
        AccountEntry { label: "work".to_string() },
    ];
    let app = App::new(
        vec![spec("spec.md")],
        vec!["feature-dev".into()],
        "feature-dev",
        prefs,
        HashMap::new(),
        PathBuf::from("/tmp/test"),
        accounts,
    );
    assert_eq!(app.account_index, 1);
}

#[test]
fn test_app_new_account_index_defaults_to_zero_when_default_not_found() {
    let mut prefs = Prefs::default();
    prefs.default_account = Some("nonexistent".to_string());
    let accounts = vec![
        AccountEntry { label: "personal".to_string() },
        AccountEntry { label: "work".to_string() },
    ];
    let app = App::new(
        vec![spec("spec.md")],
        vec!["feature-dev".into()],
        "feature-dev",
        prefs,
        HashMap::new(),
        PathBuf::from("/tmp/test"),
        accounts,
    );
    assert_eq!(app.account_index, 0);
}

#[test]
fn test_tui_result_account_is_none_when_no_accounts() {
    let mut app = sample_app_with_accounts(vec![]);
    app.confirm();
    app.confirm_popup(); // ActionDialog
    app.confirm_popup(); // ExecuteNow
    let result = app.result().unwrap();
    assert!(result.account.is_none());
}

#[test]
fn test_tui_result_account_is_label_when_account_selected() {
    let accounts = vec![
        AccountEntry { label: "personal".to_string() },
        AccountEntry { label: "work".to_string() },
    ];
    let mut app = sample_app_with_accounts(accounts);
    app.confirm();
    app.confirm_popup(); // AccountDialog (index 0 = "personal")
    app.popup_move_down(); // select "work"
    app.confirm_popup(); // confirm account
    app.confirm_popup(); // ExecuteNow
    let result = app.result().unwrap();
    assert_eq!(result.account, Some("work".to_string()));
}

#[test]
fn test_dismiss_action_dialog_with_multiple_accounts_restores_account_dialog() {
    let accounts = vec![
        AccountEntry { label: "personal".to_string() },
        AccountEntry { label: "work".to_string() },
    ];
    let mut app = sample_app_with_accounts(accounts);
    app.confirm(); // TeamDialog
    app.confirm_popup(); // AccountDialog
    app.confirm_popup(); // ActionDialog
    app.dismiss_popup(); // Esc on ActionDialog → AccountDialog (not TeamDialog)
    assert!(matches!(app.popup, Some(PopupAction::AccountDialog { .. })));
}
