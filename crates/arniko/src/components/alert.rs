//! Alert component for Arniko
//!
//! Provides alert components for notifications and messages.
//! Supports configurable icon themes including MacTahoe SVG icons.

use crate::{Component, ComponentMetadata};
use crate::components::escape_html;
use crate::components::icon_theme::IconThemeVariant;

#[derive(Clone, PartialEq, Default)]
pub enum AlertVariant {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

pub struct Alert {
    message: String,
    variant: AlertVariant,
    class: String,
    icon_theme: IconThemeVariant,
}

impl Alert {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            variant: AlertVariant::Info,
            class: String::new(),
            icon_theme: IconThemeVariant::default(),
        }
    }

    pub fn variant(mut self, v: AlertVariant) -> Self {
        self.variant = v;
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn icon_theme(mut self, theme: IconThemeVariant) -> Self {
        self.icon_theme = theme;
        self
    }

    pub fn render(&self) -> String {
        let variant_class = match self.variant {
            AlertVariant::Info => "arniko-alert-info",
            AlertVariant::Success => "arniko-alert-success",
            AlertVariant::Warning => "arniko-alert-warning",
            AlertVariant::Error => "arniko-alert-error",
        };

        let icon = self.icon();
        let icon_html = if !icon.is_empty() {
            format!(r#" <div class="arniko-alert-icon">{}</div>"#, escape_html(&icon))
        } else {
            String::new()
        };

        format!(
            r#"<div class="arniko-alert {}{}" role="alert" aria-live="polite">
                <div class="arniko-alert-message">{}</div>
            </div>"#,
            variant_class,
            escape_html(&self.class),
            escape_html(&self.message)
        ) + &icon_html
    }

    fn icon(&self) -> String {
        match self.variant {
            AlertVariant::Info => "ℹ️",
            AlertVariant::Success => "✅",
            AlertVariant::Warning => "⚠️",
            AlertVariant::Error => "❌",
        }
    }
}

impl Component for Alert {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec![
                "arniko-alert".to_string(),
                match self.variant {
                    AlertVariant::Info => "arniko-alert-info",
                    AlertVariant::Success => "arniko-alert-success",
                    AlertVariant::Warning => "arniko-alert-warning",
                    AlertVariant::Error => "arniko-alert-error",
                }
                .to_string(),
            ],
            requires_gpu: false,
            capabilities: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_info() {
        let alert = Alert::new("Info message");
        let html = alert.render();

        assert!(html.contains("arniko-alert"));
        assert!(html.contains("arniko-alert-info"));
        assert!(html.contains("Info message"));
        assert!(html.contains("ℹ️"));
        assert!(html.contains(r#"role="alert""#));
    }

    #[test]
    fn test_alert_variants() {
        let variants = [
            (AlertVariant::Info, "arniko-alert-info", "ℹ️"),
            (AlertVariant::Success, "arniko-alert-success", "✅"),
            (AlertVariant::Warning, "arniko-alert-warning", "⚠️"),
            (AlertVariant::Error, "arniko-alert-error", "❌"),
        ];

        for (variant, expected_class, expected_icon) in variants {
            let alert = Alert::new("Test").variant(variant);
            let html = alert.render();
            assert!(html.contains(expected_class));
            assert!(html.contains(expected_icon));
        }
    }
}
