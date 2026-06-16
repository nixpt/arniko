//! Theme system for Arniko
//!
//! Provides theme management with multiple built-in themes:
//! - Dark (default dark theme)
//! - Light (clean light theme)
//! - System (follows OS preference)
//! - Frosted (clean white/gray glass)
//! - Cyberpunk (neon cyberpunk glass)
//! - Aurora (light pastel glass)

use serde::{Deserialize, Serialize};

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
    --bg-void: #020205;
    --bg-surface: #0a0a12;
    --bg-elevated: #12121f;
    --bg-input: #0f0f1a;
    --bg-subtle: #18182a;
    --bg-skeleton: #1e1e2e;
    --bg-hover: rgba(255, 255, 255, 0.04);
    --bg-active: rgba(167, 139, 250, 0.1);
    --bg-accent-subtle: rgba(167, 139, 250, 0.08);
    --bg-accent-hover: rgba(167, 139, 250, 0.12);
    --text-primary: #f5f5f5;
    --text-secondary: #d4d4d8;
    --text-muted: #a1a1aa;
    --text-accent: #c4b5fd;
    --border: rgba(255, 255, 255, 0.06);
    --border-accent: rgba(167, 139, 250, 0.3);
    --accent: #a78bfa;
    --accent-hover: #c4b5fd;
    --accent-light: #ddd6fe;
    --sidebar-logo: #c4b5fd;
    --tab-bg: rgba(255, 255, 255, 0.03);
    --tab-hover: rgba(255, 255, 255, 0.06);
    --tab-active-bg: rgba(124, 58, 237, 0.15);
    --metric-bar-bg: #1e1e2e;
    --metric-fill: #a78bfa;
    --spinner-border: rgba(167, 139, 250, 0.3);
    --status-ok: #4ade80;
    --status-ok-bg: rgba(34, 197, 94, 0.1);
    --status-warn: #fbbf24;
    --status-warn-bg: rgba(245, 158, 11, 0.1);
    --status-error: #f87171;
    --status-error-bg: rgba(239, 68, 68, 0.1);
    --status-info: #60a5fa;
    --status-info-bg: rgba(96, 165, 250, 0.1);
    --glass-bg: rgba(255, 255, 255, 0.025);
    --glass-border: rgba(255, 255, 255, 0.06);
    --glass-border-hover: rgba(255, 255, 255, 0.12);
    --glass-shadow: 0 4px 24px rgba(0, 0, 0, 0.2);
    --glass-shadow-hover: 0 12px 40px rgba(0, 0, 0, 0.35);
    --accent-secondary: #6366f1;
    --accent-glow: rgba(167, 139, 250, 0.3);
    --badge-bg: rgba(255, 255, 255, 0.04);
    --badge-border: rgba(255, 255, 255, 0.08);
    --status-ok-border: rgba(34, 197, 94, 0.2);
    --status-warn-border: rgba(245, 158, 11, 0.2);
    --status-error-border: rgba(239, 68, 68, 0.2);
    --status-info-border: rgba(96, 165, 250, 0.2);
}
"#;

const LIGHT_OVERRIDES: &str = r#"
:root {
    --bg-void: #fafafa;
    --bg-surface: #f4f4f5;
    --bg-elevated: #ffffff;
    --bg-input: #ffffff;
    --bg-subtle: #f4f4f5;
    --bg-skeleton: #e4e4e7;
    --bg-hover: rgba(0, 0, 0, 0.03);
    --bg-active: rgba(99, 102, 241, 0.08);
    --bg-accent-subtle: rgba(99, 102, 241, 0.06);
    --bg-accent-hover: rgba(99, 102, 241, 0.1);
    --text-primary: #18181b;
    --text-secondary: #3f3f46;
    --text-muted: #71717a;
    --text-accent: #6366f1;
    --border: rgba(0, 0, 0, 0.08);
    --border-accent: rgba(99, 102, 241, 0.3);
    --accent: #6366f1;
    --accent-hover: #818cf8;
    --accent-light: #a5b4fc;
    --sidebar-logo: #6366f1;
    --tab-bg: rgba(0, 0, 0, 0.02);
    --tab-hover: rgba(0, 0, 0, 0.04);
    --tab-active-bg: rgba(99, 102, 241, 0.1);
    --metric-bar-bg: #e4e4e7;
    --metric-fill: #6366f1;
    --spinner-border: rgba(99, 102, 241, 0.3);
    --status-ok: #16a34a;
    --status-ok-bg: rgba(34, 197, 94, 0.12);
    --status-warn: #d97706;
    --status-warn-bg: rgba(245, 158, 11, 0.12);
    --status-error: #dc2626;
    --status-error-bg: rgba(239, 68, 68, 0.12);
    --status-info: #2563eb;
    --status-info-bg: rgba(59, 130, 246, 0.12);
    --glass-bg: rgba(255, 255, 255, 0.5);
    --glass-border: rgba(0, 0, 0, 0.06);
    --glass-border-hover: rgba(0, 0, 0, 0.12);
    --glass-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
    --glass-shadow-hover: 0 8px 32px rgba(0, 0, 0, 0.1);
    --accent-secondary: #818cf8;
    --accent-glow: rgba(99, 102, 241, 0.3);
    --badge-bg: rgba(0, 0, 0, 0.03);
    --badge-border: rgba(0, 0, 0, 0.06);
    --status-ok-border: rgba(34, 197, 94, 0.2);
    --status-warn-border: rgba(245, 158, 11, 0.2);
    --status-error-border: rgba(239, 68, 68, 0.2);
    --status-info-border: rgba(59, 130, 246, 0.2);
}
"#;

