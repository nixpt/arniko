//! Sparkline — Inline SVG sparkline mini-chart component.
//!
//! Generalized from the Khukuri ResourceMonitor `build_sparkline` helper.
//! Renders a small inline `<svg>` with an area-fill and line path scaled to fit.

#[cfg(feature = "components")]
use {
    crate::{Component, ComponentMetadata},
    super::svg_util::{build_path, build_area},
};

// ── HTML Component ───────────────────────────────────────────────────────────

/// A mini SVG sparkline chart (inline, no axes or labels).
///
/// Renders a single polyline with area fill, scaled to fit the configured
/// dimensions. Suitable for embedding inside metric cards or dashboards.
#[cfg(feature = "components")]
pub struct Sparkline {
    values: Vec<f64>,
    color: String,
    width: f64,
    height: f64,
    padding: f64,
    class: String,
}

#[cfg(feature = "components")]
impl Sparkline {
    pub fn new(values: Vec<f64>) -> Self {
        Self {
            values,
            color: "#00f2ff".to_string(),
            width: 240.0,
            height: 40.0,
            padding: 4.0,
            class: String::new(),
        }
    }

    pub fn color(mut self, color: &str) -> Self {
        self.color = color.to_string();
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

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        if self.values.is_empty() {
            return format!(
                r#"<svg width="100%" height="{h}" viewBox="0 0 {w} {h}" xmlns="http://www.w3.org/2000/svg" class="arniko-sparkline {class}"></svg>"#,
                h = self.height as usize,
                w = self.width as usize,
                class = self.class,
            );
        }

        let step_x = if self.values.len() > 1 {
            (self.width - self.padding * 2.0) / (self.values.len() - 1) as f64
        } else {
            self.width
        };
        let max_val = self.values.iter().copied().fold(0.0_f64, f64::max).max(1.0);
        let graph_h = self.height - self.padding * 2.0;

        let points: Vec<(f64, f64)> = self
            .values
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let x = self.padding + i as f64 * step_x;
                let y = self.padding + graph_h - (v / max_val) * graph_h;
                (x, y)
            })
            .collect();

        let line_path = build_path(&points);
        let area_path = build_area(&points, self.height - self.padding);

        format!(
            r#"<svg width="100%" height="{h}" viewBox="0 0 {w} {h}" xmlns="http://www.w3.org/2000/svg" class="arniko-sparkline {class}">
                <path d="{area}" fill="{color}" opacity="0.15" stroke="none"/>
                <path d="{line}" fill="none" stroke="{color}" stroke-width="1.5" stroke-linejoin="round" opacity="0.85"/>
            </svg>"#,
            h = self.height as usize,
            w = self.width as usize,
            class = self.class,
            area = area_path,
            line = line_path,
            color = self.color,
        )
    }
}

#[cfg(feature = "components")]
impl Default for Sparkline {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[cfg(feature = "components")]
impl Component for Sparkline {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec!["arniko-sparkline".to_string()],
            requires_gpu: false,
            capabilities: vec![],
        }
    }
}

// ── Reactive View ────────────────────────────────────────────────────────────

#[cfg(feature = "reactive")]
use crate::reactive::{ReactiveHtml, Signal, View};

/// Create a reactive sparkline that updates when the data signal changes.
#[cfg(feature = "reactive")]
pub fn sparkline_reactive(values_signal: Signal<Vec<f64>>) -> Box<dyn View> {
    let html = values_signal.derive(move |values| {
        Sparkline {
            values,
            ..Sparkline::default()
        }
        .render()
    });
    Box::new(ReactiveHtml::new(html))
}

/// Create a reactive sparkline with a builder for full configuration.
#[cfg(feature = "reactive")]
pub fn sparkline_reactive_with<F>(
    values_signal: Signal<Vec<f64>>,
    build: F,
) -> Box<dyn View>
where
    F: Fn(&mut Sparkline) + Send + Sync + 'static,
{
    let html = values_signal.derive(move |values| {
        let mut chart = Sparkline::new(values);
        build(&mut chart);
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
    fn test_sparkline_empty() {
        let chart = Sparkline::new(vec![]);
        let html = chart.render();
        assert!(html.contains("<svg"));
        assert!(html.contains("arniko-sparkline"));
        assert!(!html.contains("<path"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_sparkline_single_point() {
        let chart = Sparkline::new(vec![50.0]);
        let html = chart.render();
        assert!(html.contains("<path"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_sparkline_multiple_points() {
        let chart = Sparkline::new(vec![10.0, 20.0, 15.0, 30.0, 25.0]);
        let html = chart.render();
        assert!(html.contains("<svg"));
        assert!(html.contains("<path"));
        // Should have area fill + line = 2 paths
        let path_count = html.matches("<path").count();
        assert_eq!(path_count, 2);
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_sparkline_custom_color() {
        let chart = Sparkline::new(vec![1.0, 2.0]).color("#ef4444");
        let html = chart.render();
        assert!(html.contains("#ef4444"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_sparkline_custom_dimensions() {
        let chart = Sparkline::new(vec![1.0, 2.0]).width(100.0).height(30.0);
        let html = chart.render();
        assert!(html.contains("100"));
        assert!(html.contains("30"));
    }
}
