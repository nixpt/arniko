//! EmptyState component for Arniko
//!
//! A placeholder displayed when no content is available.
//! Maps to capsule-ui's EmptyState component.

use crate::components::escape_html;
use crate::{Component, ComponentMetadata};

/// # Examples
///
/// ```rust,no_run
/// use arniko::EmptyState;
///
/// let empty = EmptyState::new("No items found")
///     .icon("📦")
///     .description("Add your first item.")
///     .action("<button>Add Item</button>");
///
/// let html = empty.render();
/// ```
pub struct EmptyState {
    icon: Option<String>,
    title: String,
    description: Option<String>,
    action: Option<String>,
    class: String,
}

impl EmptyState {
    pub fn new(title: &str) -> Self {
        Self {
            icon: None,
            title: title.to_string(),
            description: None,
            action: None,
            class: String::new(),
        }
    }

    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    pub fn description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    pub fn action(mut self, action_html: &str) -> Self {
        self.action = Some(action_html.to_string());
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        let icon_html = self
            .icon
            .as_ref()
            .map(|i| format!(r#"<div class="arniko-empty-icon">{}</div>"#, escape_html(i)))
            .unwrap_or_default();

        let desc_html = self
            .description
            .as_ref()
            .map(|d| format!(r#"<p class="arniko-empty-desc">{}</p>"#, escape_html(d)))
            .unwrap_or_default();

        let action_html = self
            .action
            .as_ref()
            .map(|a| format!(r#"<div class="arniko-empty-action">{}</div>"#, a))
            .unwrap_or_default();

        format!(
            r#"<div class="arniko-empty-state {}" role="status">{}<h3 class="arniko-empty-title">{}</h3>{}{}</div>"#,
            escape_html(&self.class),
            icon_html,
            escape_html(&self.title),
            desc_html,
            action_html
        )
    }
}

impl Component for EmptyState {
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
    fn test_empty_state_title_only() {
        let es = EmptyState::new("No items found");
        let html = es.render();
        assert!(html.contains("arniko-empty-state"));
        assert!(html.contains("No items found"));
        assert!(html.contains(r#"role="status""#));
    }

    #[test]
    fn test_empty_state_full() {
        let es = EmptyState::new("Empty")
            .icon("📦")
            .description("Nothing here yet")
            .action("<button>Create</button>");
        let html = es.render();
        assert!(html.contains("📦"));
        assert!(html.contains("Nothing here yet"));
        assert!(html.contains("<button>Create</button>"));
    }
}
