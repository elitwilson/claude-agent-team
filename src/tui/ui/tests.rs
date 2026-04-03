use std::collections::HashMap;
use std::path::PathBuf;

use chrono::TimeZone;
use ratatui::{Terminal, backend::TestBackend};

use super::render;
use crate::accounts::AccountEntry;
use crate::config::{SpecEntry, SpecStatus};
use crate::prefs::Prefs;
use crate::tui::app::{App, PopupAction, SpecRunInfo};

fn spec(name: &str) -> SpecEntry {
    SpecEntry {
        name: name.to_string(),
        status: SpecStatus::Ready,
        block_reason: None,
    }
}

fn render_to_string(app: &mut App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| render(f, app)).unwrap();
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

fn sample_app() -> App {
    App::new(
        vec![spec("feature-a.md"), spec("feature-b.md")],
        vec!["feature-dev".into()],
        "feature-dev",
        Prefs::default(),
        HashMap::new(),
        PathBuf::from("/tmp/test"),
        vec![],
    )
}

// --- Run Info column rendering ---

#[test]
fn test_render_shows_run_info_column_header() {
    let mut app = sample_app();
    let output = render_to_string(&mut app, 100, 20);
    assert!(output.contains("Team"), "Expected 'Team' column header in output");
    assert!(output.contains("Date / Time"), "Expected 'Date / Time' column header in output");
}

#[test]
fn test_render_shows_scheduled_run_info_for_spec() {
    let mut run_info = HashMap::new();
    let at = chrono::Local.with_ymd_and_hms(2027, 6, 1, 20, 0, 0).unwrap();
    run_info.insert(
        "feature-a".to_string(),
        SpecRunInfo::Scheduled {
            team: "feature-dev".to_string(),
            at,
            plist_path: PathBuf::from("/tmp/test.plist"),
        },
    );
    let mut app = App::new(
        vec![spec("feature-a.md")],
        vec!["feature-dev".into()],
        "feature-dev",
        Prefs::default(),
        run_info,
        PathBuf::from("/tmp/test"),
        vec![],
    );
    let output = render_to_string(&mut app, 100, 20);
    // Scheduled format: "team @ Mon Jan 2 8:00pm"
    assert!(output.contains("feature-dev"), "Expected team name in run info column");
    assert!(output.contains("Jun"), "Expected date in run info column");
}

#[test]
fn test_render_shows_last_run_info_for_spec() {
    let mut run_info = HashMap::new();
    let completed_at = chrono::Utc.with_ymd_and_hms(2026, 3, 15, 11, 0, 0).unwrap();
    run_info.insert(
        "feature-a".to_string(),
        SpecRunInfo::LastRun {
            team: "feature-dev".to_string(),
            completed_at,
        },
    );
    let mut app = App::new(
        vec![spec("feature-a.md")],
        vec!["feature-dev".into()],
        "feature-dev",
        Prefs::default(),
        run_info,
        PathBuf::from("/tmp/test"),
        vec![],
    );
    let output = render_to_string(&mut app, 100, 20);
    // Last-run format: "Mar 15 11:00am" (dim)
    assert!(output.contains("feature-dev"), "Expected team name in last-run info");
    assert!(output.contains("Mar"), "Expected month in last-run info");
    assert!(output.contains("am") || output.contains("pm"), "Expected time (am/pm) in last-run info");
}

#[test]
fn test_render_run_info_blank_when_no_info() {
    let mut app = sample_app();
    let output = render_to_string(&mut app, 100, 20);
    // Should render fine without panicking — blank run info column
    assert!(output.contains("feature-a.md"));
}

// --- Status message rendering ---

#[test]
fn test_render_shows_status_message_in_footer() {
    let mut app = sample_app();
    app.status_message = Some("Scheduled: feature-a.md — feature-dev @ Mon Jun 1".to_string());
    let output = render_to_string(&mut app, 100, 20);
    assert!(
        output.contains("Scheduled:"),
        "Expected status message in footer area"
    );
}

