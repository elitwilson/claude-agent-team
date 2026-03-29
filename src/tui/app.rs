use super::metrics::MetricsState;
use crate::config::{SpecEntry, SpecStatus};

/// Which screen is currently displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Launcher,
    Metrics,
}

/// Which panel is currently focused in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Spec,
    Team,
    RunOptions,
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
    pub headless: bool,
    pub focused_panel: Panel,
    pub active_tab: SpecTab,
    pub should_quit: bool,
    pub confirmed: bool,
    pub screen: Screen,
    pub metrics_state: Option<MetricsState>,
}

impl App {
    pub fn new(all_entries: Vec<SpecEntry>, teams: Vec<String>, default_team: &str) -> Self {
        let team_index = teams.iter().position(|t| t == default_team).unwrap_or(0);
        let (requirements, specs): (Vec<SpecEntry>, Vec<SpecEntry>) =
            all_entries.into_iter().partition(|e| e.status == SpecStatus::Raw);
        Self {
            specs,
            requirements,
            teams,
            spec_index: 0,
            requirements_index: 0,
            team_index,
            headless: false,
            focused_panel: Panel::Spec,
            active_tab: SpecTab::Specs,
            should_quit: false,
            confirmed: false,
            screen: Screen::Launcher,
            metrics_state: None,
        }
    }

    /// Move focus to the next panel (Tab).
    pub fn next_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            Panel::Spec => Panel::Team,
            Panel::Team => Panel::RunOptions,
            Panel::RunOptions => Panel::Spec,
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
            Panel::RunOptions => {}
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
                    if self.spec_index + 1 < self.specs.len() {
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
            Panel::RunOptions => {}
        }
    }

    /// Toggle the headless option (Space).
    pub fn toggle_headless(&mut self) {
        self.headless = !self.headless;
    }

    /// Confirm and exit the TUI (Enter). No-op if the active list is empty or selected spec is blocked.
    pub fn confirm(&mut self) {
        match self.active_tab {
            SpecTab::Specs => {
                if self.specs.is_empty() {
                    return;
                }
                if self.specs[self.spec_index].status == SpecStatus::Blocked {
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

    /// Get the selected result, if confirmed.
    pub fn result(&self) -> Option<TuiResult> {
        if !self.confirmed {
            return None;
        }
        match self.active_tab {
            SpecTab::Specs => Some(TuiResult {
                spec: self.specs[self.spec_index].name.clone(),
                team: self.teams[self.team_index].clone(),
                headless: self.headless,
                mode: RunMode::TeamRun,
            }),
            SpecTab::Requirements => Some(TuiResult {
                spec: self.requirements[self.requirements_index].name.clone(),
                team: self.teams[self.team_index].clone(),
                headless: self.headless,
                mode: RunMode::DraftRun,
            }),
        }
    }
}

#[cfg(test)]
mod tests;
