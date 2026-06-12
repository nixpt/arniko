//! Placeholder components for Arniko
//!
//! These will be implemented in subsequent phases.

use crate::{Component, ComponentMetadata};

// MetricCard placeholder
pub struct MetricCard {
    title: String,
    value: String,
    // TODO: Add full implementation
}

impl MetricCard {
    pub fn new(title: &str, value: &str) -> Self {
        Self {
            title: title.to_string(),
            value: value.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!(
            r#"<div class="arniko-metric-card">
            <div class="arniko-metric-title">{}</div>
            <div class="arniko-metric-value">{}</div>
        </div>"#,
            self.title, self.value
        )
    }
}

impl Component for MetricCard {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::default()
    }
}

// ProgressBar placeholder
pub struct ProgressBar {
    value: f32,
    max: f32,
    // TODO: Add full implementation
}

impl ProgressBar {
    pub fn new(value: f32) -> Self {
        Self { value, max: 100.0 }
    }

    pub fn render(&self) -> String {
        let percentage = (self.value / self.max * 100.0).min(100.0);
        format!(
            r#"<div class="arniko-progress">
            <div class="arniko-progress-bar" style="width: {}%"></div>
        </div>"#,
            percentage
        )
    }
}

impl Component for ProgressBar {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::default()
    }
}

// StatusGrid placeholder
pub struct StatusGrid {
    items: Vec<String>,
    // TODO: Add full implementation
}

impl StatusGrid {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn render(&self) -> String {
        format!(
            r#"<div class="arniko-status-grid">
            <div class="arniko-status-item">Status Grid (TODO)</div>
        </div>"#
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

// Tooltip placeholder
pub struct Tooltip {
    text: String,
    tooltip: String,
    // TODO: Add full implementation
}

impl Tooltip {
    pub fn new(text: &str, tooltip: &str) -> Self {
        Self {
            text: text.to_string(),
            tooltip: tooltip.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!(
            r#"<span class="arniko-tooltip" data-tooltip="{}">{}</span>"#,
            self.tooltip, self.text
        )
    }
}

impl Component for Tooltip {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::default()
    }
}

// Spinner placeholder
pub struct Spinner;

impl Spinner {
    pub fn render() -> String {
        r#"<div class="arniko-spinner"></div>"#.to_string()
    }
}

impl Component for Spinner {
    fn render(&self) -> String {
        Self::render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::default()
    }
}

// Skeleton placeholder
pub struct Skeleton {
    width: String,
    height: String,
    // TODO: Add full implementation
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            width: "100%".to_string(),
            height: "20px".to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!(
            r#"<div class="arniko-skeleton" style="width: {}; height: {};"></div>"#,
            self.width, self.height
        )
    }
}

impl Component for Skeleton {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::default()
    }
}

// Separator placeholder
pub struct Separator;

impl Separator {
    pub fn render() -> String {
        r#"<hr class="arniko-separator" />"#.to_string()
    }
}

impl Component for Separator {
    fn render(&self) -> String {
        Self::render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::default()
    }
}

// Kbd placeholder
pub struct Kbd {
    key: String,
    // TODO: Add full implementation
}

impl Kbd {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!(r#"<kbd class="arniko-kbd">{}</kbd>"#, self.key)
    }
}

impl Component for Kbd {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::default()
    }
}
