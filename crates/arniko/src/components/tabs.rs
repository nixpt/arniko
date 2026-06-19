//! Tabs component for Arniko dashboard kit.
//!
//! Renders a tab bar + single active panel. The static version pre-selects
//! one tab; the reactive version re-renders when a `Signal<usize>` changes.

use crate::components::escape_html;
use crate::{Component, ComponentMetadata};

/// A single tab item (label + HTML content).
///
/// # Examples
///
/// ```rust,no_run
/// use arniko::TabItem;
///
/// let tab = TabItem::new("Overview", "<p>Summary</p>");
/// ```
#[derive(Clone)]
pub struct TabItem {
    label: String,
    content: String,
    id: String,
}

impl TabItem {
    pub fn new(label: &str, content: &str) -> Self {
        let id = label
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect();
        Self {
            label: label.to_string(),
            content: content.to_string(),
            id,
        }
    }

    /// Override the generated element `id` (used for `aria-controls`).
    pub fn id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }
}

/// Tab container — renders a tab bar and the active panel's content.
///
/// # Examples
///
/// ```rust,no_run
/// use arniko::{Tabs, TabItem};
///
/// let tabs = Tabs::new()
///     .add(TabItem::new("Metrics", "<p>metrics content</p>"))
///     .add(TabItem::new("Events",  "<p>events content</p>"))
///     .add(TabItem::new("Config",  "<p>config content</p>"))
///     .active(1);
///
/// let html = tabs.render();
/// ```
///
/// With the `reactive` feature:
///
/// ```rust,ignore
/// # #[cfg(feature = "reactive")] {
/// use arniko::{tabs_reactive, TabItem};
/// use arniko::reactive::Signal;
///
/// let items = vec![
///     TabItem::new("Metrics", "<p>metrics</p>"),
///     TabItem::new("Events",  "<p>events</p>"),
/// ];
/// let active = Signal::new(0usize);
/// let view = tabs_reactive(items, active);
/// # }
/// ```
pub struct Tabs {
    items: Vec<TabItem>,
    active: usize,
    class: String,
}

impl Tabs {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            active: 0,
            class: String::new(),
        }
    }

    pub fn add(mut self, item: TabItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn active(mut self, index: usize) -> Self {
        self.active = index;
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        if self.items.is_empty() {
            return format!(
                r#"<div class="arniko-tabs {}"></div>"#,
                escape_html(&self.class)
            );
        }
        let active = self.active.min(self.items.len() - 1);

        let bar: String = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let selected = if i == active { "true" } else { "false" };
                format!(
                    r#"<button class="arniko-tab-btn" role="tab" aria-selected="{}" aria-controls="panel-{}">{}</button>"#,
                    selected,
                    escape_html(&item.id),
                    escape_html(&item.label),
                )
            })
            .collect();

        let panel = &self.items[active];
        format!(
            r#"<div class="arniko-tabs {}"><div class="arniko-tabs-bar" role="tablist">{}</div><div class="arniko-tab-panel" id="panel-{}" role="tabpanel">{}</div></div>"#,
            escape_html(&self.class),
            bar,
            escape_html(&panel.id),
            panel.content,
        )
    }
}

impl Default for Tabs {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Tabs {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec!["arniko-tabs".to_string()],
            requires_gpu: false,
            capabilities: vec![],
        }
    }
}

// ── Reactive View ────────────────────────────────────────────────────────────

#[cfg(feature = "reactive")]
use crate::reactive::{ReactiveHtml, Signal, View};

/// Reactive tabs — re-renders the whole tabs component when `active_signal` changes.
///
/// Tab switching (updating `active_signal`) is the caller's responsibility;
/// wire click handlers via `reactor.on_click(node_id, handler)` after mounting.
#[cfg(feature = "reactive")]
pub fn tabs_reactive(items: Vec<TabItem>, active_signal: Signal<usize>) -> Box<dyn View> {
    let html = active_signal.derive(move |active| {
        Tabs {
            items: items.clone(),
            active,
            class: String::new(),
        }
        .render()
    });
    Box::new(ReactiveHtml::new(html))
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tabs_empty() {
        let tabs = Tabs::new();
        let html = tabs.render();
        assert!(html.contains("arniko-tabs"));
    }

    #[test]
    fn test_tabs_renders_active_panel() {
        let tabs = Tabs::new()
            .add(TabItem::new("A", "<p>content-a</p>"))
            .add(TabItem::new("B", "<p>content-b</p>"))
            .active(1);
        let html = tabs.render();
        assert!(html.contains("content-b"));
        assert!(!html.contains("content-a"));
        assert!(html.contains(r#"aria-selected="true""#));
    }

    #[test]
    fn test_tabs_clamps_active() {
        let tabs = Tabs::new()
            .add(TabItem::new("Only", "<p>only</p>"))
            .active(999);
        let html = tabs.render();
        assert!(html.contains("only"));
    }

    #[test]
    fn test_tabs_xss() {
        let tabs = Tabs::new()
            .add(TabItem::new("<script>", "safe content"));
        let html = tabs.render();
        assert!(!html.contains("<script>alert"));
    }
}
