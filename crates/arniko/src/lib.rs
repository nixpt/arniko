//! # Arniko - Unified UI Framework for Exosphere
//!
//! Arniko provides a comprehensive UI framework for Exosphere capsules,
//! combining HTML component generation and GPU-accelerated visual effects.
//!
//! ## Quick Start
//!
//! ### HTML-only Usage (Lightweight)
//!
//! ```rust,no_run
//! use arniko::{Button, ButtonVariant, Card};
//!
//! let html = format!(r#"
//!   <html><body>
//!     {}
//!     {}
//!   </body></html>
//! "#,
//!     Button::new("Click me").variant(ButtonVariant::Accent).render(),
//!     Card::new().title("Status").body("Online").render(),
//! );
//! ```
//!
//! ## Architecture
//!
//! - **Components**: HTML component library with embedded CSS
//! - **Compositor**: Post-processing effects for advanced CSS features
//! - **Mustang**: GPU-accelerated effect compositor
//! - **Theme**: Theme management system (Dark, Frosted, Cyberpunk, Aurora)
//! - **CSS**: CSS normalization for Bliss compatibility
//!
//! ## Feature Flags
//!
//! - `html`: HTML string generation (default)
//! - `components`: UI components (default)
//! - `gpu`: GPU acceleration (via Mustang)
//! - `networking`: Exosphere networking
//! - `full`: All features enabled

// Core modules
#[cfg(feature = "components")]
pub mod components;

#[cfg(any(feature = "components", feature = "bliss_html"))]
pub mod icon_theme;

#[cfg(feature = "reactive")]
pub mod reactive;

pub mod css;

// Mustang is now an external crate at crates/platform/rendering/mustang
// Re-export it when gpu or reactive is enabled (reactive's SceneScheduler hook
// lives in mustang; it is a hard dependency, so this only controls the re-export).
#[cfg(any(feature = "gpu", feature = "reactive"))]
pub extern crate mustang;

pub mod config;
pub mod theme;

// Re-export main types for convenience
#[cfg(feature = "components")]
pub use components::*;

#[cfg(any(feature = "components", feature = "bliss_html"))]
pub use components::icon_theme::*;

pub use css::*;

#[cfg(feature = "gpu")]
pub use mustang::*;

pub use config::*;
pub use theme::*;

/// Main application entry point
pub struct ArnikoApp;

impl ArnikoApp {
    /// Create an HTML-only application
    #[cfg(feature = "html")]
    pub fn html() -> ArnikoHtmlBuilder {
        ArnikoHtmlBuilder::new()
    }
}

/// HTML-only application builder
///
/// Builds a complete HTML document from components, styles, and metadata.
/// Features include automatic Arniko base styles, base CSS reset,
/// and `<title>` support.
#[cfg(feature = "html")]
pub struct ArnikoHtmlBuilder {
    components: Vec<String>,
    styles: Vec<String>,
    title: Option<String>,
    include_arniko: bool,
    include_reset: bool,
    theme_class: Option<String>,
}

