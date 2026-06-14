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
//! use arniko::{Button, Card, Variant};
//!
//! let html = format!(r#"
//!   <html><body>
//!     {}
//!     {}
//!   </body></html>
//! "#,
//!     Button::new("Click me").variant(Variant::Accent).render(),
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

pub mod css;

// Mustang is now an external crate at crates/platform/rendering/mustang
// Re-export it when gpu feature is enabled
#[cfg(feature = "gpu")]
pub extern crate mustang;

pub mod config;
pub mod theme;

// Re-export main types for convenience
#[cfg(feature = "components")]
pub use components::*;

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
#[cfg(feature = "html")]
pub struct ArnikoHtmlBuilder {
    components: Vec<String>,
    styles: Option<String>,
}

#[cfg(feature = "html")]
impl ArnikoHtmlBuilder {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            styles: None,
        }
    }

    pub fn component<C: Component>(mut self, component: C) -> Self {
        self.components.push(component.render());
        self
    }

    /// Add raw HTML content (convenience method)
    pub fn html_content(mut self, html: &str) -> Self {
        self.components.push(html.to_string());
        self
    }

    pub fn style(mut self, style: &str) -> Self {
        self.styles = Some(style.to_string());
        self
    }

    #[cfg(feature = "launch")]
    pub fn launch(self) {
        bliss::launch_static_html(&self.render());
    }

    pub fn render(self) -> String {
        let styles = self.styles.unwrap_or_default();
        let components = self.components.join("\n");

        format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <style>{}</style>
</head>
<body>
    {}
</body>
</html>
"#,
            styles, components
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
}
