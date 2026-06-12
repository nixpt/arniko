//! Input component for Arniko
//!
//! Provides customizable input components for forms and user input.

use crate::{Component, ComponentMetadata};

pub struct Input {
    input_type: String,
    placeholder: String,
    value: String,
    disabled: bool,
    class: String,
    name: String,
}

impl Input {
    pub fn new() -> Self {
        Self {
            input_type: "text".to_string(),
            placeholder: String::new(),
            value: String::new(),
            disabled: false,
            class: String::new(),
            name: String::new(),
        }
    }

    pub fn input_type(mut self, t: &str) -> Self {
        self.input_type = t.to_string();
        self
    }

    pub fn placeholder(mut self, p: &str) -> Self {
        self.placeholder = p.to_string();
        self
    }

    pub fn value(mut self, v: &str) -> Self {
        self.value = v.to_string();
        self
    }

    pub fn name(mut self, n: &str) -> Self {
        self.name = n.to_string();
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
        let disabled = if self.disabled { " disabled" } else { "" };
        let name = if self.name.is_empty() {
            String::new()
        } else {
            format!(r#" name="{}""#, self.name)
        };

        format!(
            r#"<input class="arniko-input {}" type="{}" placeholder="{}" value="{}"{}{} />"#,
            self.class, self.input_type, self.placeholder, self.value, name, disabled
        )
    }
}

impl Component for Input {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec!["arniko-input".to_string()],
            requires_gpu: false,
            capabilities: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_default() {
        let input = Input::new();
        let html = input.render();

        assert!(html.contains("arniko-input"));
        assert!(html.contains(r#"type="text""#));
        assert!(!html.contains("disabled"));
    }

    #[test]
    fn test_input_with_placeholder() {
        let input = Input::new().placeholder("Enter your name");
        let html = input.render();

        assert!(html.contains(r#"placeholder="Enter your name""#));
    }

    #[test]
    fn test_input_with_value() {
        let input = Input::new().value("John Doe");
        let html = input.render();

        assert!(html.contains(r#"value="John Doe""#));
    }

    #[test]
    fn test_input_with_name() {
        let input = Input::new().name("username");
        let html = input.render();

        assert!(html.contains(r#"name="username""#));
    }

    #[test]
    fn test_input_disabled() {
        let input = Input::new().disabled(true);
        let html = input.render();

        assert!(html.contains("disabled"));
    }

    #[test]
    fn test_input_different_types() {
        let types = ["text", "password", "email", "number", "tel"];

        for input_type in types {
            let input = Input::new().input_type(input_type);
            let html = input.render();
            assert!(html.contains(&format!(r#"type="{}""#, input_type)));
        }
    }

    #[test]
    fn test_input_with_custom_class() {
        let input = Input::new().class("custom-input form-control");
        let html = input.render();

        assert!(html.contains("custom-input form-control"));
    }

    #[test]
    fn test_input_complete() {
        let input = Input::new()
            .input_type("email")
            .placeholder("Enter your email")
            .value("test@example.com")
            .name("email")
            .class("email-input")
            .disabled(false);

        let html = input.render();
        assert!(html.contains(r#"type="email""#));
        assert!(html.contains(r#"placeholder="Enter your email""#));
        assert!(html.contains(r#"value="test@example.com""#));
        assert!(html.contains(r#"name="email""#));
        assert!(html.contains("email-input"));
    }

    #[test]
    fn test_input_metadata() {
        let input = Input::new().class("test-input");
        let metadata = input.metadata();

        assert_eq!(metadata.css_classes.len(), 1);
        assert!(metadata.css_classes.contains(&"arniko-input".to_string()));
        assert!(!metadata.requires_gpu);
    }

    #[test]
    fn test_input_builder_pattern() {
        let input = Input::new()
            .input_type("password")
            .placeholder("Enter password")
            .name("password")
            .class("password-field");

        let html = input.render();
        assert!(html.contains(r#"type="password""#));
        assert!(html.contains(r#"placeholder="Enter password""#));
        assert!(html.contains(r#"name="password""#));
        assert!(html.contains("password-field"));
    }
}
