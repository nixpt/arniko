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

    /// Golden-fixture table: every StatusVariant maps to its unique CSS class.
    /// Covers all 6 variants (Active, Success, Warning, Error, Info, Offline),
    /// closing the gap where the upstream test only spot-checks 3.
    #[test]
    fn test_status_badge_variants_golden() {
        let cases: &[(StatusVariant, &str)] = &[
            (StatusVariant::Active, "arniko-status-active"),
            (StatusVariant::Success, "arniko-status-success"),
            (StatusVariant::Warning, "arniko-status-warning"),
            (StatusVariant::Error, "arniko-status-error"),
            (StatusVariant::Info, "arniko-status-info"),
            (StatusVariant::Offline, "arniko-status-offline"),
        ];
        for (i, (variant, expected)) in cases.iter().enumerate() {
            let html = StatusBadge::new("m").variant(*variant).render();
            assert!(
                html.contains(expected),
                "StatusBadge case #{i}: missing '{expected}'",
            );
        }
    }

    /// Golden-fixture table: StatusBadge × class() × pulse() — the three
    /// independent orthogonal concerns must coexist in the rendered class
    /// attribute without trampling each other.
    #[test]
    fn test_status_badge_compose_golden() {
        let cases: &[(&'static str, StatusBadge, &[&'static str])] = &[
            (
                "default-active",
                StatusBadge::new("Online"),
                &["arniko-status-badge", "arniko-status-active"],
            ),
            (
                "pulse-on-active",
                StatusBadge::new("Live").pulse(true),
                &["arniko-status-active", "arniko-status-pulse"],
            ),
            (
                "error-with-class",
                StatusBadge::new("Down")
                    .variant(StatusVariant::Error)
                    .class("ml-2"),
                &["arniko-status-error", "ml-2"],
            ),
            (
                "offline-no-pulse",
                StatusBadge::new("Off").variant(StatusVariant::Offline),
                &["arniko-status-offline"],
            ),
        ];
        for (label, badge, expected_frags) in cases {
            let html = badge.render();
            for frag in *expected_frags {
                assert!(
                    html.contains(frag),
                    "StatusBadge[{label}]: missing fragment {frag:?}\nhtml={html}",
                );
            }
        }
    }
}
