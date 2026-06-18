//! Spinner component for Arniko

use crate::{Component, ComponentMetadata};

#[derive(Clone, PartialEq, Default)]
pub enum SpinnerSize {
    #[default]
    Default,
    Sm,
    Lg,
}

/// # Examples
///
/// ```rust,no_run
/// use arniko::{Spinner, SpinnerSize};
///
/// let spinner = Spinner::new().size(SpinnerSize::Lg);
/// ```
pub struct Spinner {
    size: SpinnerSize,
    class: String,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            size: SpinnerSize::Default,
            class: String::new(),
        }
    }

    pub fn size(mut self, size: SpinnerSize) -> Self {
        self.size = size;
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        let size_class = match self.size {
            SpinnerSize::Default => "",
            SpinnerSize::Sm => " arniko-spinner-sm",
            SpinnerSize::Lg => " arniko-spinner-lg",
        };

        format!(
            r#"<div class="arniko-spinner{}{}" role="status" aria-label="Loading"></div>"#,
            size_class, self.class
        )
    }
}

impl Component for Spinner {
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
    fn test_spinner_default() {
        let spinner = Spinner::new();
        let html = spinner.render();
        assert!(html.contains("arniko-spinner"));
        assert!(html.contains(r#"role="status""#));
        assert!(html.contains(r#"aria-label="Loading""#));
    }

    #[test]
    fn test_spinner_sizes() {
        let sm = Spinner::new().size(SpinnerSize::Sm);
        assert!(sm.render().contains("arniko-spinner-sm"));

        let lg = Spinner::new().size(SpinnerSize::Lg);
        assert!(lg.render().contains("arniko-spinner-lg"));
    }

    #[test]
    fn test_spinner_custom_class() {
        let spinner = Spinner::new().class("my-spinner");
        assert!(spinner.render().contains("my-spinner"));
    }
}
