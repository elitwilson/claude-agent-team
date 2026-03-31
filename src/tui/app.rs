use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveTime, TimeZone};

use super::metrics::MetricsState;
use crate::config::{SpecEntry, SpecStatus};
use crate::prefs::Prefs;

/// Which screen is currently displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Launcher,
    Metrics,
    SchedulePicker,
}

/// The popup shown over the launcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupAction {
    TeamDialog { selected_index: usize },
    ActionDialog { selected: ActionChoice },
}

/// Choices available in the action popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionChoice {
    ExecuteNow,
    ScheduleLater,
}

/// AM/PM indicator for 12-hour time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmPm {
    Am,
    Pm,
}

/// Which field is focused in the schedule picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerField {
    Month,
    Day,
    Year,
    Hour,
    Minute,
    AmPm,
}

/// State for the schedule date/time picker.
#[derive(Debug, Clone)]
pub struct SchedulePickerState {
    pub month: u32,
    pub day: u32,
    pub year: i32,
    pub hour: u32,
    pub minute: u32,
    pub am_pm: AmPm,
    pub focused: PickerField,
    pub error: Option<String>,
}

impl Default for SchedulePickerState {
    fn default() -> Self {
        let now = Local::now();
        Self {
            month: now.month(),
            day: now.day(),
            year: now.year(),
            hour: 8,
            minute: 0,
            am_pm: AmPm::Pm,
            focused: PickerField::Month,
            error: None,
        }
    }
}

impl SchedulePickerState {
    /// Move focus to the next field (Tab).
    pub fn next_field(&mut self) {
        self.focused = match self.focused {
            PickerField::Month => PickerField::Day,
            PickerField::Day => PickerField::Year,
            PickerField::Year => PickerField::Hour,
            PickerField::Hour => PickerField::Minute,
            PickerField::Minute => PickerField::AmPm,
            PickerField::AmPm => PickerField::Month,
        };
    }

    /// Move focus to the previous field (Shift-Tab).
    pub fn prev_field(&mut self) {
        self.focused = match self.focused {
            PickerField::Month => PickerField::AmPm,
            PickerField::Day => PickerField::Month,
            PickerField::Year => PickerField::Day,
            PickerField::Hour => PickerField::Year,
            PickerField::Minute => PickerField::Hour,
            PickerField::AmPm => PickerField::Minute,
        };
    }

    /// Increment the focused field (Up arrow).
    pub fn increment(&mut self) {
        match self.focused {
            PickerField::Month => {
                self.month = if self.month == 12 { 1 } else { self.month + 1 };
                self.clamp_day();
            }
            PickerField::Day => {
                let max = days_in_month(self.month, self.year);
                self.day = if self.day >= max { 1 } else { self.day + 1 };
            }
            PickerField::Year => {
                let max_year = Local::now().year() + 5;
                if self.year < max_year {
                    self.year += 1;
                    self.clamp_day();
                }
            }
            PickerField::Hour => {
                self.hour = if self.hour == 12 { 1 } else { self.hour + 1 };
            }
            PickerField::Minute => {
                self.minute = if self.minute == 59 {
                    0
                } else {
                    self.minute + 1
                };
            }
            PickerField::AmPm => {
                self.am_pm = match self.am_pm {
                    AmPm::Am => AmPm::Pm,
                    AmPm::Pm => AmPm::Am,
                };
            }
        }
    }

    /// Decrement the focused field (Down arrow).
    pub fn decrement(&mut self) {
        match self.focused {
            PickerField::Month => {
                self.month = if self.month == 1 { 12 } else { self.month - 1 };
                self.clamp_day();
            }
            PickerField::Day => {
                let max = days_in_month(self.month, self.year);
                self.day = if self.day <= 1 { max } else { self.day - 1 };
            }
            PickerField::Year => {
                let min_year = Local::now().year();
                if self.year > min_year {
                    self.year -= 1;
                    self.clamp_day();
                }
            }
            PickerField::Hour => {
                self.hour = if self.hour == 1 { 12 } else { self.hour - 1 };
            }
            PickerField::Minute => {
                self.minute = if self.minute == 0 {
                    59
                } else {
                    self.minute - 1
                };
            }
            PickerField::AmPm => {
                self.am_pm = match self.am_pm {
                    AmPm::Am => AmPm::Pm,
                    AmPm::Pm => AmPm::Am,
                };
            }
        }
    }

