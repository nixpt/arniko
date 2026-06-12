//! Button component for Arniko
//!
//! Provides customizable button components with multiple variants and sizes.

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
}

impl Button {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            variant: ButtonVariant::Default,
            size: ButtonSize::Default,
            disabled: false,
            class: String::new(),
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

        format!(
            r#"<button class="arniko-btn {} {} {}"{}>
                {}
            </button>"#,
            variant_class, size_class, self.class, disabled, self.label
        )
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
}
