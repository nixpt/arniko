//! MetricCard component for Arniko
//!
//! Provides metric card components for displaying data with icons and trends.

use crate::{Component, ComponentMetadata};
use crate::components::escape_html;

#[derive(Clone, PartialEq, Default)]
pub enum MetricColor {
    #[default]
    Blue,
    Green,
    Purple,
    Orange,
    Red,
    Cyan,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum MetricTrend {
    #[default]
    Stable,
    Up,
    Down,
}

pub struct MetricCard {
    title: String,
    value: String,
    subtitle: Option<String>,
    icon: Option<String>,
    color: MetricColor,
    trend: MetricTrend,
    class: String,
}

impl MetricCard {
    pub fn new(title: &str, value: &str) -> Self {
        Self {
            title: title.to_string(),
            value: value.to_string(),
            subtitle: None,
            icon: None,
            color: MetricColor::Blue,
            trend: MetricTrend::Stable,
            class: String::new(),
        }
    }

    pub fn subtitle(mut self, subtitle: &str) -> Self {
        self.subtitle = Some(subtitle.to_string());
        self
    }

    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    pub fn color(mut self, color: MetricColor) -> Self {
        self.color = color;
        self
    }

    pub fn trend(mut self, trend: MetricTrend) -> Self {
        self.trend = trend;
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        let color_class = match self.color {
            MetricColor::Blue => "arniko-metric-blue",
            MetricColor::Green => "arniko-metric-green",
            MetricColor::Purple => "arniko-metric-purple",
            MetricColor::Orange => "arniko-metric-orange",
            MetricColor::Red => "arniko-metric-red",
            MetricColor::Cyan => "arniko-metric-cyan",
        };

        let trend_class = match self.trend {
            MetricTrend::Stable => "arniko-trend-stable",
            MetricTrend::Up => "arniko-trend-up",
            MetricTrend::Down => "arniko-trend-down",
        };

        let icon_html = self
            .icon
            .as_ref()
            .map(|icon| {
                format!(
                    r#"<div class="arniko-metric-icon {}">{}</div>"#,
                    color_class, escape_html(icon)
                )
            })
            .unwrap_or_default();

        let subtitle_html = self
            .subtitle
            .as_ref()
            .map(|subtitle| format!(r#"<div class="arniko-metric-subtitle">{}</div>"#, escape_html(subtitle)))
            .unwrap_or_default();

        let trend_html = if !matches!(self.trend, MetricTrend::Stable) {
            format!(
                r#"<span class="arniko-metric-trend {}">{}</span>"#,
                trend_class,
                self.trend_symbol()
            )
        } else {
            String::new()
        };

        format!(
            r#"<div class="arniko-metric-card {} {}">
                <div class="arniko-metric-header">
                    <div class="arniko-metric-info">
                        <div class="arniko-metric-title">{}</div>
                        <div class="arniko-metric-value">{}{}{}</div>
                        {}
                    </div>
                    {}
                </div>
            </div>"#,
            escape_html(&self.class),
            color_class,
            escape_html(&self.title),
            escape_html(&self.value),
            trend_html,
            if self.subtitle.is_some() {
                ""
            } else {
                "</div>"
            },
            subtitle_html,
            icon_html
        )
    }

    fn trend_symbol(&self) -> &'static str {
        match self.trend {
            MetricTrend::Up => "↑",
            MetricTrend::Down => "↓",
            MetricTrend::Stable => "",
        }
    }
}

impl Component for MetricCard {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec!["arniko-metric-card".to_string()],
            requires_gpu: false,
            capabilities: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_card_basic() {
        let card = MetricCard::new("CPU Usage", "85%");
        let html = card.render();

        assert!(html.contains("arniko-metric-card"));
        assert!(html.contains("CPU Usage"));
        assert!(html.contains("85%"));
        assert!(html.contains("arniko-metric-blue"));
    }

    #[test]
    fn test_metric_card_with_subtitle() {
        let card = MetricCard::new("Memory", "8.2 GB").subtitle("16 GB total");
        let html = card.render();

        assert!(html.contains("16 GB total"));
        assert!(html.contains("arniko-metric-subtitle"));
    }

    #[test]
    fn test_metric_card_with_icon() {
        let card = MetricCard::new("Users", "1,234").icon("👥");
        let html = card.render();

        assert!(html.contains("👥"));
        assert!(html.contains("arniko-metric-icon"));
    }

    #[test]
    fn test_metric_card_colors() {
        let colors = [
            (MetricColor::Blue, "arniko-metric-blue"),
            (MetricColor::Green, "arniko-metric-green"),
            (MetricColor::Purple, "arniko-metric-purple"),
            (MetricColor::Orange, "arniko-metric-orange"),
            (MetricColor::Red, "arniko-metric-red"),
            (MetricColor::Cyan, "arniko-metric-cyan"),
        ];

        for (color, expected_class) in colors {
            let card = MetricCard::new("Test", "100").color(color);
            let html = card.render();
            assert!(html.contains(expected_class));
        }
    }

    #[test]
    fn test_metric_card_trends() {
        let trends = [
            (MetricTrend::Stable, "arniko-trend-stable", ""),
            (MetricTrend::Up, "arniko-trend-up", "↑"),
            (MetricTrend::Down, "arniko-trend-down", "↓"),
        ];

        for (trend, expected_class, expected_symbol) in trends {
            let card = MetricCard::new("Test", "100").trend(trend);
            let html = card.render();

            if !matches!(trend, MetricTrend::Stable) {
                assert!(html.contains(expected_class));
                assert!(html.contains(expected_symbol));
            }
        }
    }
}