    /// Convert the 12-hour fields to a `NaiveTime` in 24-hour format.
    pub fn to_naive_time(&self) -> NaiveTime {
        let hour_24 = match (self.am_pm, self.hour) {
            (AmPm::Am, 12) => 0,     // 12 AM = midnight
            (AmPm::Am, h) => h,      // 1-11 AM = 1-11
            (AmPm::Pm, 12) => 12,    // 12 PM = noon
            (AmPm::Pm, h) => h + 12, // 1-11 PM = 13-23
        };
        NaiveTime::from_hms_opt(hour_24, self.minute, 0).unwrap()
    }

    /// Validate and confirm the picker selection.
    /// Returns `Some(DateTime<Local>)` if the datetime is at least 1 minute in the future.
    /// Sets `self.error` and returns `None` otherwise.
    pub fn confirm(&mut self) -> Option<DateTime<Local>> {
        let date = NaiveDate::from_ymd_opt(self.year, self.month, self.day).unwrap();
        let time = self.to_naive_time();
        let naive_dt = date.and_time(time);
        let local_dt = match Local.from_local_datetime(&naive_dt).single() {
            Some(dt) => dt,
            None => {
                self.error = Some("Invalid date/time".to_string());
                return None;
            }
        };

        let now = Local::now();
        if local_dt < now + Duration::minutes(1) {
            self.error = Some("Scheduled time must be in the future".to_string());
            return None;
        }

        self.error = None;
        Some(local_dt)
    }

    /// Clamp day to valid range for the current month/year.
    fn clamp_day(&mut self) {
        let max = days_in_month(self.month, self.year);
        if self.day > max {
            self.day = max;
        }
    }
}

/// Returns the number of days in the given month and year.
fn days_in_month(month: u32, year: i32) -> u32 {
    if month == 12 {
        31
    } else {
        let next = NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap();
        next.pred_opt().unwrap().day()
    }
}

/// Which panel is currently focused in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Spec,
    Options,
}

/// Number of items in the Options panel.
pub const OPTIONS_ITEMS: usize = 3;

/// Which tab is active in the spec panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecTab {
    Specs,
    Requirements,
}

/// Whether the confirmed selection should run a team or draft a spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    TeamRun,
    DraftRun,
}

/// The result of the TUI session.
#[derive(Debug, Clone)]
pub struct TuiResult {
    pub spec: String,
    pub team: String,
    pub headless: bool,
    pub mode: RunMode,
    pub scheduled_at: Option<DateTime<Local>>,
}

/// TUI application state.
#[derive(Debug)]
pub struct App {
    pub specs: Vec<SpecEntry>,
    pub requirements: Vec<SpecEntry>,
    pub teams: Vec<String>,
    pub spec_index: usize,
    pub requirements_index: usize,
    pub team_index: usize,
    pub options_index: usize,
    pub focused_panel: Panel,
    pub prefs: Prefs,
    pub active_tab: SpecTab,
    pub should_quit: bool,
    pub confirmed: bool,
    pub screen: Screen,
    pub metrics_state: Option<MetricsState>,
    pub popup: Option<PopupAction>,
    pub picker: SchedulePickerState,
    pub scheduled_at: Option<DateTime<Local>>,
}

impl App {
    pub fn new(
        all_entries: Vec<SpecEntry>,
        teams: Vec<String>,
        default_team: &str,
        prefs: Prefs,
    ) -> Self {
        let team_index = teams.iter().position(|t| t == default_team).unwrap_or(0);
        let (requirements, specs): (Vec<SpecEntry>, Vec<SpecEntry>) = all_entries
            .into_iter()
            .partition(|e| e.status == SpecStatus::Raw);
        Self {
            specs,
            requirements,
            teams,
            spec_index: 0,
            requirements_index: 0,
            team_index,
            options_index: 0,
            focused_panel: Panel::Spec,
            prefs,
            active_tab: SpecTab::Specs,
            should_quit: false,
            confirmed: false,
            screen: Screen::Launcher,
            metrics_state: None,
            popup: None,
            picker: SchedulePickerState::default(),
            scheduled_at: None,
        }
    }

