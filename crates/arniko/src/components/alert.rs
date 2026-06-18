//! Alert component for Arniko
//!
//! Provides alert components for notifications and messages.

use crate::components::escape_html;
use crate::{Component, ComponentMetadata};

#[derive(Clone, PartialEq, Default)]
pub enum AlertVariant {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

/// # Examples
///
/// ```rust,no_run
/// use arniko::{Alert, AlertVariant};
///
/// let alert = Alert::new("Deployment successful")
///     .variant(AlertVariant::Success);
/// ```
pub struct Alert {
    message: String,
    variant: AlertVariant,
    class: String,
}

impl Alert {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            variant: AlertVariant::Info,
            class: String::new(),
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

    pub fn render(&self) -> String {
        let variant_class = match self.variant {
            AlertVariant::Info => "arniko-alert-info",
            AlertVariant::Success => "arniko-alert-success",
            AlertVariant::Warning => "arniko-alert-warning",
            AlertVariant::Error => "arniko-alert-error",
        };

        format!(
            r#"<div class="arniko-alert {} {}" role="alert" aria-live="polite">
                <div class="arniko-alert-icon">{}</div>
                <div class="arniko-alert-message">{}</div>
            </div>"#,
            variant_class,
            escape_html(&self.class),
            self.icon(),
            escape_html(&self.message)
        )
    }

    fn icon(&self) -> &'static str {
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
