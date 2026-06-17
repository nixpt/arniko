//! Theme system for Arniko
//!
//! Provides theme management with multiple built-in themes:
//! - Dark (default dark theme)
//! - Light (clean light theme)
//! - System (follows OS preference)
//! - Frosted (clean white/gray glass)
//! - Cyberpunk (neon cyberpunk glass)
//! - Aurora (light pastel glass)
//!
//! Each theme also includes MacTahoe-inspired blur presets for the
//! Mustang GPU compositor.

use serde::{Deserialize, Serialize};

use arniko_mustang::{MacTahoeBlurPreset, BlurPresetBuilder, MACTAHOE_DEFAULT_BLUR_PARAMS};

/// Blur configuration for a theme
///
/// Provides a way to configure MacTahoe-style blur effects
/// for the Mustang GPU compositor per theme.
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeBlurConfig {
    /// Default blur preset for this theme
    pub default_preset: MacTahoeBlurPreset,
    /// Alternative blur presets for different components
    pub presets: Vec<(MacTahoeBlurPreset, String)>,
}

impl Default for ThemeBlurConfig {
    fn default() -> Self {
        Self {
            default_preset: MacTahoeBlurPreset::Medium,
            presets: vec![],
        }
    }
}

impl ThemeBlurConfig {
    /// Create a new blur configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration with the default preset
    pub fn with_default(mut self, preset: MacTahoeBlurPreset) -> Self {
        self.default_preset = preset;
        self
    }

    /// Add an alternative preset
    pub fn add_preset(mut self, preset: MacTahoeBlurPreset, name: String) -> Self {
        self.presets.push((preset, name));
        self
    }

    /// Get a preset by name
    pub fn get_preset(&self, name: &str) -> Option<&MacTahoeBlurPreset> {
        self.presets.iter().find(|(preset, _)| preset.name() == name).map(|(p, _)| *p)
    }

    /// Create a blur effect for a selector using the default preset
    pub fn effect(&self, selector: &str, viewport_width: u32, viewport_height: u32) -> crate::effect::Effect {
        let preset = self.default_preset();
        preset.effect(selector, viewport_width, viewport_height)
    }

    /// Get the default preset
    pub fn default_preset(&self) -> MacTahoeBlurPreset {
        self.default_preset
    }
}

/// Available theme modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ThemeMode {
    #[default]
    Dark, // Original dark theme
    Light,     // Clean light theme
    System,    // Follows OS preference
    Frosted,   // Clean white/gray glass
    Cyberpunk, // Neon cyberpunk glass
    Aurora,    // Light pastel glass
}

