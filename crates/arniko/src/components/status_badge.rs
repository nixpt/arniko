//! StatusBadge component for Arniko
//!
//! A badge with a colored dot indicator and optional pulse animation.
//! Maps to capsule-ui's StatusBadge component.

use crate::components::escape_html;
use crate::{Component, ComponentMetadata};

#[derive(Clone, Copy, PartialEq, Default)]
pub enum StatusVariant {
    #[default]
    Active,
    Success,
    Warning,
    Error,
    Info,
    Offline,
}

/// # Examples
///
/// ```rust,no_run
/// use arniko::{StatusBadge, StatusVariant};
///
/// let badge = StatusBadge::new("Online")
///     .variant(StatusVariant::Active)
///     .pulse(true);
///
/// let html = badge.render();
/// ```
pub struct StatusBadge {
    label: String,
    variant: StatusVariant,
    pulse: bool,
    class: String,
}

impl StatusBadge {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            variant: StatusVariant::Active,
            pulse: false,
            class: String::new(),
        }
    }

    pub fn variant(mut self, v: StatusVariant) -> Self {
        self.variant = v;
        self
    }

    pub fn pulse(mut self, pulse: bool) -> Self {
        self.pulse = pulse;
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        let variant_class = match self.variant {
            StatusVariant::Active => "arniko-status-active",
            StatusVariant::Success => "arniko-status-success",
            StatusVariant::Warning => "arniko-status-warning",
            StatusVariant::Error => "arniko-status-error",
            StatusVariant::Info => "arniko-status-info",
            StatusVariant::Offline => "arniko-status-offline",
        };
        let pulse_class = if self.pulse {
            " arniko-status-pulse"
        } else {
            ""
        };

        format!(
            r#"<span class="arniko-status-badge {} {}{}" role="status">{}</span>"#,
            variant_class,
            escape_html(&self.class),
            pulse_class,
            escape_html(&self.label)
        )
    }
}

impl Component for StatusBadge {
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
    fn test_status_badge_basic() {
        let badge = StatusBadge::new("Online");
        let html = badge.render();
        assert!(html.contains("arniko-status-badge"));
        assert!(html.contains("Online"));
        assert!(html.contains("arniko-status-active"));
        assert!(html.contains(r#"role="status""#));
    }

    #[test]
    fn test_status_badge_variants() {
        let cases = [
            (StatusVariant::Success, "arniko-status-success"),
            (StatusVariant::Warning, "arniko-status-warning"),
            (StatusVariant::Error, "arniko-status-error"),
        ];
        for (variant, expected) in cases {
            let badge = StatusBadge::new("test").variant(variant);
            assert!(badge.render().contains(expected));
        }
    }

    #[test]
    fn test_status_badge_pulse() {
        let badge = StatusBadge::new("Live").pulse(true);
        assert!(badge.render().contains("arniko-status-pulse"));
    }
}
