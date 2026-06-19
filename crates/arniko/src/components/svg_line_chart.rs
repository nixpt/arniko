//! SvgLineChart — SVG multi-series line chart component.
//!
//! Generalized from the Khukuri VulnTimeline. Renders `<svg>` elements with
//! `<path>` lines, area fills, grid lines, axis labels, data circles, and legend.
//!
//! Supports multiple series, each with its own color and line style.

#[cfg(feature = "components")]
use {
    super::svg_util::{build_area, build_path},
    crate::components::escape_html,
    crate::{Component, ComponentMetadata},
};

/// Line style for a series.
#[derive(Clone, Debug, PartialEq)]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
}

/// A single data series in the line chart.
#[derive(Clone, Debug)]
pub struct LineSeries {
    pub name: String,
    pub color: String,
    pub values: Vec<f64>,
    pub line_style: LineStyle,
    pub show_area: bool,
    pub show_circles: bool,
}

impl LineSeries {
    pub fn new(name: &str, color: &str, values: Vec<f64>) -> Self {
        Self {
            name: name.to_string(),
            color: color.to_string(),
            values,
            line_style: LineStyle::Solid,
            show_area: false,
            show_circles: false,
        }
    }

    pub fn dashed(mut self) -> Self {
        self.line_style = LineStyle::Dashed;
        self
    }

    pub fn dotted(mut self) -> Self {
        self.line_style = LineStyle::Dotted;
        self
    }

    pub fn with_area(mut self) -> Self {
        self.show_area = true;
        self
    }

    pub fn with_circles(mut self) -> Self {
        self.show_circles = true;
        self
    }
}

// ── HTML Component ───────────────────────────────────────────────────────────

/// An SVG multi-series line chart.
///
/// Each series is rendered as a `<path>` line with optional area fill, data point
/// circles, grid lines, axis labels, and legend.
/// # Examples
///
/// ```rust,no_run
/// use arniko::{SvgLineChart, LineSeries, LineStyle};
///
/// let chart = SvgLineChart::new()
///     .title("CPU/Mem")
///     .x_labels(vec!["T1".into(), "T2".into(), "T3".into(), "T4".into()])
///     .add_series(LineSeries::new("CPU", "var(--arniko-cyan)", vec![42.0, 51.0, 47.0, 60.0]).with_area())
///     .add_series(LineSeries::new("Mem", "var(--arniko-accent)", vec![60.0, 65.0, 71.0, 73.0]).dashed());
///
/// let html = chart.render();
/// ```
///
/// With the `reactive` feature:
///
/// ```rust,no_run
/// # #[cfg(feature = "reactive")] {
/// use arniko::{LineSeries, svg_line_chart_reactive, reactive::Signal};
///
/// let series_signal = Signal::new(Vec::<LineSeries>::new());
/// let view = svg_line_chart_reactive(series_signal);
/// # }
/// ```
#[cfg(feature = "components")]
pub struct SvgLineChart {
    series: Vec<LineSeries>,
    x_labels: Vec<String>,
    title: Option<String>,
    title_icon: Option<String>,
    empty_message: String,
    width: f64,
    height: f64,
    padding: f64,
    grid_lines: usize,
    show_legend: bool,
    class: String,
}

#[cfg(feature = "components")]
impl SvgLineChart {
    pub fn new() -> Self {
        Self {
            series: Vec::new(),
            x_labels: Vec::new(),
            title: None,
            title_icon: None,
            empty_message: "No data".to_string(),
            width: 560.0,
            height: 200.0,
            padding: 40.0,
            grid_lines: 4,
            show_legend: true,
            class: String::new(),
        }
    }

    pub fn add_series(mut self, s: LineSeries) -> Self {
        self.series.push(s);
        self
    }

