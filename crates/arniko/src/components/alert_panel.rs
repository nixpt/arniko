//! AlertPanel — Multi-alert banner panel component.
//!
//! Generalized from the Khukuri AlertSystem. Renders a reactive list of alert
//! banners with severity levels, timestamps, and an "all clear" state.
//! Complements the simpler single-banner `Alert` component.

#[cfg(feature = "components")]
use crate::components::escape_html;
#[cfg(feature = "components")]
use crate::{Component, ComponentMetadata};

// ── Data Types ───────────────────────────────────────────────────────────────

/// Severity level for an alert entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AlertLevel {
    Critical,
    Warning,
    Info,
}

impl AlertLevel {
    pub fn class(&self) -> &'static str {
        match self {
            AlertLevel::Critical => "arniko-alertpanel-critical",
            AlertLevel::Warning => "arniko-alertpanel-warning",
            AlertLevel::Info => "arniko-alertpanel-info",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            AlertLevel::Critical => "🔴",
            AlertLevel::Warning => "🟡",
            AlertLevel::Info => "🔵",
        }
    }
}

/// A single alert entry in the panel.
#[derive(Clone, Debug)]
pub struct AlertEntry {
    pub id: String,
    pub level: AlertLevel,
    pub title: String,
    pub message: String,
    pub timestamp: String,
}

impl AlertEntry {
    pub fn new(id: &str, level: AlertLevel, title: &str, message: &str, timestamp: &str) -> Self {
        Self {
            id: id.to_string(),
            level,
            title: title.to_string(),
            message: message.to_string(),
            timestamp: timestamp.to_string(),
        }
    }
}

// ── HTML Component ───────────────────────────────────────────────────────────

/// A panel rendering a list of alert banners.
///
/// When no alerts are present, renders an "all clear" status.
/// Otherwise renders a header with count, then individual alert items
/// with icon, title, message, and timestamp.
#[cfg(feature = "components")]
pub struct AlertPanel {
    entries: Vec<AlertEntry>,
    title: Option<String>,
    title_icon: Option<String>,
    empty_message: String,
    class: String,
}

#[cfg(feature = "components")]
impl AlertPanel {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            title: Some("Active Alerts".to_string()),
            title_icon: Some("🚨".to_string()),
            empty_message: "All systems secure. No active alerts.".to_string(),
            class: String::new(),
        }
    }

    pub fn add(mut self, entry: AlertEntry) -> Self {
        self.entries.push(entry);
        self
    }

    pub fn add_entries(mut self, entries: &[AlertEntry]) -> Self {
        self.entries.extend(entries.iter().cloned());
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn title_icon(mut self, icon: &str) -> Self {
        self.title_icon = Some(icon.to_string());
        self
    }

    pub fn empty_message(mut self, msg: &str) -> Self {
        self.empty_message = msg.to_string();
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        if self.entries.is_empty() {
            return self.render_empty();
        }

        let header = self.render_header();

        let rows: String = self
            .entries
            .iter()
            .map(|e| {
                format!(
                    r#"<div class="arniko-alertpanel-item {}">
                    <span class="arniko-alertpanel-icon">{}</span>
                    <div class="arniko-alertpanel-content">
                        <span class="arniko-alertpanel-title">{}</span>
                        <span class="arniko-alertpanel-message">{}</span>
                    </div>
                    <span class="arniko-alertpanel-time">{}</span>
                </div>"#,
                    e.level.class(),
                    e.level.icon(),
                    escape_html(&e.title),
                    escape_html(&e.message),
                    escape_html(&e.timestamp),
                )
            })
            .collect();

        format!(
            r#"<div class="arniko-alertpanel {class}">
    {header}
    <div class="arniko-alertpanel-list">
        {rows}
    </div>
</div>"#,
            class = escape_html(&self.class),
            header = header,
            rows = rows,
        )
    }

    fn render_header(&self) -> String {
        let count = self.entries.len();
        match (&self.title, &self.title_icon) {
            (Some(title), Some(icon)) => {
                format!(
                    r#"<div class="arniko-alertpanel-header">
                    <span class="arniko-alertpanel-header-icon">{}</span>
                    <span class="arniko-alertpanel-header-title">{} ({})</span>
                </div>"#,
                    icon,
                    escape_html(title),
                    count
                )
            }
            (Some(title), None) => {
                format!(
                    r#"<div class="arniko-alertpanel-header">
                    <span class="arniko-alertpanel-header-title">{} ({})</span>
                </div>"#,
                    escape_html(title),
                    count
                )
            }
            _ => String::new(),
        }
    }

    fn render_empty(&self) -> String {
        format!(
            r#"<div class="arniko-alertpanel arniko-alertpanel-allclear {class}">
    <span class="arniko-alertpanel-allclear-icon">🛡️</span>
    <span class="arniko-alertpanel-allclear-text">{}</span>
</div>"#,
            escape_html(&self.empty_message),
            class = escape_html(&self.class),
        )
    }
}

