use super::metrics::MetricsState;

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

/// The result of the TUI session.
#[derive(Debug, Clone)]
pub struct TuiResult {
    pub spec: String,
    pub team: String,
    pub headless: bool,
}

/// TUI application state.
#[derive(Debug)]
pub struct App {
    pub specs: Vec<String>,
    pub teams: Vec<String>,
    pub spec_index: usize,
    pub team_index: usize,
    pub headless: bool,
    pub focused_panel: Panel,
    pub should_quit: bool,
    pub confirmed: bool,
    pub screen: Screen,
    pub metrics_state: Option<MetricsState>,
}

impl App {
    pub fn new(specs: Vec<String>, teams: Vec<String>, default_team: &str) -> Self {
        let team_index = teams.iter().position(|t| t == default_team).unwrap_or(0);
        Self {
            specs,
            teams,
            spec_index: 0,
            team_index,
            headless: false,
            focused_panel: Panel::Spec,
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

    /// Move selection up within the current panel.
    pub fn move_up(&mut self) {
        match self.focused_panel {
            Panel::Spec => {
                self.spec_index = self.spec_index.saturating_sub(1);
            }
            Panel::Team => {
                self.team_index = self.team_index.saturating_sub(1);
            }
            Panel::RunOptions => {}
        }
    }

    /// Move selection down within the current panel.
    pub fn move_down(&mut self) {
        match self.focused_panel {
            Panel::Spec => {
                if self.spec_index + 1 < self.specs.len() {
                    self.spec_index += 1;
                }
            }
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

    /// Confirm and exit the TUI (Enter).
    pub fn confirm(&mut self) {
        self.confirmed = true;
    }

    /// Get the selected result, if confirmed.
    pub fn result(&self) -> Option<TuiResult> {
        if !self.confirmed {
            return None;
        }
        Some(TuiResult {
            spec: self.specs[self.spec_index].clone(),
            team: self.teams[self.team_index].clone(),
            headless: self.headless,
        })
    }
}

#[cfg(test)]
mod tests;
