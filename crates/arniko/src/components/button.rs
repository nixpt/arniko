//! Button component for Arniko
//!
//! Provides customizable button components with multiple variants and sizes.
//! Supports rendering as both `<button>` and `<a>` (link) elements.

use crate::components::escape_html;
use crate::{Component, ComponentMetadata};

#[derive(Clone, PartialEq, Default)]
pub enum ButtonVariant {
    #[default]
    Default,
    Destructive,
    Outline,
    Secondary,
    Ghost,
    Accent,
}

#[derive(Clone, PartialEq, Default)]
pub enum ButtonSize {
    #[default]
    Default,
    Sm,
    Lg,
    Icon,
}

pub struct Button {
    label: String,
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: bool,
    class: String,
    href: Option<String>,
}

impl Button {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            variant: ButtonVariant::Default,
            size: ButtonSize::Default,
            disabled: false,
            class: String::new(),
            href: None,
        }
    }

    pub fn variant(mut self, v: ButtonVariant) -> Self {
        self.variant = v;
        self
    }

    pub fn size(mut self, s: ButtonSize) -> Self {
        self.size = s;
        self
    }

    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    /// Set the href URL, making this button render as an `<a>` link element
    /// instead of a `<button>` element.
    pub fn href(mut self, url: &str) -> Self {
        self.href = Some(url.to_string());
        self
    }

    /// Convenience constructor for a link button in one call.
    pub fn link(label: &str, url: &str) -> Self {
        Self::new(label).href(url)
    }

    pub fn render(&self) -> String {
        let variant_class = match self.variant {
            ButtonVariant::Default => "arniko-btn-default",
            ButtonVariant::Destructive => "arniko-btn-destructive",
            ButtonVariant::Outline => "arniko-btn-outline",
            ButtonVariant::Secondary => "arniko-btn-secondary",
            ButtonVariant::Ghost => "arniko-btn-ghost",
            ButtonVariant::Accent => "arniko-btn-accent",
        };
        let size_class = match self.size {
            ButtonSize::Default => "",
            ButtonSize::Sm => "arniko-btn-sm",
            ButtonSize::Lg => "arniko-btn-lg",
            ButtonSize::Icon => "arniko-btn-icon",
        };
        let disabled = if self.disabled { " disabled" } else { "" };
        let class_str = if self.class.is_empty() {
            format!("arniko-btn {} {}", variant_class, size_class)
        } else {
            format!("arniko-btn {} {} {}", variant_class, size_class, self.class)
        };

        if let Some(ref url) = self.href {
            format!(
                r#"<a href="{}" class="{}"{} role="button">{}</a>"#,
                escape_html(url),
                class_str,
                disabled,
                escape_html(&self.label)
            )
        } else {
            format!(
                r#"<button class="{}"{}>{}</button>"#,
                class_str,
                disabled,
                escape_html(&self.label)
            )
        }
    }
}

impl Component for Button {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec![
                "arniko-btn".to_string(),
                match self.variant {
                    ButtonVariant::Default => "arniko-btn-default",
                    ButtonVariant::Destructive => "arniko-btn-destructive",
                    ButtonVariant::Outline => "arniko-btn-outline",
                    ButtonVariant::Secondary => "arniko-btn-secondary",
                    ButtonVariant::Ghost => "arniko-btn-ghost",
                    ButtonVariant::Accent => "arniko-btn-accent",
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
    fn test_button_default() {
        let button = Button::new("Click me");
        let html = button.render();

        assert!(html.contains("arniko-btn"));
        assert!(html.contains("arniko-btn-default"));
        assert!(html.contains("Click me"));
        assert!(!html.contains("disabled"));
        assert!(html.contains("<button"));
    }

    #[test]
    fn test_button_variants() {
        let variants = [
            (ButtonVariant::Default, "arniko-btn-default"),
            (ButtonVariant::Destructive, "arniko-btn-destructive"),
            (ButtonVariant::Outline, "arniko-btn-outline"),
            (ButtonVariant::Secondary, "arniko-btn-secondary"),
            (ButtonVariant::Ghost, "arniko-btn-ghost"),
            (ButtonVariant::Accent, "arniko-btn-accent"),
        ];

        for (variant, expected_class) in variants {
            let button = Button::new("Test").variant(variant);
            let html = button.render();
            assert!(html.contains(expected_class));
        }
    }

    #[test]
    fn test_button_sizes() {
        let sizes = [
            (ButtonSize::Default, ""),
            (ButtonSize::Sm, "arniko-btn-sm"),
            (ButtonSize::Lg, "arniko-btn-lg"),
            (ButtonSize::Icon, "arniko-btn-icon"),
        ];

        for (size, expected_class) in sizes {
            let button = Button::new("Test").size(size);
            let html = button.render();

            if !expected_class.is_empty() {
                assert!(html.contains(expected_class));
            }
        }
    }

    #[test]
    fn test_button_disabled() {
        let button = Button::new("Disabled").disabled(true);
        let html = button.render();

        assert!(html.contains("disabled"));
    }

    #[test]
    fn test_button_custom_class() {
        let button = Button::new("Custom").class("my-class another-class");
        let html = button.render();

        assert!(html.contains("my-class another-class"));
    }

    #[test]
    fn test_button_metadata() {
        let button = Button::new("Test").variant(ButtonVariant::Accent);
        let metadata = button.metadata();

        assert_eq!(metadata.css_classes.len(), 2);
        assert!(metadata.css_classes.contains(&"arniko-btn".to_string()));
        assert!(
            metadata
                .css_classes
                .contains(&"arniko-btn-accent".to_string())
        );
        assert!(!metadata.requires_gpu);
    }

    #[test]
    fn test_button_builder_pattern() {
        let button = Button::new("Builder Test")
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::Lg)
            .disabled(true)
            .class("custom-btn");

        let html = button.render();
        assert!(html.contains("arniko-btn-outline"));
        assert!(html.contains("arniko-btn-lg"));
        assert!(html.contains("disabled"));
        assert!(html.contains("custom-btn"));
        assert!(html.contains("Builder Test"));
    }

    #[test]
    fn test_button_href_renders_as_link() {
        let button = Button::link("Visit", "https://example.com");
        let html = button.render();

        assert!(html.contains("<a"));
        assert!(html.contains("href=\"https://example.com\""));
        assert!(html.contains("Visit"));
        assert!(html.contains("arniko-btn"));
        assert!(html.contains(r#"role="button""#));
        assert!(!html.contains("<button"));
    }

    #[test]
    fn test_button_href_builder() {
        let button = Button::new("Docs")
            .href("/docs")
            .variant(ButtonVariant::Accent);
        let html = button.render();

        assert!(html.contains("<a"));
        assert!(html.contains("href=\"/docs\""));
        assert!(html.contains("arniko-btn-accent"));
    }

    #[test]
    fn test_button_default_renders_as_button() {
        let button = Button::new("Click");
        let html = button.render();
        assert!(html.contains("<button"));
        assert!(!html.contains("href="));
    }
}
