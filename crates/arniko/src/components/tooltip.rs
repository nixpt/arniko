//! Tooltip component for Arniko

use crate::{Component, ComponentMetadata};
use crate::components::escape_html;

#[derive(Clone, PartialEq, Default)]
pub enum TooltipPosition {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

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
            r#"<span class="arniko-tooltip {}" data-tooltip="{}" data-tooltip-position="{}" role="tooltip">{}</span>"#,
            escape_html(&self.class), escape_html(&self.tooltip), position_attr, escape_html(&self.text)
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
