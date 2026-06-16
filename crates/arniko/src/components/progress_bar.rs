//! ProgressBar component for Arniko

use crate::{Component, ComponentMetadata};
use crate::components::escape_html;

#[derive(Clone, PartialEq, Default)]
pub enum ProgressColor {
    #[default]
    Accent,
    Blue,
    Green,
    Purple,
    Orange,
    Red,
}

pub struct ProgressBar {
    value: f32,
    max: f32,
    label: Option<String>,
    color: ProgressColor,
    show_percentage: bool,
    class: String,
}

impl ProgressBar {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            max: 100.0,
            label: None,
            color: ProgressColor::Accent,
            show_percentage: true,
            class: String::new(),
        }
    }

    pub fn max(mut self, max: f32) -> Self {
        self.max = max;
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn color(mut self, color: ProgressColor) -> Self {
        self.color = color;
        self
    }

    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        let percentage = (self.value / self.max * 100.0).min(100.0);
        let color_class = match self.color {
            ProgressColor::Accent => "arniko-progress-accent",
            ProgressColor::Blue => "arniko-progress-blue",
            ProgressColor::Green => "arniko-progress-green",
            ProgressColor::Purple => "arniko-progress-purple",
            ProgressColor::Orange => "arniko-progress-orange",
            ProgressColor::Red => "arniko-progress-red",
        };

        let header_html = if self.label.is_some() || self.show_percentage {
            let label_text = self.label.as_deref().unwrap_or("");
            let percentage_text = if self.show_percentage {
                format!("{}%", percentage as u32)
            } else {
                String::new()
            };

            format!(
                r#"<div class="arniko-progress-header">
                <span class="arniko-progress-label">{}</span>
                <span class="arniko-progress-pct">{}</span>
            </div>"#,
                escape_html(label_text), percentage_text
            )
        } else {
            String::new()
        };

        let aria_label = self.label.as_deref().unwrap_or("Progress");

        format!(
            r#"<div class="arniko-progress {}" role="progressbar" aria-valuenow="{}" aria-valuemin="0" aria-valuemax="100" aria-label="{}">
                {}
                <div class="arniko-progress-track">
                    <div class="arniko-progress-bar {}" style="width: {}%"></div>
                </div>
            </div>"#,
            escape_html(&self.class),
            percentage as u32,
            escape_html(aria_label),
            header_html,
            color_class,
            percentage
        )
    }
}

impl Component for ProgressBar {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec!["arniko-progress".to_string()],
            requires_gpu: false,
            capabilities: vec![],
        }
    }
}
