//! ProgressRing — SVG circular progress ring component.
//!
//! Generalized from the Khukuri HealthRing. Renders an SVG donut chart
//! with a configurable value, color thresholds, and label.

#[cfg(feature = "components")]
use crate::{Component, ComponentMetadata};

/// Color scheme for the ring based on value thresholds.
#[derive(Clone, Debug)]
pub struct RingThreshold {
    pub up_to: f64,
    pub color: &'static str,
    pub label: &'static str,
}

/// Predefined thresholds: red < 50, yellow < 80, green ≥ 80.
pub const DEFAULT_RING_THRESHOLDS: &[RingThreshold] = &[
    RingThreshold { up_to: 50.0, color: "#ef4444", label: "CRITICAL" },
    RingThreshold { up_to: 80.0, color: "#f59e0b", label: "WARNING" },
    RingThreshold { up_to: 100.0, color: "#10b981", label: "SECURE" },
];

// ── HTML Component ───────────────────────────────────────────────────────────

/// An SVG circular progress ring.
#[cfg(feature = "components")]
pub struct ProgressRing {
    value: f64,
    max: f64,
    radius: f64,
    stroke_width: f64,
    thresholds: Vec<RingThreshold>,
    show_percentage: bool,
    size_px: u32,
    class: String,
}

#[cfg(feature = "components")]
impl ProgressRing {
    /// Create a new ring with a value between 0 and max.
    pub fn new(value: f64) -> Self {
        Self {
            value,
            max: 100.0,
            radius: 45.0,
            stroke_width: 10.0,
            thresholds: DEFAULT_RING_THRESHOLDS.to_vec(),
            show_percentage: true,
            size_px: 120,
            class: String::new(),
        }
    }

    /// Set the maximum value (default 100).
    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }

    /// Set the ring radius in SVG user units (default 45).
    pub fn radius(mut self, r: f64) -> Self {
        self.radius = r;
        self
    }

    /// Set the stroke width (default 10).
    pub fn stroke_width(mut self, w: f64) -> Self {
        self.stroke_width = w;
        self
    }

    /// Override the color thresholds.
    pub fn thresholds(mut self, t: Vec<RingThreshold>) -> Self {
        self.thresholds = t;
        self
    }

    /// Show the percentage text in the center (default true).
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Set the SVG element size in CSS pixels (default 120).
    pub fn size_px(mut self, size: u32) -> Self {
        self.size_px = size;
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    /// Get the current color and label based on thresholds.
    fn threshold_info(&self) -> (&'static str, &'static str) {
        let pct = (self.value / self.max * 100.0).clamp(0.0, 100.0);
        for t in &self.thresholds {
            if pct <= t.up_to {
                return (t.color, t.label);
            }
        }
        // Fallback
        ("#10b981", "OK")
    }

    pub fn render(&self) -> String {
        let pct = (self.value / self.max * 100.0).clamp(0.0, 100.0);
        let circumference = 2.0 * std::f64::consts::PI * self.radius;
        let offset = circumference - (pct / 100.0) * circumference;
        let (color, label) = self.threshold_info();
        let viewbox_size = (self.radius + self.stroke_width) * 2.0 + 4.0;
        let center = viewbox_size / 2.0;

        let percentage_text = if self.show_percentage {
            format!(
                r#"<div class="arniko-ring-text">
                    <span class="arniko-ring-percent">{:.0}%</span>
                    <span class="arniko-ring-label">{}</span>
                </div>"#,
                pct, label
            )
        } else {
            format!(
                r#"<div class="arniko-ring-text">
                    <span class="arniko-ring-value">{:.0}/{:.0}</span>
                    <span class="arniko-ring-label">{}</span>
                </div>"#,
                self.value, self.max, label
            )
        };

        format!(
            r#"<div class="arniko-progress-ring {cls}" style="width:{w}px;height:{h}px;">
                <svg class="arniko-ring-svg" viewBox="0 0 {v} {v}" xmlns="http://www.w3.org/2000/svg">
                    <circle class="arniko-ring-bg" cx="{c}" cy="{c}" r="{r}"
                        fill="none" stroke="rgba(255,255,255,0.06)" stroke-width="{sw}" />
                    <circle class="arniko-ring-value" cx="{c}" cy="{c}" r="{r}"
                        fill="none" stroke="{color}" stroke-width="{sw}"
                        stroke-linecap="round"
                        stroke-dasharray="{circ} {circ}"
                        stroke-dashoffset="{off}"
                        style="transition: stroke-dashoffset 0.8s ease-out;" />
                </svg>
                {pct_text}
            </div>"#,
            cls = self.class, w = self.size_px, h = self.size_px,
            v = viewbox_size, c = center, r = self.radius,
            color = color, sw = self.stroke_width,
            circ = circumference, off = offset,
            pct_text = percentage_text
        )
    }
}

#[cfg(feature = "components")]
impl Component for ProgressRing {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec!["arniko-progress-ring".to_string()],
            requires_gpu: false,
            capabilities: vec![],
        }
    }
}

// ── Reactive View ────────────────────────────────────────────────────────────

#[cfg(feature = "reactive")]
use crate::reactive::{Signal, View, ReactiveHtml};

/// Create a reactive progress ring that updates when the value signal changes.
#[cfg(feature = "reactive")]
pub fn progress_ring_reactive(
    value_signal: Signal<f64>,
    thresholds: Vec<RingThreshold>,
) -> Box<dyn View> {
    let ring_html = value_signal.derive(move |value| {
        ProgressRing::new(value)
            .thresholds(thresholds.clone())
            .render()
    });
    Box::new(ReactiveHtml::new(ring_html))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "components")]
    #[test]
    fn test_progress_ring_render() {
        let ring = ProgressRing::new(75.0);
        let html = ring.render();
        assert!(html.contains("arniko-progress-ring"));
        assert!(html.contains("75%"));
        assert!(html.contains("WARNING"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_progress_ring_thresholds() {
        let critical = ProgressRing::new(30.0);
        assert!(critical.render().contains("CRITICAL"));

        let warning = ProgressRing::new(60.0);
        assert!(warning.render().contains("WARNING"));

        let secure = ProgressRing::new(90.0);
        assert!(secure.render().contains("SECURE"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_progress_ring_no_percentage() {
        let ring = ProgressRing::new(5.0).max(10.0).show_percentage(false);
        let html = ring.render();
        assert!(html.contains("5/10"));
        assert!(!html.contains("%"));
    }
}
