//! SplashScreen — Loading/boot screen with progress bar and status messages.
//!
//! Generalized from the Khukuri BootScreen. Provides a full-screen overlay
//! with a progress bar, status text, and optional branding elements.

#[cfg(feature = "components")]
use crate::components::escape_html;
#[cfg(feature = "components")]
use crate::{Component, ComponentMetadata};

/// Configuration for the splash screen's appearance.
#[derive(Clone, Debug)]
pub struct SplashConfig {
    pub title: String,
    pub subtitle: String,
    pub progress: u8, // 0-100
    pub status: String,
    pub logo_svg: Option<String>, // Optional inline SVG for the logo
}

impl Default for SplashConfig {
    fn default() -> Self {
        Self {
            title: "Loading".to_string(),
            subtitle: String::new(),
            progress: 0,
            status: "Initializing...".to_string(),
            logo_svg: None,
        }
    }
}

// ── HTML Component ───────────────────────────────────────────────────────────

/// A full-screen splash/loading screen with a progress bar.
#[cfg(feature = "components")]
pub struct SplashScreen {
    config: SplashConfig,
    opacity: f64,
    display: bool,
    class: String,
}

#[cfg(feature = "components")]
impl SplashScreen {
    /// Create a splash screen with the given config.
    pub fn new(config: SplashConfig) -> Self {
        Self {
            config,
            opacity: 1.0,
            display: true,
            class: String::new(),
        }
    }

    /// Set the opacity (0.0 = hidden, 1.0 = fully visible).
    pub fn opacity(mut self, o: f64) -> Self {
        self.opacity = o.clamp(0.0, 1.0);
        self
    }

    /// Set whether the splash is displayed.
    pub fn display(mut self, show: bool) -> Self {
        self.display = show;
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        let display_val = if self.display { "flex" } else { "none" };
        let logo_html = self
            .config
            .logo_svg
            .as_ref()
            .map(|svg| {
                format!(
                    r#"<div class="arniko-splash-logo-box">
                    {}
                    <div class="arniko-splash-pulse-ring"></div>
                </div>"#,
                    svg
                )
            })
            .unwrap_or_default();

        let subtitle_html = if self.config.subtitle.is_empty() {
            String::new()
        } else {
            format!(
                r#"<div class="arniko-splash-subtitle">{}</div>"#,
                escape_html(&self.config.subtitle)
            )
        };

        format!(
            r#"<div class="arniko-splash-overlay {}" style="opacity:{};display:{};" role="progressbar" aria-valuenow="{}" aria-valuemin="0" aria-valuemax="100" aria-label="{}">
                <div class="arniko-splash-container">
                    {}
                    <div class="arniko-splash-title">{}</div>
                    {}
                    <div class="arniko-splash-progress-wrapper">
                        <div class="arniko-splash-progress-track">
                            <div class="arniko-splash-progress-fill" style="width:{}%;"></div>
                        </div>
                        <div class="arniko-splash-progress-pct">{}%</div>
                    </div>
                    <div class="arniko-splash-status" aria-live="polite">{}</div>
                </div>
            </div>"#,
            escape_html(&self.class),
            self.opacity,
            display_val,
            self.config.progress,
            escape_html(&self.config.title),
            logo_html,
            escape_html(&self.config.title),
            subtitle_html,
            self.config.progress,
            self.config.progress,
            escape_html(&self.config.status)
        )
    }
}

#[cfg(feature = "components")]
impl Component for SplashScreen {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec!["arniko-splash-overlay".to_string()],
            requires_gpu: false,
            capabilities: vec![],
        }
    }
}

// ── Reactive View ────────────────────────────────────────────────────────────

#[cfg(feature = "reactive")]
use crate::reactive::{ReactiveHtml, Signal, View};

/// Create a reactive splash screen that updates when the config signal changes.
#[cfg(feature = "reactive")]
pub fn splash_screen_reactive(config_signal: &Signal<SplashConfig>) -> Box<dyn View> {
    let html = config_signal.derive(|config| SplashScreen::new(config.clone()).render());
    Box::new(ReactiveHtml::new(html))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "components")]
    #[test]
    fn test_splash_screen_defaults() {
        let config = SplashConfig::default();
        let screen = SplashScreen::new(config);
        let html = screen.render();
        assert!(html.contains("arniko-splash-overlay"));
        assert!(html.contains("Loading"));
        assert!(html.contains("Initializing..."));
        assert!(html.contains(r#"role="progressbar""#));
        assert!(html.contains(r#"aria-live="polite""#));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_splash_screen_hidden() {
        let screen = SplashScreen::new(SplashConfig::default()).display(false);
        let html = screen.render();
        assert!(html.contains("display:none"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_splash_screen_progress() {
        let config = SplashConfig {
            progress: 42,
            ..SplashConfig::default()
        };
        let screen = SplashScreen::new(config);
        let html = screen.render();
        assert!(html.contains("42%"));
        assert!(html.contains("width:42%"));
    }
}
