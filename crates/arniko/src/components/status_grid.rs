//! StatusGrid component for Arniko

use crate::{Component, ComponentMetadata};
use crate::components::escape_html;

#[derive(Clone, PartialEq, Default)]
pub enum StatusState {
    #[default]
    Active,
    Warning,
    Error,
    Idle,
    Offline,
}

pub struct StatusIndicator {
    label: String,
    value: String,
    state: StatusState,
    pulse: bool,
    class: String,
}

impl StatusIndicator {
    pub fn new(label: &str, value: &str) -> Self {
        Self {
            label: label.to_string(),
            value: value.to_string(),
            state: StatusState::Active,
            pulse: false,
            class: String::new(),
        }
    }

    pub fn state(mut self, state: StatusState) -> Self {
        self.state = state;
        self
    }

    pub fn pulse(mut self, pulse: bool) -> Self {
        self.pulse = pulse;
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        let state_class = match self.state {
            StatusState::Active => "arniko-status-active",
            StatusState::Warning => "arniko-status-warning",
            StatusState::Error => "arniko-status-error",
            StatusState::Idle => "arniko-status-idle",
            StatusState::Offline => "arniko-status-offline",
        };

        let pulse_class = if self.pulse {
            " arniko-status-pulse"
        } else {
            ""
        };

        format!(
            r#"<div class="arniko-status-item {}">
                <span class="arniko-status-label">{}</span>
                <div class="arniko-status-value">
                    <div class="arniko-status-dot {}{}"></div>
                    <span class="arniko-status-text">{}</span>
                </div>
            </div>"#,
            escape_html(&self.class), escape_html(&self.label), state_class, pulse_class, escape_html(&self.value)
        )
    }
}

impl Component for StatusIndicator {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::default()
    }
}

pub struct StatusGrid {
    items: Vec<StatusIndicator>,
    columns: u32,
    class: String,
}

impl StatusGrid {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            columns: 1,
            class: String::new(),
        }
    }

    pub fn add(mut self, item: StatusIndicator) -> Self {
        self.items.push(item);
        self
    }

    pub fn columns(mut self, columns: u32) -> Self {
        self.columns = columns;
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        let items_html = self
            .items
            .iter()
            .map(|item| item.render())
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"<div class="arniko-status-grid {}" style="grid-template-columns: repeat({}, 1fr);">
                {}
            </div>"#,
            escape_html(&self.class), self.columns, items_html
        )
    }
}

impl Component for StatusGrid {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_indicator_default() {
        let indicator = StatusIndicator::new("CPU", "45%");
        let html = indicator.render();
        assert!(html.contains("arniko-status-item"));
        assert!(html.contains("CPU"));
        assert!(html.contains("45%"));
        assert!(html.contains("arniko-status-active"));
    }

    #[test]
    fn test_status_indicator_states() {
        let warning = StatusIndicator::new("Disk", "80%").state(StatusState::Warning);
        assert!(warning.render().contains("arniko-status-warning"));

        let error = StatusIndicator::new("Net", "down").state(StatusState::Error);
        assert!(error.render().contains("arniko-status-error"));
    }

    #[test]
    fn test_status_grid_empty() {
        let grid = StatusGrid::new();
        let html = grid.render();
        assert!(html.contains("arniko-status-grid"));
    }

    #[test]
    fn test_status_grid_with_items() {
        let grid = StatusGrid::new()
            .add(StatusIndicator::new("CPU", "45%"))
            .add(StatusIndicator::new("RAM", "60%"));
        let html = grid.render();
        assert!(html.contains("CPU"));
        assert!(html.contains("RAM"));
    }
}
