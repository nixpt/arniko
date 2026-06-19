//! Tag / chip component for Arniko dashboard kit.
//!
//! A pill-shaped label for categorization and filtering UIs.
//! Visually distinct from `Badge` (pill shape, border, no background fill).

use crate::components::escape_html;
use crate::{Component, ComponentMetadata};

/// Color variant for a `Tag`.
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum TagVariant {
    #[default]
    Default,
    Success,
    Warning,
    Error,
    Info,
}

/// A pill-shaped label for categorization.
///
/// Use `Badge` for status indicators; use `Tag` for user-applied labels,
/// filter chips, and category markers.
///
/// # Examples
///
/// ```rust,no_run
/// use arniko::{Tag, TagVariant};
///
/// let tag = Tag::new("production").variant(TagVariant::Success);
/// let html = tag.render();
/// ```
pub struct Tag {
    label: String,
    variant: TagVariant,
    class: String,
}

impl Tag {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            variant: TagVariant::Default,
            class: String::new(),
        }
    }

    pub fn variant(mut self, v: TagVariant) -> Self {
        self.variant = v;
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        let variant_class = match self.variant {
            TagVariant::Default => "arniko-tag-default",
            TagVariant::Success => "arniko-tag-success",
            TagVariant::Warning => "arniko-tag-warning",
            TagVariant::Error => "arniko-tag-error",
            TagVariant::Info => "arniko-tag-info",
        };
        format!(
            r#"<span class="arniko-tag {} {}">{}</span>"#,
            variant_class,
            escape_html(&self.class),
            escape_html(&self.label),
        )
    }
}

impl Component for Tag {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec!["arniko-tag".to_string()],
            requires_gpu: false,
            capabilities: vec![],
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_default() {
        let html = Tag::new("env:prod").render();
        assert!(html.contains("arniko-tag"));
        assert!(html.contains("arniko-tag-default"));
        assert!(html.contains("env:prod"));
    }

    #[test]
    fn test_tag_variants() {
        use TagVariant::*;
        for (v, cls) in [
            (Success, "arniko-tag-success"),
            (Warning, "arniko-tag-warning"),
            (Error, "arniko-tag-error"),
            (Info, "arniko-tag-info"),
        ] {
            assert!(Tag::new("x").variant(v).render().contains(cls));
        }
    }

    #[test]
    fn test_tag_xss() {
        let html = Tag::new("<script>").render();
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
