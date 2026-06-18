//! Card component for Arniko
//!
//! Provides card components with title and body content.

use crate::components::escape_html;
use crate::{Component, ComponentMetadata};

/// # Examples
///
/// ```rust,no_run
/// use arniko::Card;
///
/// let card = Card::new()
///     .title("System Status")
///     .body("All systems operational")
///     .class("dashboard-card");
/// ```
pub struct Card {
    title: Option<String>,
    body: Option<String>,
    class: String,
}

impl Card {
    pub fn new() -> Self {
        Self {
            title: None,
            body: None,
            class: String::new(),
        }
    }

    pub fn title(mut self, t: &str) -> Self {
        self.title = Some(t.to_string());
        self
    }

    pub fn body(mut self, b: &str) -> Self {
        self.body = Some(b.to_string());
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        let title_html = self
            .title
            .as_ref()
            .map(|t| format!(r#"<div class="arniko-card-title">{}</div>"#, escape_html(t)))
            .unwrap_or_default();

        let body_html = self
            .body
            .as_ref()
            .map(|b| format!(r#"<div class="arniko-card-body">{}</div>"#, escape_html(b)))
            .unwrap_or_default();

        format!(
            r#"<div class="arniko-card {}" role="region" aria-label="{}">{}{}</div>"#,
            escape_html(&self.class),
            escape_html(self.title.as_deref().unwrap_or("Card")),
            title_html,
            body_html
        )
    }
}

impl Component for Card {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec!["arniko-card".to_string()],
            requires_gpu: false,
            capabilities: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_empty() {
        let card = Card::new();
        let html = card.render();

        assert!(html.contains("arniko-card"));
        assert!(html.contains(r#"role="region""#));
        assert!(!html.contains("arniko-card-title"));
        assert!(!html.contains("arniko-card-body"));
    }

    #[test]
    fn test_card_with_title() {
        let card = Card::new().title("Test Title");
        let html = card.render();

        assert!(html.contains("arniko-card"));
        assert!(html.contains("arniko-card-title"));
        assert!(html.contains("Test Title"));
        assert!(!html.contains("arniko-card-body"));
    }

    #[test]
    fn test_card_with_body() {
        let card = Card::new().body("Test body content");
        let html = card.render();

        assert!(html.contains("arniko-card"));
        assert!(!html.contains("arniko-card-title"));
        assert!(html.contains("arniko-card-body"));
        assert!(html.contains("Test body content"));
    }

    #[test]
    fn test_card_with_title_and_body() {
        let card = Card::new().title("Card Title").body("Card body content");

        let html = card.render();

        assert!(html.contains("arniko-card"));
        assert!(html.contains("arniko-card-title"));
        assert!(html.contains("Card Title"));
        assert!(html.contains("arniko-card-body"));
        assert!(html.contains("Card body content"));
    }

    #[test]
    fn test_card_with_custom_class() {
        let card = Card::new().title("Test").class("custom-card highlighted");

        let html = card.render();

        assert!(html.contains("arniko-card"));
        assert!(html.contains("custom-card highlighted"));
    }

    #[test]
    fn test_card_metadata() {
        let card = Card::new().title("Test");
        let metadata = card.metadata();

        assert_eq!(metadata.css_classes.len(), 1);
        assert!(metadata.css_classes.contains(&"arniko-card".to_string()));
        assert!(!metadata.requires_gpu);
    }

    #[test]
    fn test_card_builder_pattern() {
        let card = Card::new()
            .title("Builder Card")
            .body("This card was built using the builder pattern")
            .class("builder-example");

        let html = card.render();
        assert!(html.contains("Builder Card"));
        assert!(html.contains("This card was built using the builder pattern"));
        assert!(html.contains("builder-example"));
    }

    #[test]
    fn test_card_with_html_content() {
        let card = Card::new()
            .title("HTML <em>Content</em>")
            .body("Body with <strong>bold</strong> text");

        let html = card.render();
        assert!(html.contains("HTML &lt;em&gt;Content&lt;/em&gt;"));
        assert!(html.contains("Body with &lt;strong&gt;bold&lt;/strong&gt; text"));
    }
}