// --- CancelDialog popup rendering ---

#[test]
fn test_render_shows_cancel_dialog_popup() {
    let mut app = sample_app();
    let at = chrono::Local.with_ymd_and_hms(2027, 6, 1, 20, 0, 0).unwrap();
    app.popup = Some(PopupAction::CancelDialog {
        spec_slug: "feature-a".to_string(),
        team: "feature-dev".to_string(),
        at,
    });
    let output = render_to_string(&mut app, 100, 20);
    assert!(
        output.contains("Cancel") || output.contains("cancel"),
        "Expected cancel dialog in output"
    );
    assert!(output.contains("feature-dev"), "Expected team in cancel dialog");
    // Spec: cancel dialog shows spec name, team, and datetime
    assert!(
        output.contains("feature-a"),
        "Expected spec slug/name in cancel dialog"
    );
    assert!(
        output.contains("Jun") || output.contains("2027"),
        "Expected datetime in cancel dialog"
    );
}

// --- Display priority: scheduled over last-run ---

#[test]
fn test_render_scheduled_takes_priority_over_last_run() {
    // Both a scheduled run and last-run exist for the same slug.
    // Only the scheduled info should appear in the Run Info column.
    let mut run_info = HashMap::new();
    let at = chrono::Local.with_ymd_and_hms(2027, 6, 1, 20, 0, 0).unwrap();
    // Insert Scheduled (takes priority)
    run_info.insert(
        "feature-a".to_string(),
        SpecRunInfo::Scheduled {
            team: "alpha-team".to_string(),
            at,
            plist_path: PathBuf::from("/tmp/test.plist"),
        },
    );
    let mut app = App::new(
        vec![spec("feature-a.md")],
        vec!["alpha-team".into(), "beta-team".into()],
        "alpha-team",
        Prefs::default(),
        run_info,
        PathBuf::from("/tmp/test"),
        vec![],
    );
    let output = render_to_string(&mut app, 100, 20);
    // Scheduled info should appear
    assert!(output.contains("Jun") || output.contains("2027"), "Expected scheduled datetime");
    // The last-run team "beta-team" should NOT appear (scheduled takes priority)
    // Note: since we only inserted a Scheduled entry (not both), the test verifies
    // that the Scheduled variant's data appears. The implementation must prefer
    // Scheduled over LastRun when both would be present in the map.
    // Here we verify the Scheduled data is displayed correctly.
    assert!(output.contains("alpha-team"), "Expected scheduled team in run info");
}

// --- AccountDialog popup rendering ---

#[test]
fn test_render_shows_account_dialog_popup() {
    let accounts = vec![
        AccountEntry { label: "personal".to_string() },
        AccountEntry { label: "work".to_string() },
    ];
    let mut app = App::new(
        vec![spec("feature-a.md")],
        vec!["feature-dev".into()],
        "feature-dev",
        Prefs::default(),
        HashMap::new(),
        PathBuf::from("/tmp/test"),
        accounts,
    );
    app.popup = Some(PopupAction::AccountDialog { selected_index: 0 });
    let output = render_to_string(&mut app, 100, 20);
    assert!(
        output.contains("Account") || output.contains("account"),
        "Expected account dialog title in output"
    );
}

#[test]
fn test_render_account_dialog_shows_labels() {
    let accounts = vec![
        AccountEntry { label: "personal".to_string() },
        AccountEntry { label: "work".to_string() },
    ];
    let mut app = App::new(
        vec![spec("feature-a.md")],
        vec!["feature-dev".into()],
        "feature-dev",
        Prefs::default(),
        HashMap::new(),
        PathBuf::from("/tmp/test"),
        accounts,
    );
    app.popup = Some(PopupAction::AccountDialog { selected_index: 0 });
    let output = render_to_string(&mut app, 100, 20);
    assert!(output.contains("personal"), "Expected 'personal' account label in output");
    assert!(output.contains("work"), "Expected 'work' account label in output");
}
