//! Separator component for Arniko

use crate::components::escape_html;
use crate::{Component, ComponentMetadata};

/// # Examples
///
/// ```rust,no_run
/// use arniko::Separator;
///
/// let sep = Separator::new().class("my-4");
/// ```
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
        format!(
            r#"<hr class="arniko-separator {}" role="separator" />"#,
            escape_html(&self.class)
        )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_separator_default() {
        let sep = Separator::new();
        let html = sep.render();
        assert!(html.contains("arniko-separator"));
        assert!(html.contains(r#"role="separator""#));
    }

    #[test]
    fn test_separator_custom_class() {
        let sep = Separator::new().class("my-sep");
        let html = sep.render();
        assert!(html.contains("my-sep"));
    }
}
