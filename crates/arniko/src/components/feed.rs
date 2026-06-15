//! Feed — Activity feed / event log component.
//!
//! Generalized from the Khukuri AuditFeed. Renders a scrollable list of
//! timestamped events with severity levels and styled classes.

#[cfg(feature = "components")]
use crate::{Component, ComponentMetadata};

// ── Data Types ───────────────────────────────────────────────────────────────

/// A single entry in an activity feed.
#[derive(Clone, Debug)]
pub struct FeedEntry {
    pub timestamp: String,
    pub level: String,
    pub text: String,
    pub class: String,
}

impl FeedEntry {
    pub fn new(timestamp: &str, level: &str, text: &str, class: &str) -> Self {
        Self {
            timestamp: timestamp.to_string(),
            level: level.to_string(),
            text: text.to_string(),
            class: class.to_string(),
        }
    }
}

// ── HTML Component ───────────────────────────────────────────────────────────

/// A scrollable activity feed panel.
#[cfg(feature = "components")]
pub struct Feed {
    entries: Vec<FeedEntry>,
    title: String,
    title_icon: String,
    empty_message: String,
    class: String,
}

#[cfg(feature = "components")]
impl Feed {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            title: "Activity Feed".to_string(),
            title_icon: "📋".to_string(),
            empty_message: "No events yet.".to_string(),
            class: String::new(),
        }
    }

    pub fn add(mut self, entry: FeedEntry) -> Self {
        self.entries.push(entry);
        self
    }

    pub fn add_entries(mut self, entries: &[FeedEntry]) -> Self {
        self.entries.extend(entries.iter().cloned());
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn title_icon(mut self, icon: &str) -> Self {
        self.title_icon = icon.to_string();
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
        let mut items_html = String::new();
        for entry in &self.entries {
            items_html.push_str(&format!(
                r#"
                <div class="arniko-feed-item {}">
                    <span class="arniko-feed-timestamp">[{}]</span>
                    <span class="arniko-feed-level {}">{}</span>
                    <span class="arniko-feed-text">{}</span>
                </div>
                "#,
                entry.class, entry.timestamp, entry.class, entry.level, entry.text,
            ));
        }

        if items_html.is_empty() {
            items_html = format!(
                r#"<div class="arniko-feed-empty">{}</div>"#,
                self.empty_message
            );
        }

        format!(
            r##"
            <div class="arniko-feed {}">
                <div class="arniko-feed-header">
                    <span class="arniko-feed-header-icon">{}</span>
                    <span class="arniko-feed-header-title">{}</span>
                </div>
                <div class="arniko-feed-body">
                    {}
                </div>
            </div>
            "##,
            self.class, self.title_icon, self.title, items_html
        )
    }
}

#[cfg(feature = "components")]
impl Default for Feed {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "components")]
impl Component for Feed {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec!["arniko-feed".to_string()],
            requires_gpu: false,
            capabilities: vec![],
        }
    }
}

// ── Reactive View ────────────────────────────────────────────────────────────

#[cfg(feature = "reactive")]
use crate::reactive::{ReactiveHtml, Signal, View};

/// Create a reactive feed that updates when the entries signal changes.
#[cfg(feature = "reactive")]
pub fn feed_reactive(entries_signal: Signal<Vec<FeedEntry>>) -> Box<dyn View> {
    let html = entries_signal.derive(move |entries| {
        Feed {
            entries,
            ..Feed::default()
        }
        .render()
    });
    Box::new(ReactiveHtml::new(html))
}

/// Create a reactive feed with a builder for full configuration.
#[cfg(feature = "reactive")]
pub fn feed_reactive_with<F>(
    entries_signal: Signal<Vec<FeedEntry>>,
    build: F,
) -> Box<dyn View>
where
    F: Fn(&mut Feed) + Send + Sync + 'static,
{
    let html = entries_signal.derive(move |entries| {
        let mut feed = Feed::new();
        build(&mut feed);
        feed.entries = entries;
        feed.render()
    });
    Box::new(ReactiveHtml::new(html))
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "components")]
    #[test]
    fn test_feed_empty() {
        let feed = Feed::new();
        let html = feed.render();
        assert!(html.contains("arniko-feed"));
        assert!(html.contains("No events yet"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_feed_with_entries() {
        let feed = Feed::new()
            .add(FeedEntry::new("14:23", "INFO", "Scan started", "info"))
            .add(FeedEntry::new("14:24", "CRITICAL", "Vuln found", "critical"));
        let html = feed.render();
        assert!(html.contains("Scan started"));
        assert!(html.contains("Vuln found"));
        assert!(html.contains("arniko-feed-item info"));
        assert!(html.contains("arniko-feed-item critical"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_feed_custom_title() {
        let feed = Feed::new()
            .title("Security Feed")
            .title_icon("🛡️")
            .add(FeedEntry::new("12:00", "WARN", "Alert", "warning"));
        let html = feed.render();
        assert!(html.contains("Security Feed"));
        assert!(html.contains("🛡️"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_feed_custom_empty_message() {
        let feed = Feed::new().empty_message("Awaiting events...");
        let html = feed.render();
        assert!(html.contains("Awaiting events..."));
    }
}
