//! BarChart — Horizontal bar chart component.
//!
//! Generalized from the Khukuri SeverityBreakdown and BenchmarkChart.
//! Renders a series of horizontal bars scaled proportionally to the largest value.

#[cfg(feature = "components")]
use crate::{Component, ComponentMetadata};

/// A single bar entry in the chart.
#[derive(Clone, Debug)]
pub struct BarEntry {
    pub label: String,
    pub value: f64,
    pub color: String,
    pub glow: Option<String>,
}

impl BarEntry {
    pub fn new(label: &str, value: f64, color: &str) -> Self {
        Self {
            label: label.to_string(),
            value,
            color: color.to_string(),
            glow: None,
        }
    }

    pub fn with_glow(mut self, glow: &str) -> Self {
        self.glow = Some(glow.to_string());
        self
    }
}

// ── HTML Component ───────────────────────────────────────────────────────────

/// A horizontal bar chart rendered as HTML `<div>` elements.
///
/// Each bar is scaled relative to the largest value. Bars are rendered left
/// to right with a label, colored fill, and numeric value display.
#[cfg(feature = "components")]
pub struct BarChart {
    entries: Vec<BarEntry>,
    bar_height_px: u32,
    max_bar_width_px: u32,
    show_values: bool,
    class: String,
}

#[cfg(feature = "components")]
impl BarChart {
    /// Create a new empty bar chart.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            bar_height_px: 8,
            max_bar_width_px: 300,
            show_values: true,
            class: String::new(),
        }
    }

    /// Add a bar entry.
    pub fn add(mut self, entry: BarEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Set bar height in pixels.
    pub fn bar_height(mut self, px: u32) -> Self {
        self.bar_height_px = px;
        self
    }

    /// Set the maximum bar fill width in pixels.
    pub fn max_width(mut self, px: u32) -> Self {
        self.max_bar_width_px = px;
        self
    }

    /// Show/hide numeric values next to bars.
    pub fn show_values(mut self, show: bool) -> Self {
        self.show_values = show;
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        if self.entries.is_empty() {
            return format!(
                r#"<div class="arniko-bar-chart {}"><div class="arniko-bar-chart-empty">No data</div></div>"#,
                self.class
            );
        }

        let max_val = self.entries.iter()
            .map(|e| e.value)
            .fold(0.0_f64, f64::max)
            .max(1.0);

        let bars: String = self.entries.iter().map(|entry| {
            let pct = (entry.value / max_val * 100.0).min(100.0);
            let glow_css = entry.glow.as_ref()
                .map(|g| format!("box-shadow:0 0 8px {};", g))
                .unwrap_or_default();
            let value_text = if self.show_values {
                format!(
                    r#"<span class="arniko-bar-chart-value" style="color:{};">{:.1}</span>"#,
                    entry.color, entry.value
                )
            } else {
                String::new()
            };

            format!(
                r#"<div class="arniko-bar-chart-row">
                    <span class="arniko-bar-chart-label">{}</span>
                    <div class="arniko-bar-chart-track">
                        <div class="arniko-bar-chart-fill" style="width:{:.1}%;background:{};{};min-width:2px;"></div>
                    </div>
                    {}
                </div>"#,
                entry.label, pct, entry.color, glow_css, value_text
            )
        }).collect();

        format!(
            r#"<div class="arniko-bar-chart {}">{}</div>"#,
            self.class, bars
        )
    }
}

#[cfg(feature = "components")]
impl Default for BarChart {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "components")]
impl Component for BarChart {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec!["arniko-bar-chart".to_string()],
            requires_gpu: false,
            capabilities: vec![],
        }
    }
}

// ── Reactive View ────────────────────────────────────────────────────────────

#[cfg(feature = "reactive")]
use crate::reactive::{Signal, View, ReactiveHtml};

/// Create a reactive bar chart that updates when the entries signal changes.
#[cfg(feature = "reactive")]
pub fn bar_chart_reactive(
    entries_signal: Signal<Vec<BarEntry>>,
) -> Box<dyn View> {
    let html = entries_signal.derive(move |entries| {
        BarChart {
            entries,
            bar_height_px: 8,
            max_bar_width_px: 300,
            show_values: true,
            class: String::new(),
        }.render()
    });
    Box::new(ReactiveHtml::new(html))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "components")]
    #[test]
    fn test_bar_chart_empty() {
        let chart = BarChart::new();
        let html = chart.render();
        assert!(html.contains("No data"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_bar_chart_with_entries() {
        let chart = BarChart::new()
            .add(BarEntry::new("A", 50.0, "#ef4444"))
            .add(BarEntry::new("B", 30.0, "#f59e0b"));
        let html = chart.render();
        assert!(html.contains("A"));
        assert!(html.contains("B"));
        assert!(html.contains("arniko-bar-chart-fill"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_bar_chart_no_values() {
        let chart = BarChart::new()
            .add(BarEntry::new("X", 10.0, "#00f2ff"))
            .show_values(false);
        let html = chart.render();
        assert!(html.contains("X"));
        assert!(!html.contains("arniko-bar-chart-value"));
    }
}