    /// Returns specs visible under current filter settings.
    pub fn visible_specs(&self) -> Vec<&SpecEntry> {
        self.specs
            .iter()
            .filter(|s| match s.status {
                SpecStatus::Complete => self.prefs.show_complete,
                SpecStatus::Blocked => self.prefs.show_blocked,
                _ => true,
            })
            .collect()
    }

    /// Cycle focus to the next panel (Tab).
    pub fn next_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            Panel::Spec => Panel::Options,
            Panel::Options => Panel::Spec,
        };
    }

    /// Toggle between Specs and Requirements tabs.
    pub fn switch_tab(&mut self) {
        self.active_tab = match self.active_tab {
            SpecTab::Specs => SpecTab::Requirements,
            SpecTab::Requirements => SpecTab::Specs,
        };
    }

    /// Move selection up within the focused panel or scroll metrics.
    pub fn move_up(&mut self) {
        if self.screen == Screen::Metrics {
            if let Some(ref mut state) = self.metrics_state {
                state.scroll_up();
            }
            return;
        }
        match self.focused_panel {
            Panel::Spec => match self.active_tab {
                SpecTab::Specs => {
                    self.spec_index = self.spec_index.saturating_sub(1);
                }
                SpecTab::Requirements => {
                    self.requirements_index = self.requirements_index.saturating_sub(1);
                }
            },
            Panel::Options => {
                self.options_index = self.options_index.saturating_sub(1);
            }
        }
    }

    /// Move selection down within the focused panel or scroll metrics.
    pub fn move_down(&mut self) {
        if self.screen == Screen::Metrics {
            if let Some(ref mut state) = self.metrics_state {
                state.scroll_down();
            }
            return;
        }
        match self.focused_panel {
            Panel::Spec => match self.active_tab {
                SpecTab::Specs => {
                    let visible = self.visible_specs().len();
                    if self.spec_index + 1 < visible {
                        self.spec_index += 1;
                    }
                }
                SpecTab::Requirements => {
                    if self.requirements_index + 1 < self.requirements.len() {
                        self.requirements_index += 1;
                    }
                }
            },
            Panel::Options => {
                if self.options_index + 1 < OPTIONS_ITEMS {
                    self.options_index += 1;
                }
            }
        }
    }

    /// Toggle the option at options_index and save prefs.
    pub fn toggle_option(&mut self) {
        match self.options_index {
            0 => self.prefs.headless = !self.prefs.headless,
            1 => {
                self.prefs.show_complete = !self.prefs.show_complete;
                self.clamp_spec_index();
            }
            2 => {
                self.prefs.show_blocked = !self.prefs.show_blocked;
                self.clamp_spec_index();
            }
            _ => {}
        }
        self.prefs.save();
    }

    /// Toggle headless pref and save.
    pub fn toggle_headless(&mut self) {
        self.prefs.headless = !self.prefs.headless;
        self.prefs.save();
    }

    /// Toggle show_complete pref, clamp spec index, and save.
    pub fn toggle_show_complete(&mut self) {
        self.prefs.show_complete = !self.prefs.show_complete;
        self.clamp_spec_index();
        self.prefs.save();
    }

    /// Toggle show_blocked pref, clamp spec index, and save.
    pub fn toggle_show_blocked(&mut self) {
        self.prefs.show_blocked = !self.prefs.show_blocked;
        self.clamp_spec_index();
        self.prefs.save();
    }

    /// Clamp spec_index to the visible specs list length after a filter change.
    fn clamp_spec_index(&mut self) {
        let len = self.visible_specs().len();
        if len == 0 {
            self.spec_index = 0;
        } else if self.spec_index >= len {
            self.spec_index = len - 1;
        }
    }

    /// Confirm Enter key on the launcher. Only acts when Spec panel is focused.
    pub fn confirm(&mut self) {
        if self.focused_panel != Panel::Spec {
            return;
        }
        match self.active_tab {
            SpecTab::Specs => self.open_team_popup(),
            SpecTab::Requirements => {
                if !self.requirements.is_empty() {
                    self.confirmed = true;
                }
            }
        }
    }

    /// Switch to the metrics screen with loaded data.
    pub fn open_metrics(&mut self, state: MetricsState) {
        self.screen = Screen::Metrics;
        self.metrics_state = Some(state);
    }

    /// Return to the launcher from the metrics screen.
    pub fn close_metrics(&mut self) {
        self.screen = Screen::Launcher;
    }

    /// Open the team selection popup for the currently highlighted spec.
    /// No-op if the spec list is empty or the selected spec is Blocked/Complete.
    pub fn open_team_popup(&mut self) {
        let visible = self.visible_specs();
        if visible.is_empty() {
            return;
        }
        let selected = visible[self.spec_index];
        if matches!(selected.status, SpecStatus::Blocked | SpecStatus::Complete) {
            return;
        }
        self.popup = Some(PopupAction::TeamDialog {
            selected_index: self.team_index,
        });
    }

    /// Dismiss the active popup.
    /// Esc on TeamDialog → spec list (popup = None).
    /// Esc on ActionDialog → restores TeamDialog.
    pub fn dismiss_popup(&mut self) {
        self.popup = match self.popup.take() {
            Some(PopupAction::ActionDialog { .. }) => Some(PopupAction::TeamDialog {
                selected_index: self.team_index,
            }),
            _ => None,
        };
    }

    /// Move selection down within the active popup.
    pub fn popup_move_down(&mut self) {
        match &mut self.popup {
            Some(PopupAction::TeamDialog { selected_index }) => {
                if *selected_index + 1 < self.teams.len() {
                    *selected_index += 1;
                }
            }
            Some(PopupAction::ActionDialog { selected }) => {
                if *selected == ActionChoice::ExecuteNow {
                    *selected = ActionChoice::ScheduleLater;
                }
            }
            None => {}
        }
    }

    /// Move selection up within the active popup.
    pub fn popup_move_up(&mut self) {
        match &mut self.popup {
            Some(PopupAction::TeamDialog { selected_index }) => {
                *selected_index = selected_index.saturating_sub(1);
            }
            Some(PopupAction::ActionDialog { selected }) => {
                if *selected == ActionChoice::ScheduleLater {
                    *selected = ActionChoice::ExecuteNow;
                }
            }
            None => {}
        }
    }

    /// Confirm the active popup selection.
    /// TeamDialog → stores team_index and opens ActionDialog.
    /// ActionDialog → ExecuteNow sets confirmed; ScheduleLater switches to SchedulePicker.
    pub fn confirm_popup(&mut self) {
        let action = match self.popup.take() {
            Some(a) => a,
            None => return,
        };
        match action {
            PopupAction::TeamDialog { selected_index } => {
                self.team_index = selected_index;
                self.popup = Some(PopupAction::ActionDialog {
                    selected: ActionChoice::ExecuteNow,
                });
            }
            PopupAction::ActionDialog { selected } => match selected {
                ActionChoice::ExecuteNow => {
                    self.confirmed = true;
                }
                ActionChoice::ScheduleLater => {
                    self.screen = Screen::SchedulePicker;
                }
            },
        }
    }

    /// Confirm the schedule picker: store the datetime and exit.
    pub fn confirm_picker(&mut self) {
        if let Some(dt) = self.picker.confirm() {
            self.scheduled_at = Some(dt);
            self.confirmed = true;
        }
    }

    /// Get the selected result, if confirmed.
    pub fn result(&self) -> Option<TuiResult> {
        if !self.confirmed {
            return None;
        }
        match self.active_tab {
            SpecTab::Specs => {
                let visible = self.visible_specs();
                Some(TuiResult {
                    spec: visible[self.spec_index].name.clone(),
                    team: self.teams[self.team_index].clone(),
                    headless: self.prefs.headless,
                    mode: RunMode::TeamRun,
                    scheduled_at: self.scheduled_at,
                })
            }
            SpecTab::Requirements => Some(TuiResult {
                spec: self.requirements[self.requirements_index].name.clone(),
                team: self.teams[self.team_index].clone(),
                headless: self.prefs.headless,
                mode: RunMode::DraftRun,
                scheduled_at: None,
            }),
        }
    }
}

#[cfg(test)]
mod tests;
