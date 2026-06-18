//! Badge component for Arniko
//!
//! Provides badge components for status indicators and labels.

use crate::components::escape_html;
use crate::{Component, ComponentMetadata};

#[derive(Clone, PartialEq, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Success,
    Warning,
    Error,
    Info,
    Purple,
}

/// # Examples
///
/// ```rust,no_run
/// use arniko::{Badge, BadgeVariant};
///
/// let badge = Badge::new("Live")
///     .variant(BadgeVariant::Success);
/// ```
pub struct Badge {
    text: String,
    variant: BadgeVariant,
    class: String,
}

impl Badge {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            variant: BadgeVariant::Default,
            class: String::new(),
        }
    }

    pub fn variant(mut self, v: BadgeVariant) -> Self {
        self.variant = v;
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        let variant_class = match self.variant {
            BadgeVariant::Default => "arniko-badge-default",
            BadgeVariant::Success => "arniko-badge-success",
            BadgeVariant::Warning => "arniko-badge-warning",
            BadgeVariant::Error => "arniko-badge-error",
            BadgeVariant::Info => "arniko-badge-info",
            BadgeVariant::Purple => "arniko-badge-purple",
        };

        format!(
            r#"<span class="arniko-badge {} {}" role="status">{}</span>"#,
            variant_class,
            escape_html(&self.class),
            escape_html(&self.text)
        )
    }
}

impl Component for Badge {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec![
                "arniko-badge".to_string(),
                match self.variant {
                    BadgeVariant::Default => "arniko-badge-default",
                    BadgeVariant::Success => "arniko-badge-success",
                    BadgeVariant::Warning => "arniko-badge-warning",
                    BadgeVariant::Error => "arniko-badge-error",
                    BadgeVariant::Info => "arniko-badge-info",
                    BadgeVariant::Purple => "arniko-badge-purple",
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
    fn test_badge_default() {
        let badge = Badge::new("Test");
        let html = badge.render();

        assert!(html.contains("arniko-badge"));
        assert!(html.contains("arniko-badge-default"));
        assert!(html.contains("Test"));
        assert!(html.contains(r#"role="status""#));
    }

    #[test]
    fn test_badge_variants() {
        let variants = [
            (BadgeVariant::Default, "arniko-badge-default"),
            (BadgeVariant::Success, "arniko-badge-success"),
            (BadgeVariant::Warning, "arniko-badge-warning"),
            (BadgeVariant::Error, "arniko-badge-error"),
        ];

        for (variant, expected_class) in variants {
            let badge = Badge::new("Test").variant(variant);
            let html = badge.render();
            assert!(html.contains(expected_class));
        }
    }
}
