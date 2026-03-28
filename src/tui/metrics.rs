use crate::metrics::query::RunSummary;

/// State for the metrics screen.
#[derive(Debug)]
pub struct MetricsState {
    pub runs: Vec<RunSummary>,
    pub scroll_offset: usize,
}

impl MetricsState {
    pub fn new(runs: Vec<RunSummary>) -> Self {
        Self {
            runs,
            scroll_offset: 0,
        }
    }

    /// Scroll up by one row.
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scroll down by one row (clamped to data length).
    pub fn scroll_down(&mut self) {
        if !self.runs.is_empty() && self.scroll_offset + 1 < self.runs.len() {
            self.scroll_offset += 1;
        }
    }
}

#[cfg(test)]
mod tests;