    pub fn x_labels(mut self, labels: Vec<String>) -> Self {
        self.x_labels = labels;
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn title_icon(mut self, icon: &str) -> Self {
        self.title_icon = Some(icon.to_string());
        self
    }

    pub fn empty_message(mut self, msg: &str) -> Self {
        self.empty_message = msg.to_string();
        self
    }

    pub fn width(mut self, w: f64) -> Self {
        self.width = w;
        self
    }

    pub fn height(mut self, h: f64) -> Self {
        self.height = h;
        self
    }

    pub fn padding(mut self, p: f64) -> Self {
        self.padding = p;
        self
    }

    pub fn grid_lines(mut self, n: usize) -> Self {
        self.grid_lines = n.max(1);
        self
    }

    pub fn legend(mut self, show: bool) -> Self {
        self.show_legend = show;
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        if self.series.is_empty() || self.series.iter().all(|s| s.values.is_empty()) {
            return self.render_empty();
        }

        let graph_w = self.width - self.padding * 2.0;
        let graph_h = self.height - self.padding * 2.0;
        let bottom_y = self.padding + graph_h;

        // Find global max across all series
        let max_y = self
            .series
            .iter()
            .flat_map(|s| s.values.iter())
            .copied()
            .fold(0.0_f64, f64::max)
            .max(1.0);

        // Use the longest series to determine x-step
        let n = self
            .series
            .iter()
            .map(|s| s.values.len())
            .max()
            .unwrap_or(1);
        let step_x = if n > 1 {
            graph_w / (n - 1) as f64
        } else {
            graph_w
        };

        // ── Grid lines ──
        let mut grid_svg = String::new();
        for i in 0..=self.grid_lines {
            let y = self.padding + graph_h - (i as f64 / self.grid_lines as f64) * graph_h;
            grid_svg.push_str(&format!(
                r#"<line x1="{p}" y1="{y:.1}" x2="{x2}" y2="{y:.1}" stroke="var(--arniko-border-light)" stroke-width="1"/>"#,
                p = self.padding,
                y = y,
                x2 = self.width - self.padding,
            ));
        }

        // ── Y-axis labels ──
        let mut y_labels_svg = String::new();
        for i in 0..=self.grid_lines {
            let y = self.padding + graph_h - (i as f64 / self.grid_lines as f64) * graph_h;
            let val = (i as f64 / self.grid_lines as f64 * max_y).round() as usize;
            y_labels_svg.push_str(&format!(
                r##"<text x="{x:.1}" y="{y:.1}" text-anchor="end" font-size="9" fill="var(--arniko-text-muted)" font-family="JetBrains Mono,monospace" dominant-baseline="middle">{val}</text>"##,
                x = self.padding - 6.0,
                y = y + 3.0,
                val = val,
            ));
        }

        // ── X-axis labels (skip some if too dense) ──
        let mut x_labels_svg = String::new();
        let label_step = (n / 6).max(1);
        for i in 0..n {
            if i % label_step == 0 || i == n - 1 {
                let x = self.padding + i as f64 * step_x;
                let label = self.x_labels.get(i).map(|s| s.as_str()).unwrap_or("");
                x_labels_svg.push_str(&format!(
                    r##"<text x="{x:.1}" y="{y:.1}" text-anchor="middle" font-size="9" fill="var(--arniko-text-muted)" font-family="JetBrains Mono,monospace">{label}</text>"##,
                    x = x,
                    y = self.height - 10.0,
                    label = escape_html(label),
                ));
            }
        }

        // ── Series: lines, areas, circles ──
        let mut series_svg = String::new();
        let mut legend_svg = String::new();

        for (si, s) in self.series.iter().enumerate() {
            if s.values.is_empty() {
                continue;
            }

            let points: Vec<(f64, f64)> = s
                .values
                .iter()
                .enumerate()
                .map(|(i, &v)| {
                    let x = self.padding + i as f64 * step_x;
                    let y = self.padding + graph_h - (v / max_y) * graph_h;
                    (x, y)
                })
                .collect();

            let stroke_dash = match s.line_style {
                LineStyle::Solid => String::new(),
                LineStyle::Dashed => " stroke-dasharray=\"4,3\"".to_string(),
                LineStyle::Dotted => " stroke-dasharray=\"2,3\"".to_string(),
            };

            // Area fill
            if s.show_area {
                let area_path = build_area(&points, bottom_y);
                series_svg.push_str(&format!(
                    r#"<path d="{path}" fill="{color}" opacity="0.06" stroke="none"/>"#,
                    path = area_path,
                    color = s.color,
                ));
            }

            // Line path
            let line_path = build_path(&points);
            series_svg.push_str(&format!(
                r#"<path d="{path}" fill="none" stroke="{color}" stroke-width="2" stroke-linejoin="round" opacity="0.85{stroke_dash}"/>"#,
                path = line_path,
                color = s.color,
                stroke_dash = stroke_dash,
            ));

            // Data point circles
            if s.show_circles {
                let circle_radius = if s.line_style == LineStyle::Solid {
                    "3"
                } else {
                    "2"
                };
                let circle_opacity = if s.line_style == LineStyle::Solid {
                    "0.8"
                } else {
                    "0.6"
                };
                for &(cx, cy) in &points {
                    series_svg.push_str(&format!(
                        r##"<circle cx="{cx:.1}" cy="{cy:.1}" r="{r}" fill="{color}" opacity="{op}"/>"##,
                        cx = cx,
                        cy = cy,
                        r = circle_radius,
                        color = s.color,
                        op = circle_opacity,
                    ));
                }
            }

            // Legend entry
            if self.show_legend {
                let lx = (si % 3) as f64 * 85.0;
                let ly = (si / 3) as f64 * 14.0;
                legend_svg.push_str(&format!(
                    r##"<rect x="{x:.1}" y="{y:.1}" width="10" height="3" fill="{color}" rx="1"/>
                        <text x="{tx:.1}" y="{ty:.1}" font-size="10" fill="var(--arniko-text-secondary)" font-family="JetBrains Mono,monospace">{name}</text>"##,
                    x = lx,
                    y = ly,                        color = s.color,
                        tx = lx + 16.0,
                        ty = ly + 4.0,
                        name = escape_html(&s.name),
                ));
            }
        }

        let legend_group = if self.show_legend && !legend_svg.is_empty() {
            format!(
                r##"<g transform="translate(40, 12)">\n            {legend}\n        </g>"##,
                legend = legend_svg,
            )
        } else {
            String::new()
        };

        let header = self.render_header();
        let chart_label = self.title.as_deref().unwrap_or("Line chart");
        format!(
            r##"<div class="arniko-linechart {}">{header}
    <svg width="100%" height="{height}" viewBox="0 0 {width} {height}" xmlns="http://www.w3.org/2000/svg" preserveAspectRatio="xMidYMin meet" class="arniko-linechart-svg" role="img" aria-label="{chart_label}">
        {grid}
        {y_labels}
        {x_labels}
        {series}
        {legend}
    </svg>
</div>
"##,
            escape_html(&self.class),
            header = header,
            height = self.height,
            width = self.width,
            chart_label = escape_html(chart_label),
            grid = grid_svg,
            y_labels = y_labels_svg,
            x_labels = x_labels_svg,
            series = series_svg,
            legend = legend_group,
        )
    }

    fn render_header(&self) -> String {
        match (&self.title, &self.title_icon) {
            (Some(title), Some(icon)) => {
                format!(
                    r#"<div class="arniko-linechart-header">
                        <span class="arniko-linechart-icon">{}</span>
                        <span class="arniko-linechart-title">{}</span>
                    </div>"#,
                    escape_html(icon),
                    escape_html(title)
                )
            }
            (Some(title), None) => {
                format!(
                    r#"<div class="arniko-linechart-header">
                        <span class="arniko-linechart-title">{}</span>
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
            r#"<div class="arniko-linechart {}">{header}
    <div class="arniko-linechart-empty">{}</div>
</div>
"#,
            escape_html(&self.class),
            escape_html(&self.empty_message)
        )
    }
}

#[cfg(feature = "components")]
impl Default for SvgLineChart {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "components")]
impl Component for SvgLineChart {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec!["arniko-linechart".to_string()],
            requires_gpu: false,
            capabilities: vec![],
        }
    }
}