impl ThemeMode {
    /// Get the CSS variable overrides for this theme.
    /// System resolves to Dark for override purposes; the frontend
    /// should use `prefers-color-scheme` when in System mode.
    pub fn css_overrides(&self) -> &'static str {
        match self {
            ThemeMode::Dark => DARK_OVERRIDES,
            ThemeMode::Light => LIGHT_OVERRIDES,
            ThemeMode::System => DARK_OVERRIDES,
            ThemeMode::Frosted => FROSTED_OVERRIDES,
            ThemeMode::Cyberpunk => CYBERPUNK_OVERRIDES,
            ThemeMode::Aurora => AURORA_OVERRIDES,
        }
    }

    /// Get the theme name
    pub fn name(&self) -> &'static str {
        match self {
            ThemeMode::Dark => "Dark",
            ThemeMode::Light => "Light",
            ThemeMode::System => "System",
            ThemeMode::Frosted => "Frosted",
            ThemeMode::Cyberpunk => "Cyberpunk",
            ThemeMode::Aurora => "Aurora",
        }
    }

    /// Get the CSS class name for this theme (e.g. "theme-light").
    /// Apply this class to `<html>` or a container element to activate
    /// the theme's CSS variable overrides.
    pub fn html_class(&self) -> &'static str {
        match self {
            ThemeMode::Dark => "theme-dark",
            ThemeMode::Light => "theme-light",
            ThemeMode::System => "theme-system",
            ThemeMode::Frosted => "theme-frosted",
            ThemeMode::Cyberpunk => "theme-cyberpunk",
            ThemeMode::Aurora => "theme-aurora",
        }
    }

    /// All available theme modes
    pub fn all() -> &'static [ThemeMode] {
        &[
            ThemeMode::Dark,
            ThemeMode::Light,
            ThemeMode::System,
            ThemeMode::Frosted,
            ThemeMode::Cyberpunk,
            ThemeMode::Aurora,
        ]
    }

    /// Get MacTahoe-inspired blur presets for this theme
    ///
    /// Different themes have different blur characteristics:
    /// - Dark/Light: minimal blur for contrast
    /// - Frosted/Cyberpunk/Aurora: glass effects with blur
    pub fn blur_presets(&self) -> Vec<(MacTahoeBlurPreset, &'static str)> {
        match self {
            ThemeMode::Dark => vec![
                (MacTahoeBlurPreset::None, "Dark"),
                (MacTahoeBlurPreset::Subtle, "Subtle"),
                (MacTahoeBlurPreset::Medium, "Medium"),
            ],
            ThemeMode::Light => vec![
                (MacTahoeBlurPreset::None, "Light"),
                (MacTahoeBlurPreset::Subtle, "Subtle"),
                (MacTahoeBlurPreset::Medium, "Medium"),
            ],
            ThemeMode::System => vec![
                (MacTahoeBlurPreset::None, "System"),
                (MacTahoeBlurPreset::Medium, "Medium"),
                (MacTahoeBlurPreset::Frosted, "Frosted"),
            ],
            ThemeMode::Frosted => vec![
                (MacTahoeBlurPreset::None, "Frosted"),
                (MacTahoeBlurPreset::Subtle, "Subtle"),
                (MacTahoeBlurPreset::Medium, "Medium"),
                (MacTahoeBlurPreset::Strong, "Strong"),
                (MacTahoeBlurPreset::Aurora, "Aurora"),
                (MacTahoeBlurPreset::Cyberpunk, "Cyberpunk"),
            ],
            ThemeMode::Cyberpunk => vec![
                (MacTahoeBlurPreset::None, "Cyberpunk"),
                (MacTahoeBlurPreset::Subtle, "Subtle"),
                (MacTahoeBlurPreset::Strong, "Strong"),
                (MacTahoeBlurPreset::Cyberpunk, "Cyberpunk"),
                (MacTahoeBlurPreset::Ultra, "Ultra"),
            ],
            ThemeMode::Aurora => vec![
                (MacTahoeBlurPreset::None, "Aurora"),
                (MacTahoeBlurPreset::Subtle, "Subtle"),
                (MacTahoeBlurPreset::Medium, "Medium"),
                (MacTahoeBlurPreset::Aurora, "Aurora"),
            ],
        }
    }

    /// Get the default blur preset for this theme
    pub fn default_blur_preset(&self) -> MacTahoeBlurPreset {
        match self {
            ThemeMode::Dark => MacTahoeBlurPreset::Subtle,
            ThemeMode::Light => MacTahoeBlurPreset::Subtle,
            ThemeMode::System => MacTahoeBlurPreset::Medium,
            ThemeMode::Frosted => MacTahoeBlurPreset::Frosted,
            ThemeMode::Cyberpunk => MacTahoeBlurPreset::Cyberpunk,
            ThemeMode::Aurora => MacTahoeBlurPreset::Aurora,
        }
    }

    /// Create a blur effect for this theme using the default preset
    pub fn default_blur_effect(&self, selector: &str, viewport_width: u32, viewport_height: u32) -> crate::effect::Effect {
        self.default_blur_preset().effect(selector, viewport_width, viewport_height)
    }
}

/// Theme manager for caching and generating themed CSS
#[derive(Debug, Clone)]
pub struct ThemeManager {
    cached_css: std::collections::HashMap<ThemeMode, String>,
}