const FROSTED_OVERRIDES: &str = r#"
:root {
    --bg-void: #f8fafc;
    --bg-surface: #e2e8f0;
    --bg-elevated: #ffffff;
    --bg-input: #ffffff;
    --bg-subtle: #f1f5f9;
    --bg-skeleton: #e2e8f0;
    --bg-hover: rgba(148, 163, 184, 0.15);
    --bg-active: rgba(59, 130, 246, 0.12);
    --bg-accent-subtle: rgba(59, 130, 246, 0.1);
    --bg-accent-hover: rgba(59, 130, 246, 0.1);
    --text-primary: #0f172a;
    --text-secondary: #475569;
    --text-muted: #64748b;
    --text-accent: #3b82f6;
    --border: #94a3b8;
    --border-accent: #3b82f6;
    --accent: #3b82f6;
    --accent-hover: #60a5fa;
    --accent-light: #60a5fa;
    --sidebar-logo: #3b82f6;
    --tab-bg: rgba(255, 255, 255, 0.6);
    --tab-hover: rgba(255, 255, 255, 0.8);
    --tab-active-bg: #ffffff;
    --metric-bar-bg: #cbd5e1;
    --metric-fill: #3b82f6;
    --spinner-border: #94a3b8;
    --status-ok: #16a34a;
    --status-ok-bg: rgba(34, 197, 94, 0.12);
    --status-warn: #d97706;
    --status-warn-bg: rgba(245, 158, 11, 0.12);
    --status-error: #dc2626;
    --status-error-bg: rgba(239, 68, 68, 0.12);
    --status-info: #2563eb;
    --status-info-bg: rgba(59, 130, 246, 0.12);
    --glass-bg: rgba(255, 255, 255, 0.6);
    --glass-border: rgba(255, 255, 255, 0.3);
    --glass-border-hover: rgba(255, 255, 255, 0.5);
    --glass-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
    --glass-shadow-hover: 0 8px 32px rgba(0, 0, 0, 0.1);
    --accent-secondary: #60a5fa;
    --accent-glow: rgba(59, 130, 246, 0.3);
    --badge-bg: rgba(255, 255, 255, 0.04);
    --badge-border: rgba(255, 255, 255, 0.08);
    --status-ok-border: rgba(34, 197, 94, 0.2);
    --status-warn-border: rgba(245, 158, 11, 0.2);
    --status-error-border: rgba(239, 68, 68, 0.2);
    --status-info-border: rgba(59, 130, 246, 0.2);
}
"#;

