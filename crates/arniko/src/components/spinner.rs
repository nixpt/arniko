//! Spinner component for Arniko

use crate::{Component, ComponentMetadata};

#[derive(Clone, PartialEq, Default)]
pub enum SpinnerSize {
    #[default]
    Default,
    Sm,
    Lg,
}

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
