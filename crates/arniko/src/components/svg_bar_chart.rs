//! SvgBarChart — SVG-based horizontal bar chart component.
//!
//! Generalized from the Khukuri BenchmarkChart. Renders actual `<svg>` elements
//! with `<rect>` bars and `<text>` labels, scaled proportionally to the largest value.
//!
//! Distinct from the HTML-based `BarChart` which uses `<div>` elements.

#[cfg(feature = "components")]
use crate::components::escape_html;
#[cfg(feature = "components")]
use crate::{Component, ComponentMetadata};

/// A single bar entry in the SVG chart.
#[derive(Clone, Debug)]
pub struct SvgBarEntry {
    pub label: String,
    pub value: f64,
    pub color: String,
}

impl SvgBarEntry {
    pub fn new(label: &str, value: f64, color: &str) -> Self {
        Self {
            label: label.to_string(),
            value,
            color: color.to_string(),
        }
    }
}

// ── HTML Component ───────────────────────────────────────────────────────────

/// An SVG horizontal bar chart rendered as an inline `<svg>` element.
///
/// Each bar is a `<rect>` scaled relative to the largest value. Labels are
/// `<text>` elements positioned on the left; values on the right.
#[cfg(feature = "components")]
pub struct SvgBarChart {
    entries: Vec<SvgBarEntry>,
    title: Option<String>,
    title_icon: Option<String>,
    empty_message: String,
    bar_height: f64,
    max_bar_width: f64,
    row_height: f64,
    label_width: f64,
    label_max_chars: usize,
    value_suffix: String,
    class: String,
}

#[cfg(feature = "components")]
impl SvgBarChart {
    /// Create a new empty SVG bar chart with sensible defaults.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            title: None,
            title_icon: None,
            empty_message: "No data".to_string(),
            bar_height: 12.0,
            max_bar_width: 360.0,
            row_height: 28.0,
            label_width: 132.0,
            label_max_chars: 18,
            value_suffix: String::new(),
            class: String::new(),
        }
    }

    /// Add a bar entry.
    pub fn add(mut self, entry: SvgBarEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Add multiple bar entries at once.
    pub fn add_entries(mut self, entries: &[SvgBarEntry]) -> Self {
        self.entries.extend(entries.iter().cloned());
        self
    }

    /// Set the chart title shown above the SVG.
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Set an icon emoji shown next to the title.
    pub fn title_icon(mut self, icon: &str) -> Self {
        self.title_icon = Some(icon.to_string());
        self
    }

    /// Set the empty-state message.
    pub fn empty_message(mut self, msg: &str) -> Self {
        self.empty_message = msg.to_string();
        self
    }

    /// Height of each bar `<rect>` in SVG units.
    pub fn bar_height(mut self, px: f64) -> Self {
        self.bar_height = px;
        self
    }

    /// Maximum bar fill width in SVG units.
    pub fn max_bar_width(mut self, px: f64) -> Self {
        self.max_bar_width = px;
        self
    }

    /// Row height between bars in SVG units.
    pub fn row_height(mut self, px: f64) -> Self {
        self.row_height = px;
        self
    }

    /// Width reserved for labels in SVG units.
    pub fn label_width(mut self, px: f64) -> Self {
        self.label_width = px;
        self
    }

    /// Max characters before truncating labels.
    pub fn label_max_chars(mut self, n: usize) -> Self {
        self.label_max_chars = n;
        self
    }

    /// Suffix appended to each value (e.g. "ms", "%", "s").
    pub fn value_suffix(mut self, suffix: &str) -> Self {
        self.value_suffix = suffix.to_string();
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    /// Render the chart as an HTML string containing an `<svg>` element.
    pub fn render(&self) -> String {
        if self.entries.is_empty() {
            return self.render_empty();
        }

        let max_val = self
            .entries
            .iter()
            .map(|e| e.value)
            .fold(0.0_f64, f64::max)
            .max(f64::MIN_POSITIVE);

        let total_height = self.entries.len() as f64 * self.row_height + 24.0;
        let viewbox_width = self.label_width + self.max_bar_width + 100.0;

        let bars: String = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let y = 24.0 + (i as f64 * self.row_height);
                let width = if max_val > 0.0 {
                    (entry.value / max_val) * self.max_bar_width
                } else {
                    0.0
                }
                .max(2.0);
                let label = truncate_name(&entry.label, self.label_max_chars);

                let value_str = format!("{:.2}{}", entry.value, self.value_suffix);

                format!(
                    r##"<text x="8" y="{y_label}" class="arniko-svgbar-label" fill="var(--arniko-text-muted)">{label}</text>
<rect x="{bar_x}" y="{y_rect}" width="{width}" height="{bar_h}" fill="{color}" rx="2" opacity="0.85"/>
<text x="{x_val}" y="{y_label}" class="arniko-svgbar-value" fill="{color}">{value_str}</text>
"##,
                    y_label = y + 10.0,
                    y_rect = y + 2.0,
                    label = escape_html(&label),
                    bar_x = self.label_width,
                    bar_h = self.bar_height,
                    width = width,
                    color = escape_html(&entry.color),
                    x_val = self.label_width + width + 6.0,
                    value_str = value_str,
                )
            })
            .collect();

        let header = self.render_header();
        let chart_label = self.title.as_deref().unwrap_or("Bar chart");
        format!(
            r#"<div class="arniko-svgbar-chart {}">{header}
    <svg width="100%" height="{height}" viewBox="0 0 {vw} {height}" xmlns="http://www.w3.org/2000/svg" preserveAspectRatio="xMidYMin meet" class="arniko-svgbar-svg" role="img" aria-label="{chart_label}">
        {bars}
    </svg>
</div>
"#,
            escape_html(&self.class),
            header = header,
            height = total_height,
            vw = viewbox_width,
            chart_label = escape_html(chart_label),
            bars = bars,
        )
    }

    fn render_header(&self) -> String {
        match (&self.title, &self.title_icon) {
            (Some(title), Some(icon)) => {
                format!(
                    r#"<div class="arniko-svgbar-header">
                        <span class="arniko-svgbar-icon">{}</span>
                        <span class="arniko-svgbar-title">{}</span>
                    </div>"#,
                    escape_html(icon),
                    escape_html(title)
                )
            }
            (Some(title), None) => {
                format!(
                    r#"<div class="arniko-svgbar-header">
                        <span class="arniko-svgbar-title">{}</span>
                    </div>"#,
                    escape_html(title)
                )
            }
            _ => String::new(),
        }
    }

    fn render_empty(&self) -> String {
        let header = self.render_header();
        format!(
            r#"<div class="arniko-svgbar-chart {}">{header}
    <div class="arniko-svgbar-empty">{}</div>
</div>
"#,
            escape_html(&self.class),
            self.empty_message
        )
    }
}

