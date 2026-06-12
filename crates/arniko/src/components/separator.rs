//! Separator component for Arniko

use crate::{Component, ComponentMetadata};

pub struct Separator {
    class: String,
}

impl Separator {
    pub fn new() -> Self {
        Self {
            class: String::new(),
        }
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        format!(r#"<hr class="arniko-separator {}" />"#, self.class)
    }
}

impl Component for Separator {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::default()
    }
}