// ── Reactive View ────────────────────────────────────────────────────────────

#[cfg(feature = "reactive")]
use crate::reactive::{ReactiveHtml, Signal, View};

/// Create a reactive SVG line chart from a signal of series.
/// Uses the default `SvgLineChart` configuration. X-labels are not bound (empty).
#[cfg(feature = "reactive")]
pub fn svg_line_chart_reactive(series_signal: Signal<Vec<LineSeries>>) -> Box<dyn View> {
    let html = series_signal.derive(move |series| {
        SvgLineChart {
            series,
            ..SvgLineChart::default()
        }
        .render()
    });
    Box::new(ReactiveHtml::new(html))
}

/// Create a reactive SVG line chart with a builder for full configuration.
#[cfg(feature = "reactive")]
pub fn svg_line_chart_reactive_with<F>(
    series_signal: Signal<Vec<LineSeries>>,
    build: F,
) -> Box<dyn View>
where
    F: Fn(&mut SvgLineChart) + Send + Sync + 'static,
{
    let html = series_signal.derive(move |series| {
        let mut chart = SvgLineChart::new();
        build(&mut chart);
        chart.series = series;
        chart.render()
    });
    Box::new(ReactiveHtml::new(html))
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "components")]
    #[test]
    fn test_svg_line_chart_empty() {
        let chart = SvgLineChart::new();
        let html = chart.render();
        assert!(html.contains("No data"));
        assert!(html.contains("arniko-linechart"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_svg_line_chart_single_series() {
        let chart = SvgLineChart::new().add_series(LineSeries::new(
            "CPU",
            "#00f2ff",
            vec![10.0, 20.0, 15.0],
        ));
        let html = chart.render();
        assert!(html.contains("<svg"));
        assert!(html.contains("<path"));
        assert!(html.contains("CPU"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_svg_line_chart_multi_series() {
        let chart = SvgLineChart::new()
            .add_series(
                LineSeries::new("Total", "#00f2ff", vec![5.0, 8.0, 6.0])
                    .with_area()
                    .with_circles(),
            )
            .add_series(
                LineSeries::new("Critical", "#ef4444", vec![1.0, 2.0, 1.0])
                    .dashed()
                    .with_area(),
            );
        let html = chart.render();
        assert!(html.contains("Total"));
        assert!(html.contains("Critical"));
        assert!(html.contains("<circle"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_svg_line_chart_with_title() {
        let chart = SvgLineChart::new()
            .title("Trend")
            .title_icon("📈")
            .add_series(LineSeries::new("X", "#00f2ff", vec![1.0, 2.0]));
        let html = chart.render();
        assert!(html.contains("Trend"));
        assert!(html.contains("📈"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_svg_line_chart_no_legend() {
        let chart = SvgLineChart::new()
            .legend(false)
            .add_series(LineSeries::new("X", "#00f2ff", vec![1.0]));
        let html = chart.render();
        assert!(!html.contains("<g transform"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_svg_line_chart_empty_series_values() {
        let chart = SvgLineChart::new().add_series(LineSeries::new("Empty", "#00f2ff", vec![]));
        let html = chart.render();
        assert!(html.contains("No data"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_svg_line_chart_x_labels() {
        let chart = SvgLineChart::new()
            .x_labels(vec!["A".to_string(), "B".to_string(), "C".to_string()])
            .add_series(LineSeries::new("X", "#00f2ff", vec![1.0, 2.0, 3.0]));
        let html = chart.render();
        assert!(html.contains("A"));
        assert!(html.contains("B"));
        assert!(html.contains("C"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_svg_line_chart_grid_lines() {
        let chart = SvgLineChart::new()
            .grid_lines(4)
            .add_series(LineSeries::new("X", "#00f2ff", vec![1.0, 2.0]));
        let html = chart.render();
        // Should have 5 grid lines (0..=4)
        let line_count = html.matches("<line x1=").count();
        assert!(
            line_count >= 5,
            "Expected at least 5 grid lines, got {}",
            line_count
        );
    }
}
