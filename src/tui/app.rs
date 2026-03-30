use chrono::{DateTime, Local};

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

/// The action popup shown when pressing Enter on a Ready spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupAction {
    ActionDialog { selected: ActionChoice },
}

/// Choices available in the action popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionChoice {
    ExecuteNow,
    ScheduleLater,
}

/// Which panel is currently focused in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Spec,
    Team,
    Options,
}

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

/// Labels for each item in the Options panel, in order.
pub const OPTIONS_ITEMS: usize = 3;

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
    pub prefs: Prefs,
    pub focused_panel: Panel,
    pub active_tab: SpecTab,
    pub should_quit: bool,
    pub confirmed: bool,
    pub screen: Screen,
    pub metrics_state: Option<MetricsState>,
    pub popup: Option<PopupAction>,
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
            prefs,
            focused_panel: Panel::Spec,
            active_tab: SpecTab::Specs,
            should_quit: false,
            confirmed: false,
            screen: Screen::Launcher,
            metrics_state: None,
            popup: None,
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

    /// Move focus to the next panel (Tab).
    pub fn next_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            Panel::Spec => Panel::Team,
            Panel::Team => Panel::Options,
            Panel::Options => Panel::Spec,
        };
    }

    /// Toggle between Specs and Requirements tabs when the Spec panel is focused.
    pub fn switch_tab(&mut self) {
        self.active_tab = match self.active_tab {
            SpecTab::Specs => SpecTab::Requirements,
            SpecTab::Requirements => SpecTab::Specs,
        };
    }

    /// Move selection up within the current panel or scroll metrics.
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
            Panel::Team => {
                self.team_index = self.team_index.saturating_sub(1);
            }
            Panel::Options => {
                self.options_index = self.options_index.saturating_sub(1);
            }
        }
    }

    /// Move selection down within the current panel or scroll metrics.
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
            Panel::Team => {
                if self.team_index + 1 < self.teams.len() {
                    self.team_index += 1;
                }
            }
            Panel::Options => {
                if self.options_index + 1 < OPTIONS_ITEMS {
                    self.options_index += 1;
                }
            }
        }
    }

    /// Toggle the option at the current options_index and save prefs.
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

    /// Toggle headless (kept for backwards compat with key handler).
    pub fn toggle_headless(&mut self) {
        self.options_index = 0;
        self.toggle_option();
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

    /// Confirm and exit the TUI (Enter). No-op if the active list is empty or selected spec
    /// is Complete or Blocked.
    pub fn confirm(&mut self) {
        match self.active_tab {
            SpecTab::Specs => {
                let visible = self.visible_specs();
                if visible.is_empty() {
                    return;
                }
                let selected = visible[self.spec_index];
                if matches!(selected.status, SpecStatus::Blocked | SpecStatus::Complete) {
                    return;
                }
                self.confirmed = true;
            }
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

    /// Open the action popup for the currently selected spec.
    /// No-op if on the Requirements tab.
    pub fn open_action_popup(&mut self) {
        if self.active_tab == SpecTab::Requirements {
            return;
        }
        let visible = self.visible_specs();
        if visible.is_empty() {
            return;
        }
        let selected = visible[self.spec_index];
        if matches!(selected.status, SpecStatus::Blocked | SpecStatus::Complete) {
            return;
        }
        self.popup = Some(PopupAction::ActionDialog {
            selected: ActionChoice::ExecuteNow,
        });
    }

    /// Dismiss the popup without taking action.
    pub fn dismiss_popup(&mut self) {
        self.popup = None;
    }

    /// Move selection down within the popup.
    pub fn popup_move_down(&mut self) {
        if let Some(PopupAction::ActionDialog { ref mut selected }) = self.popup {
            if *selected == ActionChoice::ExecuteNow {
                *selected = ActionChoice::ScheduleLater;
            }
        }
    }

    /// Move selection up within the popup.
    pub fn popup_move_up(&mut self) {
        if let Some(PopupAction::ActionDialog { ref mut selected }) = self.popup {
            if *selected == ActionChoice::ScheduleLater {
                *selected = ActionChoice::ExecuteNow;
            }
        }
    }

    /// Confirm the popup selection.
    pub fn confirm_popup(&mut self) {
        let action = match self.popup.take() {
            Some(a) => a,
            None => return,
        };
        match action {
            PopupAction::ActionDialog { selected } => match selected {
                ActionChoice::ExecuteNow => {
                    self.confirm();
                }
                ActionChoice::ScheduleLater => {
                    self.screen = Screen::SchedulePicker;
                }
            },
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
                    scheduled_at: None,
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
