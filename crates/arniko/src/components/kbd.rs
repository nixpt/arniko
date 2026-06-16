//! Kbd (Keyboard) component for Arniko

use crate::{Component, ComponentMetadata};
use crate::components::escape_html;

pub struct Kbd {
    key: String,
    class: String,
}

impl Kbd {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            class: String::new(),
        }
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        format!(
            r#"<kbd class="arniko-kbd {}">{}</kbd>"#,
            escape_html(&self.class), escape_html(&self.key)
        )
    }
}

impl Component for Kbd {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::default()
    }
}