#[cfg(feature = "components")]
impl Default for AlertPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "components")]
impl Component for AlertPanel {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec!["arniko-alertpanel".to_string()],
            requires_gpu: false,
            capabilities: vec![],
        }
    }
}

// ── Reactive View ────────────────────────────────────────────────────────────

#[cfg(feature = "reactive")]
use crate::reactive::{ReactiveHtml, Signal, View};

/// Create a reactive alert panel that updates when the entries signal changes.
#[cfg(feature = "reactive")]
pub fn alert_panel_reactive(entries_signal: Signal<Vec<AlertEntry>>) -> Box<dyn View> {
    let html = entries_signal.derive(move |entries| {
        AlertPanel {
            entries,
            ..AlertPanel::default()
        }
        .render()
    });
    Box::new(ReactiveHtml::new(html))
}

/// Create a reactive alert panel with a builder for full configuration.
#[cfg(feature = "reactive")]
pub fn alert_panel_reactive_with<F>(
    entries_signal: Signal<Vec<AlertEntry>>,
    build: F,
) -> Box<dyn View>
where
    F: Fn(&mut AlertPanel) + Send + Sync + 'static,
{
    let html = entries_signal.derive(move |entries| {
        let mut panel = AlertPanel::new();
        build(&mut panel);
        panel.entries = entries;
        panel.render()
    });
    Box::new(ReactiveHtml::new(html))
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "components")]
    #[test]
    fn test_alert_panel_empty() {
        let panel = AlertPanel::new();
        let html = panel.render();
        assert!(html.contains("arniko-alertpanel-allclear"));
        assert!(html.contains("All systems secure"));
        assert!(html.contains("🛡️"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_alert_panel_with_entries() {
        let panel = AlertPanel::new()
            .add(AlertEntry::new(
                "a1",
                AlertLevel::Critical,
                "Critical Issue",
                "Something broke",
                "12:00",
            ))
            .add(AlertEntry::new(
                "a2",
                AlertLevel::Warning,
                "Warning",
                "Be careful",
                "12:01",
            ));
        let html = panel.render();
        assert!(html.contains("Active Alerts (2)"));
        assert!(html.contains("🚨"));
        assert!(html.contains("Critical Issue"));
        assert!(html.contains("Warning"));
        assert!(html.contains("arniko-alertpanel-critical"));
        assert!(html.contains("arniko-alertpanel-warning"));
        assert!(html.contains("🔴"));
        assert!(html.contains("🟡"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_alert_panel_custom_title() {
        let panel = AlertPanel::new()
            .title("Security Alerts")
            .title_icon("⚠️")
            .add(AlertEntry::new(
                "a1",
                AlertLevel::Info,
                "Info",
                "FYI",
                "12:00",
            ));
        let html = panel.render();
        assert!(html.contains("Security Alerts"));
        assert!(html.contains("⚠️"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_alert_panel_custom_empty_message() {
        let panel = AlertPanel::new().empty_message("Everything is fine!");
        let html = panel.render();
        assert!(html.contains("Everything is fine!"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_alert_panel_all_levels() {
        let panel = AlertPanel::new()
            .add(AlertEntry::new("a1", AlertLevel::Critical, "C", "msg", "T"))
            .add(AlertEntry::new("a2", AlertLevel::Warning, "W", "msg", "T"))
            .add(AlertEntry::new("a3", AlertLevel::Info, "I", "msg", "T"));
        let html = panel.render();
        assert!(html.contains("arniko-alertpanel-critical"));
        assert!(html.contains("arniko-alertpanel-warning"));
        assert!(html.contains("arniko-alertpanel-info"));
    }
}