const CYBERPUNK_OVERRIDES: &str = r#"
:root {
    --bg-void: #05050a;
    --bg-surface: #0a0a14;
    --bg-elevated: #151525;
    --bg-input: #101020;
    --bg-subtle: #1a1a30;
    --bg-skeleton: #1a1a30;
    --bg-hover: rgba(16, 185, 129, 0.15);
    --bg-active: rgba(34, 211, 238, 0.15);
    --bg-accent-subtle: rgba(16, 185, 129, 0.08);
    --bg-accent-hover: rgba(34, 211, 238, 0.15);
    --text-primary: #ffffff;
    --text-secondary: #d1fae5;
    --text-muted: #9ca3af;
    --text-accent: #22d3ee;
    --border: rgba(16, 185, 129, 0.3);
    --border-accent: #22d3ee;
    --accent: #22d3ee;
    --accent-hover: #60a5fa;
    --accent-light: #60a5fa;
    --sidebar-logo: #22d3ee;
    --tab-bg: rgba(16, 185, 129, 0.08);
    --tab-hover: rgba(16, 185, 129, 0.15);
    --tab-active-bg: #151525;
    --metric-bar-bg: #1a1a30;
    --metric-fill: #22d3ee;
    --spinner-border: rgba(16, 185, 129, 0.3);
    --status-ok: #34d399;
    --status-ok-bg: rgba(16, 185, 129, 0.15);
    --status-warn: #fbbf24;
    --status-warn-bg: rgba(245, 158, 11, 0.15);
    --status-error: #f87171;
    --status-error-bg: rgba(239, 68, 68, 0.15);
    --status-info: #60a5fa;
    --status-info-bg: rgba(96, 165, 250, 0.15);
    --glass-bg: rgba(16, 185, 129, 0.05);
    --glass-border: rgba(16, 185, 129, 0.15);
    --glass-border-hover: rgba(34, 211, 238, 0.3);
    --glass-shadow: 0 4px 24px rgba(0, 0, 0, 0.4);
    --glass-shadow-hover: 0 8px 40px rgba(0, 0, 0, 0.5);
    --accent-secondary: #60a5fa;
    --accent-glow: rgba(34, 211, 238, 0.3);
    --badge-bg: rgba(16, 185, 129, 0.05);
    --badge-border: rgba(16, 185, 129, 0.1);
    --status-ok-border: rgba(52, 211, 153, 0.2);
    --status-warn-border: rgba(245, 158, 11, 0.2);
    --status-error-border: rgba(248, 113, 113, 0.2);
    --status-info-border: rgba(96, 165, 250, 0.2);
}
"#;

const AURORA_OVERRIDES: &str = r#"
:root {
    --bg-void: #f5f3ff;
    --bg-surface: #f0f4ff;
    --bg-elevated: #ffffff;
    --bg-input: #ffffff;
    --bg-subtle: #fafafa;
    --bg-skeleton: #f0f4ff;
    --bg-hover: rgba(147, 51, 234, 0.1);
    --bg-active: rgba(236, 72, 153, 0.12);
    --bg-accent-subtle: rgba(236, 72, 153, 0.06);
    --bg-accent-hover: rgba(236, 72, 153, 0.1);
    --text-primary: #1f2937;
    --text-secondary: #4b5563;
    --text-muted: #9ca3af;
    --text-accent: #ec4899;
    --border: rgba(147, 51, 234, 0.25);
    --border-accent: #ec4899;
    --accent: #ec4899;
    --accent-hover: #f472b6;
    --accent-light: #f472b6;
    --sidebar-logo: #ec4899;
    --tab-bg: rgba(147, 51, 234, 0.06);
    --tab-hover: rgba(147, 51, 234, 0.12);
    --tab-active-bg: #ffffff;
    --metric-bar-bg: #fafafa;
    --metric-fill: #ec4899;
    --spinner-border: rgba(147, 51, 234, 0.25);
    --status-ok: #16a34a;
    --status-ok-bg: rgba(34, 197, 94, 0.12);
    --status-warn: #d97706;
    --status-warn-bg: rgba(245, 158, 11, 0.12);
    --status-error: #dc2626;
    --status-error-bg: rgba(239, 68, 68, 0.12);
    --status-info: #2563eb;
    --status-info-bg: rgba(59, 130, 246, 0.12);
    --glass-bg: rgba(255, 255, 255, 0.5);
    --glass-border: rgba(147, 51, 234, 0.15);
    --glass-border-hover: rgba(147, 51, 234, 0.3);
    --glass-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
    --glass-shadow-hover: 0 8px 32px rgba(0, 0, 0, 0.1);
    --accent-secondary: #f472b6;
    --accent-glow: rgba(236, 72, 153, 0.3);
    --badge-bg: rgba(0, 0, 0, 0.03);
    --badge-border: rgba(0, 0, 0, 0.06);
    --status-ok-border: rgba(34, 197, 94, 0.2);
    --status-warn-border: rgba(245, 158, 11, 0.2);
    --status-error-border: rgba(239, 68, 68, 0.2);
    --status-info-border: rgba(59, 130, 246, 0.2);
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
        assert!(css.contains("--bg-void"));

        // Should be cached now
        let cached = manager.get_cached(&ThemeMode::Dark);
        assert!(cached.is_some());
    }
}