#[cfg(feature = "html")]
impl ArnikoHtmlBuilder {
    /// Create a new builder with defaults:
    /// - Arniko base styles included
    /// - Base CSS reset **not** included
    /// - No page title
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            styles: Vec::new(),
            title: None,
            include_arniko: true,
            include_reset: false,
            theme_class: None,
        }
    }

    /// Add a component via the `Component` trait.
    pub fn component<C: Component>(mut self, component: C) -> Self {
        self.components.push(component.render());
        self
    }

    /// Add raw HTML content to the page body.
    /// Accepts both `&str` and `String` (e.g. from `.render()` calls).
    pub fn html_content(mut self, html: impl Into<String>) -> Self {
        self.components.push(html.into());
        self
    }

    /// Append an extra CSS style block. Can be called multiple times;
    /// each call adds another `<style>` block in order.
    pub fn style(mut self, style: &str) -> Self {
        self.styles.push(style.to_string());
        self
    }

    /// Set the page `<title>`. The title is HTML-escaped automatically.
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Whether to include Arniko's base component styles (`ARNIKO_STYLES`).
    /// Default: `true`.
    pub fn include_arniko_styles(mut self, include: bool) -> Self {
        self.include_arniko = include;
        self
    }

    /// Whether to include a CSS reset (`* { box-sizing: border-box; margin: 0; padding: 0; }`).
    /// Default: `false`.
    pub fn base_styles(mut self, include: bool) -> Self {
        self.include_reset = include;
        self
    }

    /// Set the theme mode. Adds a `theme-{name}` class to `<html>`
    /// so the theme's CSS variable overrides take effect.
    ///
    /// Default: no theme class (uses `:root` defaults = dark).
    ///
    /// ```rust,no_run
    /// use arniko::{ArnikoApp, ThemeMode};
    ///
    /// let html = ArnikoApp::html()
    ///     .theme(ThemeMode::Light)
    ///     .render();
    /// ```
    pub fn theme(mut self, mode: ThemeMode) -> Self {
        self.theme_class = Some(mode.html_class().to_string());
        self
    }

    #[cfg(feature = "launch")]
    pub fn launch(self) {
        bliss::launch_static_html(&self.render());
    }

    /// Build the complete HTML document string.
    pub fn render(self) -> String {
        // ── Accumulate all styles ──
        let mut all_styles = String::new();

        // 1. Arniko base component styles (when feature is available)
        #[cfg(feature = "components")]
        if self.include_arniko {
            all_styles.push_str(crate::components::ARNIKO_STYLES);
            all_styles.push('\n');
        }

        // 2. Base CSS reset
        if self.include_reset {
            all_styles.push_str("* { box-sizing: border-box; margin: 0; padding: 0; }\n");
        }

        // 3. User-provided extra styles
        for style in &self.styles {
            all_styles.push_str(style);
            all_styles.push('\n');
        }

        // ── Body content ──
        let body = self.components.join("\n");

        // ── Title (HTML-escaped) ──
        let title_line = match &self.title {
            Some(t) => {
                let escaped = t
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");
                format!("    <title>{}</title>\n", escaped)
            }
            None => String::new(),
        };

        let html_open = match &self.theme_class {
            Some(c) => format!(r#"<html class="{}">"#, c),
            None => "<html>".to_string(),
        };

        format!(
            r#"<!DOCTYPE html>
{html_open}
<head>
    <meta charset="utf-8">
{title}    <style>{styles}</style>
</head>
<body>
    {body}
</body>
</html>
"#,
            html_open = html_open,
            title = title_line,
            styles = all_styles,
            body = body,
        )
    }
}

/// Component trait for unified rendering
#[cfg(feature = "components")]
pub trait Component {
    /// Render as HTML string
    fn render(&self) -> String;

    /// Get component metadata
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::default()
    }
}

/// Component metadata
#[derive(Debug, Clone, Default)]
pub struct ComponentMetadata {
    pub css_classes: Vec<String>,
    pub requires_gpu: bool,
    pub capabilities: Vec<String>,
}

/// Default implementation for HTML-only components
#[cfg(feature = "components")]
impl<T: Component> Component for Box<T> {
    fn render(&self) -> String {
        (**self).render()
    }

