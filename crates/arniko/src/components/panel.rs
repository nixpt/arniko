//! Panel component for Arniko
//!
//! A container with a title bar and body content.
//! Maps to capsule-ui's Panel component.

use crate::components::escape_html;
use crate::{Component, ComponentMetadata};

pub struct Panel {
    title: Option<String>,
    body: Option<String>,
    icon: Option<String>,
    closable: bool,
    class: String,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            title: None,
            body: None,
            icon: None,
            closable: false,
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

    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
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
            .map(|i| {
                format!(
                    r#"<span class="arniko-panel-icon">{}</span>"#,
                    escape_html(i)
                )
            })
            .unwrap_or_default();

        let close_btn = if self.closable {
            r#"<button class="arniko-panel-close" aria-label="Close">&times;</button>"#
        } else {
            ""
        };

        let header_html = match &self.title {
            Some(t) => format!(
                r#"<div class="arniko-panel-header">{}<span class="arniko-panel-title">{}</span>{}</div>"#,
                icon_html,
                escape_html(t),
                close_btn
            ),
            None => String::new(),
        };

        let body_html = self
            .body
            .as_ref()
            .map(|b| format!(r#"<div class="arniko-panel-body">{}</div>"#, escape_html(b)))
            .unwrap_or_default();

        format!(
            r#"<div class="arniko-panel {}">{}{}</div>"#,
            escape_html(&self.class),
            header_html,
            body_html
        )
    }
}

impl Component for Panel {
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
    fn test_panel_empty() {
        let panel = Panel::new();
        let html = panel.render();
        assert!(html.contains("arniko-panel"));
    }

    #[test]
    fn test_panel_with_title_and_body() {
        let panel = Panel::new().title("Settings").body("Configure your app");
        let html = panel.render();
        assert!(html.contains("arniko-panel-title"));
        assert!(html.contains("Settings"));
        assert!(html.contains("Configure your app"));
    }

    #[test]
    fn test_panel_closable() {
        let panel = Panel::new().title("Dialog").closable(true);
        let html = panel.render();
        assert!(html.contains("arniko-panel-close"));
        assert!(html.contains("&times;"));
    }
}
