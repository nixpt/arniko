//! Tooltip component for Arniko

use crate::components::escape_html;
use crate::{Component, ComponentMetadata};

#[derive(Clone, PartialEq, Default)]
pub enum TooltipPosition {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

/// # Examples
///
/// ```rust,no_run
/// use arniko::{Tooltip, TooltipPosition};
///
/// let tip = Tooltip::new("Hover me", "This is the tooltip text")
///     .position(TooltipPosition::Bottom);
/// ```
pub struct Tooltip {
    text: String,
    tooltip: String,
    position: TooltipPosition,
    class: String,
}

impl Tooltip {
    pub fn new(text: &str, tooltip: &str) -> Self {
        Self {
            text: text.to_string(),
            tooltip: tooltip.to_string(),
            position: TooltipPosition::Top,
            class: String::new(),
        }
    }

    pub fn position(mut self, position: TooltipPosition) -> Self {
        self.position = position;
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        let position_attr = match self.position {
            TooltipPosition::Top => "top",
            TooltipPosition::Bottom => "bottom",
            TooltipPosition::Left => "left",
            TooltipPosition::Right => "right",
        };

        format!(
            r#"<span class="arniko-tooltip {}" data-tooltip="{}" data-tooltip-position="{}" role="tooltip" tabindex="0">{}</span>"#,
            escape_html(&self.class),
            escape_html(&self.tooltip),
            position_attr,
            escape_html(&self.text)
        )
    }

    pub fn render_end() -> &'static str {
        "</span>"
    }
}

impl Component for Tooltip {
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
    fn test_tooltip_default() {
        let tip = Tooltip::new("Hover me", "Helpful text");
        let html = tip.render();
        assert!(html.contains("arniko-tooltip"));
        assert!(html.contains("Hover me"));
        assert!(html.contains(r#"data-tooltip="Helpful text""#));
        assert!(html.contains(r#"role="tooltip""#));
        assert!(html.contains(r#"tabindex="0""#));
    }

    #[test]
    fn test_tooltip_positions() {
        let top = Tooltip::new("A", "B").position(TooltipPosition::Top);
        assert!(top.render().contains(r#"data-tooltip-position="top""#));

        let bottom = Tooltip::new("A", "B").position(TooltipPosition::Bottom);
        assert!(
            bottom
                .render()
                .contains(r#"data-tooltip-position="bottom""#)
        );
    }

    #[test]
    fn test_tooltip_escapes_html() {
        let tip = Tooltip::new("<script>", "&evil");
        let html = tip.render();
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&amp;evil"));
    }
}