    fn metadata(&self) -> ComponentMetadata {
        (**self).metadata()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_metadata() {
        let metadata = ComponentMetadata {
            css_classes: vec!["test".to_string()],
            requires_gpu: false,
            capabilities: vec![],
        };

        assert_eq!(metadata.css_classes.len(), 1);
        assert!(!metadata.requires_gpu);
    }

    #[test]
    fn test_theme_mode_exported() {
        // Verify theme types are exported
        let _theme = ThemeMode::Dark;
        let _manager = ThemeManager::new();
    }

    #[test]
    fn test_css_normalization_exported() {
        // Verify CSS types are exported
        let _result = NormalizationMetadata::default();
    }

    #[cfg(feature = "html")]
    #[test]
    fn test_builder_default_includes_arniko_styles() {
        let html = ArnikoHtmlBuilder::new().render();
        assert!(
            html.contains("arniko-btn"),
            "default build should include arniko styles"
        );
        assert!(
            html.contains("<!DOCTYPE html>"),
            "should produce valid HTML"
        );
    }

    #[cfg(feature = "html")]
    #[test]
    fn test_builder_title() {
        let html = ArnikoHtmlBuilder::new().title("My Page").render();
        assert!(html.contains("<title>My Page</title>"));
    }

    #[cfg(feature = "html")]
    #[test]
    fn test_builder_title_escaped() {
        let html = ArnikoHtmlBuilder::new().title("Foo & Bar <3").render();
        assert!(html.contains("Foo &amp; Bar &lt;3"));
        assert!(!html.contains("<title>Foo & Bar <3"));
    }

    #[cfg(feature = "html")]
    #[test]
    fn test_builder_arniko_styles_can_be_opted_out() {
        let html = ArnikoHtmlBuilder::new()
            .include_arniko_styles(false)
            .render();
        assert!(
            !html.contains("arniko-btn"),
            "arniko styles should be absent when opted out"
        );
    }

    #[cfg(feature = "html")]
    #[test]
    fn test_builder_base_styles() {
        let html = ArnikoHtmlBuilder::new().base_styles(true).render();
        assert!(html.contains("box-sizing: border-box"));
    }

    #[cfg(feature = "html")]
    #[test]
    fn test_builder_html_content() {
        let html = ArnikoHtmlBuilder::new()
            .include_arniko_styles(false)
            .html_content("<p>Hello</p>")
            .render();
        assert!(html.contains("<p>Hello</p>"));
    }

    #[cfg(feature = "html")]
    #[test]
    fn test_builder_custom_style() {
        let html = ArnikoHtmlBuilder::new()
            .include_arniko_styles(false)
            .style("body { background: red; }")
            .render();
        assert!(html.contains("background: red"));
    }

    #[cfg(feature = "html")]
    #[test]
    fn test_builder_multiple_styles() {
        let html = ArnikoHtmlBuilder::new()
            .include_arniko_styles(false)
            .style("body { color: red; }")
            .style("h1 { color: blue; }")
            .render();
        assert!(html.contains("color: red"));
        assert!(html.contains("color: blue"));
    }

    #[cfg(feature = "html")]
    #[test]
    fn test_builder_meta_charset() {
        let html = ArnikoHtmlBuilder::new()
            .include_arniko_styles(false)
            .render();
        assert!(html.contains("<meta charset=\"utf-8\">"));
    }

    #[cfg(feature = "html")]
    #[test]
    fn test_builder_theme_light_adds_class() {
        let html = ArnikoHtmlBuilder::new()
            .include_arniko_styles(false)
            .theme(ThemeMode::Light)
            .render();
        assert!(
            html.contains("<html class=\"theme-light\">"),
            "theme light should add class=\"theme-light\" to html tag"
        );
    }

    #[cfg(feature = "html")]
    #[test]
    fn test_builder_theme_dark_no_class_by_default() {
        let html = ArnikoHtmlBuilder::new()
            .include_arniko_styles(false)
            .render();
        // Default should be a plain <html> tag (no class)
        assert!(
            !html.contains("<html class="),
            "default render should not have a class on html"
        );
        assert!(
            html.contains("<html>"),
            "default render should have plain <html>"
        );
    }

    #[cfg(feature = "html")]
    #[test]
    fn test_builder_theme_class_with_styles() {
        // When including arniko styles, the theme class should still be present
        let html = ArnikoHtmlBuilder::new()
            .theme(ThemeMode::Light)
            .html_content("<p>Hello</p>")
            .render();
        assert!(html.contains("<html class=\"theme-light\">"));
        assert!(html.contains("<p>Hello</p>"));
    }

    #[test]
    #[cfg(feature = "components")]
    fn test_arniko_styles_golden_file() {
        let golden_dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/components/styles/__golden__"
        );
        let golden_path = std::path::Path::new(golden_dir).join("arniko_styles.css");
        let current = crate::components::ARNIKO_STYLES;

        // When UPDATE_EXPECT is set, write the current CSS as the new golden file
        if std::env::var("UPDATE_EXPECT").is_ok() {
            std::fs::write(&golden_path, current).unwrap_or_else(|e| {
                panic!(
                    "failed to write golden file {}: {}",
                    golden_path.display(),
                    e
                )
            });
            eprintln!("🖼️  Updated golden file: {}", golden_path.display());
            return;
        }

        // Otherwise, compare against the golden file
        let golden = std::fs::read_to_string(&golden_path).unwrap_or_else(|e| {
            panic!(
                "Golden file not found at {}. \
                 Run `UPDATE_EXPECT=1 cargo test -p arniko --features components --lib` \
                 to generate it.\nError: {}",
                golden_path.display(),
                e
            )
        });

        if current != golden {
            // Show a diff-like snippet
            let current_lines: Vec<&str> = current.lines().collect();
            let golden_lines: Vec<&str> = golden.lines().collect();
            let min_len = current_lines.len().min(golden_lines.len());
            let mut first_diff = None;
            for i in 0..min_len {
                if current_lines[i] != golden_lines[i] {
                    first_diff = Some(i);
                    break;
                }
            }
            let diff_info = match first_diff {
                Some(line) => format!(
                    "First difference at line {}.\n  golden: {}\n  actual: {}",
                    line + 1,
                    golden_lines[line],
                    current_lines.get(line).unwrap_or(&"<eof>")
                ),
                None => format!(
                    "Length differs: golden={} lines, actual={} lines",
                    golden_lines.len(),
                    current_lines.len()
                ),
            };
            panic!(
                "ARNIKO_STYLES has changed!\n\
                 Run `UPDATE_EXPECT=1 cargo test -p arniko --features components --lib` \
                 to update the golden file after verifying the changes.\n\
                 {}",
                diff_info
            );
        }
    }
}