impl ThemeManager {
    /// Create a new theme manager
    pub fn new() -> Self {
        Self {
            cached_css: std::collections::HashMap::new(),
        }
    }

    /// Generate CSS for a theme with base styles
    pub fn generate_css(&mut self, theme: &ThemeMode, base_css: &str) -> String {
        if let Some(cached) = self.cached_css.get(theme) {
            return cached.clone();
        }

        let combined = format!("{}\n{}", base_css, theme.css_overrides());
        self.cached_css.insert(*theme, combined.clone());
        combined
    }

    /// Get cached CSS for a theme (returns None if not cached)
    pub fn get_cached(&self, theme: &ThemeMode) -> Option<&String> {
        self.cached_css.get(theme)
    }

    /// Clear all cached themes
    pub fn clear_cache(&mut self) {
        self.cached_css.clear();
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

// Dark is the default theme - defines the full variable set
const DARK_OVERRIDES: &str = r#"
:root {
    --arniko-bg-surface: #18181b;
    --arniko-bg-elevated: #18181b;
    --arniko-bg-input: #18181b;
    --arniko-bg-subtle: #27272a;
    --arniko-bg-shimmer: #3f3f46;
    --arniko-bg-hover: rgba(255, 255, 255, 0.05);
    --arniko-text-primary: #f5f5f5;
    --arniko-text-body: #e4e4e7;
    --arniko-text-secondary: #a1a1aa;
    --arniko-text-muted: #71717a;
    --arniko-text-subtle: #52525b;
    --arniko-border: #27272a;
    --arniko-border-light: rgba(0, 242, 255, 0.06);
    --arniko-border-accent: rgba(0, 242, 255, 0.15);
    --arniko-accent: #6366f1;
    --arniko-accent-glow: rgba(99, 102, 241, 0.3);
    --arniko-cyan: #00f2ff;
    --arniko-purple: #a855f7;
    --arniko-white: #ffffff;
    --arniko-btn-default: #2563eb;
    --arniko-btn-destructive: #dc2626;
    --arniko-btn-outline-border: #374151;
    --arniko-btn-secondary-bg: #374151;
    --arniko-btn-accent-bg: #7c3aed;
    --arniko-success: #22c55e;
    --arniko-success-bg: #052e16;
    --arniko-success-border: #166534;
    --arniko-error: #ef4444;
    --arniko-error-bg: #450a0a;
    --arniko-error-border: #991b1b;
    --arniko-warning: #f59e0b;
    --arniko-warning-bg: #422006;
    --arniko-warning-border: #854d0e;
    --arniko-info: #60a5fa;
    --arniko-info-bg: #172554;
    --arniko-info-border: #1e3a8a;
    --arniko-splash-bg: #06090e;
}
"#;

const LIGHT_OVERRIDES: &str = r#"
:root {
    --arniko-bg-surface: #ffffff;
    --arniko-bg-elevated: #fafafa;
    --arniko-bg-input: #ffffff;
    --arniko-bg-subtle: #e4e4e7;
    --arniko-bg-shimmer: #d4d4d8;
    --arniko-bg-hover: rgba(0, 0, 0, 0.04);
    --arniko-text-primary: #18181b;
    --arniko-text-body: #3f3f46;
    --arniko-text-secondary: #71717a;
    --arniko-text-muted: #a1a1aa;
    --arniko-text-subtle: #d4d4d8;
    --arniko-border: #e4e4e7;
    --arniko-border-light: rgba(0, 0, 0, 0.06);
    --arniko-border-accent: rgba(99, 102, 241, 0.2);
    --arniko-accent: #6366f1;
    --arniko-accent-glow: rgba(99, 102, 241, 0.15);
    --arniko-cyan: #0891b2;
    --arniko-purple: #7c3aed;
    --arniko-white: #ffffff;
    --arniko-btn-default: #2563eb;
    --arniko-btn-destructive: #dc2626;
    --arniko-btn-outline-border: #d4d4d8;
    --arniko-btn-secondary-bg: #f4f4f5;
    --arniko-btn-accent-bg: #7c3aed;
    --arniko-success: #16a34a;
    --arniko-success-bg: #f0fdf4;
    --arniko-success-border: #bbf7d0;
    --arniko-error: #dc2626;
    --arniko-error-bg: #fef2f2;
    --arniko-error-border: #fecaca;
    --arniko-warning: #d97706;
    --arniko-warning-bg: #fffbf0;
    --arniko-warning-border: #fde68a;
    --arniko-info: #2563eb;
    --arniko-info-bg: #f0f6ff;
    --arniko-info-border: #bfdbfe;
    --arniko-splash-bg: #f8f8f8;
}
"#;

const FROSTED_OVERRIDES: &str = r#"
:root {
    --arniko-bg-surface: #f8fafc;
    --arniko-bg-elevated: #ffffff;
    --arniko-bg-input: #ffffff;
    --arniko-bg-subtle: #e2e8f0;
    --arniko-bg-shimmer: #cbd5e1;
    --arniko-bg-hover: rgba(148, 163, 184, 0.15);
    --arniko-text-primary: #0f172a;
    --arniko-text-body: #334155;
    --arniko-text-secondary: #475569;
    --arniko-text-muted: #64748b;
    --arniko-text-subtle: #94a3b8;
    --arniko-border: #cbd5e1;
    --arniko-border-light: rgba(0, 0, 0, 0.06);
    --arniko-border-accent: rgba(59, 130, 246, 0.2);
    --arniko-accent: #3b82f6;
    --arniko-accent-glow: rgba(59, 130, 246, 0.2);
    --arniko-cyan: #08a3b2;
    --arniko-purple: #8b5cf6;
    --arniko-white: #ffffff;
    --arniko-btn-default: #2563eb;
    --arniko-btn-destructive: #dc2626;
    --arniko-btn-outline-border: #cbd5e1;
    --arniko-btn-secondary-bg: #f1f5f9;
    --arniko-btn-accent-bg: #3b82f6;
    --arniko-success: #16a34a;
    --arniko-success-bg: #f0fdf4;
    --arniko-success-border: #bbf7d0;
    --arniko-error: #dc2626;
    --arniko-error-bg: #fef2f2;
    --arniko-error-border: #fecaca;
    --arniko-warning: #d97706;
    --arniko-warning-bg: #fffbf0;
    --arniko-warning-border: #fde68a;
    --arniko-info: #2563eb;
    --arniko-info-bg: #f0f6ff;
    --arniko-info-border: #bfdbfe;
    --arniko-splash-bg: #f8fafc;
}
"#;

const CYBERPUNK_OVERRIDES: &str = r#"
:root {
    --arniko-bg-surface: #05050a;
    --arniko-bg-elevated: #0a0a14;
    --arniko-bg-input: #0a0a14;
    --arniko-bg-subtle: #151525;
    --arniko-bg-shimmer: #1e1e30;
    --arniko-bg-hover: rgba(16, 185, 129, 0.12);
    --arniko-text-primary: #ecfdf5;
    --arniko-text-body: #d1fae5;
    --arniko-text-secondary: #a7f3d0;
    --arniko-text-muted: #6ee7b7;
    --arniko-text-subtle: #34d399;
    --arniko-border: rgba(16, 185, 129, 0.2);
    --arniko-border-light: rgba(16, 185, 129, 0.08);
    --arniko-border-accent: rgba(34, 211, 238, 0.25);
    --arniko-accent: #22d3ee;
    --arniko-accent-glow: rgba(34, 211, 238, 0.3);
    --arniko-cyan: #22d3ee;
    --arniko-purple: #a855f7;
    --arniko-white: #ffffff;
    --arniko-btn-default: #10b981;
    --arniko-btn-destructive: #dc2626;
    --arniko-btn-outline-border: rgba(16, 185, 129, 0.3);
    --arniko-btn-secondary-bg: #151525;
    --arniko-btn-accent-bg: #14b8a6;
    --arniko-success: #34d399;
    --arniko-success-bg: rgba(16, 185, 129, 0.12);
    --arniko-success-border: rgba(52, 211, 153, 0.2);
    --arniko-error: #f87171;
    --arniko-error-bg: rgba(239, 68, 68, 0.12);
    --arniko-error-border: rgba(248, 113, 113, 0.2);
    --arniko-warning: #fbbf24;
    --arniko-warning-bg: rgba(245, 158, 11, 0.12);
    --arniko-warning-border: rgba(251, 191, 36, 0.2);
    --arniko-info: #60a5fa;
    --arniko-info-bg: rgba(96, 165, 250, 0.12);
    --arniko-info-border: rgba(96, 165, 250, 0.2);
    --arniko-splash-bg: #05050a;
}
"#;

const AURORA_OVERRIDES: &str = r#"
:root {
    --arniko-bg-surface: #f5f3ff;
    --arniko-bg-elevated: #ffffff;
    --arniko-bg-input: #ffffff;
    --arniko-bg-subtle: #ede9fe;
    --arniko-bg-shimmer: #ddd6fe;
    --arniko-bg-hover: rgba(147, 51, 234, 0.08);
    --arniko-text-primary: #1f2937;
    --arniko-text-body: #374151;
    --arniko-text-secondary: #4b5563;
    --arniko-text-muted: #9ca3af;
    --arniko-text-subtle: #d1d5db;
    --arniko-border: rgba(147, 51, 234, 0.15);
    --arniko-border-light: rgba(147, 51, 234, 0.06);
    --arniko-border-accent: rgba(236, 72, 153, 0.2);
    --arniko-accent: #ec4899;
    --arniko-accent-glow: rgba(236, 72, 153, 0.2);
    --arniko-cyan: #22d3ee;
    --arniko-purple: #a855f7;
    --arniko-white: #ffffff;
    --arniko-btn-default: #ec4899;
    --arniko-btn-destructive: #dc2626;
    --arniko-btn-outline-border: rgba(147, 51, 234, 0.25);
    --arniko-btn-secondary-bg: #f5f3ff;
    --arniko-btn-accent-bg: #a855f7;
    --arniko-success: #16a34a;
    --arniko-success-bg: #f0fdf4;
    --arniko-success-border: #bbf7d0;
    --arniko-error: #dc2626;
    --arniko-error-bg: #fef2f2;
    --arniko-error-border: #fecaca;
    --arniko-warning: #d97706;
    --arniko-warning-bg: #fffbf0;
    --arniko-warning-border: #fde68a;
    --arniko-info: #2563eb;
    --arniko-info-bg: #f0f6ff;
    --arniko-info-border: #bfdbfe;
    --arniko-splash-bg: #f5f3ff;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_mode_name() {
        assert_eq!(ThemeMode::Dark.name(), "Dark");
        assert_eq!(ThemeMode::Light.name(), "Light");
        assert_eq!(ThemeMode::System.name(), "System");
        assert_eq!(ThemeMode::Frosted.name(), "Frosted");
        assert_eq!(ThemeMode::Cyberpunk.name(), "Cyberpunk");
        assert_eq!(ThemeMode::Aurora.name(), "Aurora");
    }

    #[test]
    fn test_theme_manager_new() {
        let manager = ThemeManager::new();
        assert!(manager.cached_css.is_empty());
    }

    #[test]
    fn test_theme_manager_generate_css() {
        let mut manager = ThemeManager::new();
        let base = "body { margin: 0; }";

        let css = manager.generate_css(&ThemeMode::Dark, base);
        assert!(css.contains("body { margin: 0; }"));
        assert!(css.contains("--arniko-bg-surface"));

        // Should be cached now
        let cached = manager.get_cached(&ThemeMode::Dark);
        assert!(cached.is_some());
    }
}
