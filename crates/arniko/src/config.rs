//! Configuration for Arniko applications
//!
//! Provides unified configuration for different rendering modes
//! and application types.

use serde::{Deserialize, Serialize};

/// Main configuration for Arniko applications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArnikoConfig {
    /// Application title
    pub title: String,

    /// Window dimensions (width, height)
    pub size: (u32, u32),

    /// Rendering mode
    pub renderer: RendererMode,

    /// Enable GPU effects
    pub gpu_effects: bool,

    /// Enable networking
    pub networking: bool,

    /// Theme configuration
    pub theme: ThemeConfig,

    /// Window configuration
    pub window: WindowConfig,
}

/// Rendering mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RendererMode {
    /// HTML string generation only
    Html,
    /// Native window with HTML content
    Native,
    /// Hybrid mode with GPU effects
    Hybrid,
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Theme name
    pub name: String,

    /// Custom CSS overrides
    pub custom_css: Option<String>,

    /// Enable animations
    pub animations: bool,

    /// Color scheme
    pub color_scheme: ColorScheme,
}

/// Color scheme selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorScheme {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// Auto (follow system)
    Auto,
}

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Resizable window
    pub resizable: bool,

    /// Decorated window (with title bar)
    pub decorated: bool,

    /// Always on top
    pub always_on_top: bool,

    /// Start maximized
    pub maximized: bool,

    /// Start in fullscreen
    pub fullscreen: bool,

    /// Minimum window size
    pub min_size: Option<(u32, u32)>,

    /// Maximum window size
    pub max_size: Option<(u32, u32)>,
}

impl Default for ArnikoConfig {
    fn default() -> Self {
        Self::new("Arniko App")
    }
}

impl ArnikoConfig {
    /// Create a new configuration with default settings
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            size: (800, 600),
            renderer: RendererMode::Native,
            gpu_effects: true,
            networking: false,
            theme: ThemeConfig::default(),
            window: WindowConfig::default(),
        }
    }

    /// Set window size
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.size = (width, height);
        self
    }

    /// Set rendering mode
    pub fn with_renderer(mut self, renderer: RendererMode) -> Self {
        self.renderer = renderer;
        self
    }

    /// Enable/disable GPU effects
    pub fn with_gpu_effects(mut self, enabled: bool) -> Self {
        self.gpu_effects = enabled;
        self
    }

    /// Enable/disable networking
    pub fn with_networking(mut self, enabled: bool) -> Self {
        self.networking = enabled;
        self
    }

    /// Set theme configuration
    pub fn with_theme(mut self, theme: ThemeConfig) -> Self {
        self.theme = theme;
        self
    }

    /// Set window configuration
    pub fn with_window(mut self, window: WindowConfig) -> Self {
        self.window = window;
        self
    }

    /// Create HTML-only configuration
    pub fn html_only(title: &str) -> Self {
        Self {
            title: title.to_string(),
            size: (800, 600),
            renderer: RendererMode::Html,
            gpu_effects: false,
            networking: false,
            theme: ThemeConfig::default(),
            window: WindowConfig::default(),
        }
    }

    /// Create native window configuration
    pub fn native_window(title: &str) -> Self {
        Self {
            title: title.to_string(),
            size: (1024, 768),
            renderer: RendererMode::Native,
            gpu_effects: true,
            networking: true,
            theme: ThemeConfig::dark(),
            window: WindowConfig::default(),
        }
    }

    /// Create hybrid configuration with GPU effects
    pub fn hybrid(title: &str) -> Self {
        Self {
            title: title.to_string(),
            size: (1280, 800),
            renderer: RendererMode::Hybrid,
            gpu_effects: true,
            networking: true,
            theme: ThemeConfig::dark(),
            window: WindowConfig::default(),
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self::dark()
    }
}

impl ThemeConfig {
    /// Create dark theme configuration
    pub fn dark() -> Self {
        Self {
            name: "dark".to_string(),
            custom_css: None,
            animations: true,
            color_scheme: ColorScheme::Dark,
        }
    }

    /// Create light theme configuration
    pub fn light() -> Self {
        Self {
            name: "light".to_string(),
            custom_css: None,
            animations: true,
            color_scheme: ColorScheme::Light,
        }
    }

    /// Create auto theme configuration
    pub fn auto() -> Self {
        Self {
            name: "auto".to_string(),
            custom_css: None,
            animations: true,
            color_scheme: ColorScheme::Auto,
        }
    }

    /// Add custom CSS
    pub fn with_custom_css(mut self, css: &str) -> Self {
        self.custom_css = Some(css.to_string());
        self
    }

    /// Enable/disable animations
    pub fn with_animations(mut self, enabled: bool) -> Self {
        self.animations = enabled;
        self
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            resizable: true,
            decorated: true,
            always_on_top: false,
            maximized: false,
            fullscreen: false,
            min_size: None,
            max_size: None,
        }
    }
}

impl WindowConfig {
    /// Set resizable
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Set decorated
    pub fn decorated(mut self, decorated: bool) -> Self {
        self.decorated = decorated;
        self
    }

    /// Set always on top
    pub fn always_on_top(mut self, always_on_top: bool) -> Self {
        self.always_on_top = always_on_top;
        self
    }

    /// Set maximized
    pub fn maximized(mut self, maximized: bool) -> Self {
        self.maximized = maximized;
        self
    }

    /// Set fullscreen
    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    /// Set minimum size
    pub fn min_size(mut self, width: u32, height: u32) -> Self {
        self.min_size = Some((width, height));
        self
    }

    /// Set maximum size
    pub fn max_size(mut self, width: u32, height: u32) -> Self {
        self.max_size = Some((width, height));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arniko_config_default() {
        let config = ArnikoConfig::default();
        assert_eq!(config.title, "Arniko App");
        assert_eq!(config.size, (800, 600));
        assert_eq!(config.renderer, RendererMode::Native);
        assert!(config.gpu_effects);
        assert!(!config.networking);
    }

    #[test]
    fn test_arniko_config_builder() {
        let config = ArnikoConfig::new("Test App")
            .with_size(1024, 768)
            .with_renderer(RendererMode::Html)
            .with_gpu_effects(false)
            .with_networking(true);

        assert_eq!(config.title, "Test App");
        assert_eq!(config.size, (1024, 768));
        assert_eq!(config.renderer, RendererMode::Html);
        assert!(!config.gpu_effects);
        assert!(config.networking);
    }

    #[test]
    fn test_theme_config() {
        let theme = ThemeConfig::dark()
            .with_custom_css("body { margin: 0; }")
            .with_animations(false);

        assert_eq!(theme.name, "dark");
        assert_eq!(theme.color_scheme, ColorScheme::Dark);
        assert!(theme.custom_css.is_some());
        assert!(!theme.animations);
    }

    #[test]
    fn test_window_config() {
        let window = WindowConfig::default()
            .resizable(false)
            .maximized(true)
            .min_size(400, 300);

        assert!(!window.resizable);
        assert!(window.maximized);
        assert_eq!(window.min_size, Some((400, 300)));
    }

    #[test]
    fn test_preset_configurations() {
        let html_config = ArnikoConfig::html_only("HTML App");
        assert_eq!(html_config.renderer, RendererMode::Html);
        assert!(!html_config.gpu_effects);

        let native_config = ArnikoConfig::native_window("Native App");
        assert_eq!(native_config.renderer, RendererMode::Native);
        assert!(native_config.gpu_effects);

        let hybrid_config = ArnikoConfig::hybrid("Hybrid App");
        assert_eq!(hybrid_config.renderer, RendererMode::Hybrid);
        assert!(hybrid_config.gpu_effects);
    }
}