#[cfg(feature = "components")]
impl Default for SvgBarChart {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "components")]
impl Component for SvgBarChart {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec!["arniko-svgbar-chart".to_string()],
            requires_gpu: false,
            capabilities: vec![],
        }
    }
}

// ── Reactive View ────────────────────────────────────────────────────────────

#[cfg(feature = "reactive")]
use crate::reactive::{ReactiveHtml, Signal, View};

/// Create a reactive SVG bar chart that updates when the entries signal changes.
///
/// Uses the default `SvgBarChart` configuration. For a customized chart, use
/// `svg_bar_chart_reactive_with` instead.
#[cfg(feature = "reactive")]
pub fn svg_bar_chart_reactive(entries_signal: Signal<Vec<SvgBarEntry>>) -> Box<dyn View> {
    let html = entries_signal.derive(move |entries| {
        SvgBarChart {
            entries,
            ..SvgBarChart::default()
        }
        .render()
    });
    Box::new(ReactiveHtml::new(html))
}

/// Create a reactive SVG bar chart with a builder for full configuration.
#[cfg(feature = "reactive")]
pub fn svg_bar_chart_reactive_with<F>(
    entries_signal: Signal<Vec<SvgBarEntry>>,
    build: F,
) -> Box<dyn View>
where
    F: Fn(&mut SvgBarChart) + Send + Sync + 'static,
{
    let html = entries_signal.derive(move |entries| {
        let mut chart = SvgBarChart::new();
        build(&mut chart);
        chart.entries = entries;
        chart.render()
    });
    Box::new(ReactiveHtml::new(html))
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn truncate_name(name: &str, max_len: usize) -> String {
    let chars: Vec<char> = name.chars().collect();
    if chars.len() > max_len {
        let truncated: String = chars.into_iter().take(max_len - 1).collect();
        format!("{}…", truncated)
    } else {
        name.to_string()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "components")]
    #[test]
    fn test_svg_bar_chart_empty() {
        let chart = SvgBarChart::new();
        let html = chart.render();
        assert!(html.contains("No data"));
        assert!(html.contains("arniko-svgbar-chart"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_svg_bar_chart_with_entries() {
        let chart = SvgBarChart::new()
            .add(SvgBarEntry::new("A", 50.0, "var(--arniko-error)"))
            .add(SvgBarEntry::new("B", 30.0, "var(--arniko-cyan)"));
        let html = chart.render();
        assert!(html.contains("<svg"));
        assert!(html.contains("A"));
        assert!(html.contains("B"));
        assert!(html.contains("<rect"));
        assert!(html.contains("50.00"));
        assert!(html.contains("30.00"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_svg_bar_chart_with_title() {
        let chart = SvgBarChart::new()
            .title("Benchmark Performance")
            .title_icon("⚡")
            .add(SvgBarEntry::new("test_bench", 12.3, "var(--arniko-cyan)"));
        let html = chart.render();
        assert!(html.contains("Benchmark Performance"));
        assert!(html.contains("⚡"));
        assert!(html.contains("12.30"));
        assert!(html.contains(r#"role="img""#));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_svg_bar_chart_value_suffix() {
        let chart = SvgBarChart::new().value_suffix("ms").add(SvgBarEntry::new(
            "X",
            5.0,
            "var(--arniko-cyan)",
        ));
        let html = chart.render();
        assert!(html.contains("5.00ms"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_svg_bar_chart_empty_custom_message() {
        let chart = SvgBarChart::new()
            .empty_message("Run benchmarks first!")
            .title("My Chart");
        let html = chart.render();
        assert!(html.contains("Run benchmarks first!"));
        assert!(html.contains("My Chart"));
    }
}
