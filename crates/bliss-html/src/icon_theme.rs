//! Icon theme support for Arniko
//!
//! Provides configurable icon themes including the MacTahoe icon set.
//! Supports SVG icons with 8 color variants (blue/purple/green/red/orange/yellow/grey/nord).

use std::path::PathBuf;
use std::sync::OnceLock;

/// Icon theme variants supported by Arniko
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconThemeVariant {
    /// Default system icons (emoji or built-in)
    #[default]
    Default,
    /// MacTahoe icon theme (8 color variants)
    MacTahoe,
}

/// Global icon theme instance (lazy-loaded)
static ICON_THEME: OnceLock<IconTheme> = OnceLock::new();

/// Icon theme configuration
#[derive(Clone, Debug)]
pub struct IconTheme {
    variant: IconThemeVariant,
    icon_path: Option<PathBuf>,
}

impl IconTheme {
    /// Create a new icon theme instance
    pub fn new(variant: IconThemeVariant) -> Self {
        Self {
            variant,
            icon_path: None,
        }
    }

    /// Set the icon theme path (useful for custom MacTahoe installations)
    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.icon_path = Some(path);
        self
    }

    /// Get the icon theme variant
    pub fn variant(&self) -> IconThemeVariant {
        self.variant
    }

    /// Get the icon path
    pub fn icon_path(&self) -> &Option<PathBuf> {
        &self.icon_path
    }

    /// Resolve the full path to a MacTahoe icon file
    ///
    /// The MacTahoe theme uses symbolic icons organized as:
    /// - actions/symbolic/ (window controls, menu icons)
    /// - apps/symbolic/ (application icons)
    /// - devices/symbolic/ (hardware icons)
    /// - status/16/ (status icons)
    /// - status/22/ (status icons)
    ///
    /// For Arniko's use case, we map common icons:
    /// - File explorer: system-file-manager-symbolic.svg
    /// - Terminal: utilities-terminal-symbolic.svg
    /// - Calculator: accessories-calculator-symbolic.svg
    /// - Browser: web-browser-symbolic.svg
    /// - etc.
    pub fn icon_path_for(&self, icon_name: &str) -> Option<PathBuf> {
        let path = self.icon_path.as_ref()?.join(icon_name);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// Check if a specific icon exists in the theme
    pub fn has_icon(&self, icon_name: &str) -> bool {
        self.icon_path_for(icon_name).is_some()
    }

    /// Load an SVG icon from the theme
    ///
    /// Returns the SVG content as a String.
    pub fn load_svg(&self, icon_name: &str) -> Option<String> {
        if let Some(path) = self.icon_path_for(icon_name) {
            std::fs::read_to_string(path).ok()
        } else {
            None
        }
    }

    /// Load all SVG files from the theme directory
    ///
    /// Returns a map of icon names to their SVG content.
    pub fn load_all(&self) -> std::collections::BTreeMap<String, String> {
        let icon_path = self.icon_path.as_ref()
            .expect("icon_path must be set to load all icons");
        
        let mut icons = std::collections::BTreeMap::new();
        
        if let Ok(entries) = std::fs::read_dir(icon_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "svg") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        // Extract just the filename (basename without extension)
                        let filename = path
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();
                        // Remove .svg extension
                        let icon_name = filename
                            .rsplit(".svg")
                            .next()
                            .unwrap_or(&filename)
                            .trim_end_matches(".svg");
                        icons.insert(icon_name.to_string(), content);
                    }
                }
            }
        }
        
        icons
    }
}

/// Get the global icon theme instance
pub fn get_global_icon_theme() -> &'static IconTheme {
    ICON_THEME.get_or_init(|| {
        // Default to MacTahoe theme path
        let theme_path = PathBuf::from("/home/nixp/MacTahoe-icon-theme/bold");
        IconTheme::new(IconThemeVariant::MacTahoe).with_path(theme_path)
    })
}

/// Get a reference to the global icon theme
pub fn global_icon_theme() -> Option<&'static IconTheme> {
    ICON_THEME.get()
}

/// Resolve a MacTahoe icon path from a symbolic name
///
/// Common mappings:
/// - "file-manager" → system-file-manager-symbolic.svg
/// - "terminal" → utilities-terminal-symbolic.svg
/// - "browser" → web-browser-symbolic.svg
/// - "settings" → preferences-system-symbolic.svg
pub fn resolve_mac_tahoe_icon(name: &str) -> Option<PathBuf> {
    // Map Arniko-friendly names to MacTahoe symbolic names
    let mapping = match name {
        "file-manager" | "explorer" | "folder" => "system-file-manager-symbolic.svg",
        "terminal" | "tux" | "shell" => "utilities-terminal-symbolic.svg",
        "browser" | "internet" | "web" | "firefox" | "chrome" | "safari" => "web-browser-symbolic.svg",
        "settings" | "preferences" | "gear" => "preferences-system-symbolic.svg",
        "calculator" | "calc" => "accessories-calculator-symbolic.svg",
        "music" | "audio" | "speaker" | "volume" => "multimedia-volume-control-symbolic.svg",
        "document" | "doc" | "text" | "page" | "file" => "file-roller-symbolic.svg",
        "code" | "editor" | "code-editor" | "rust" => "inkscape-symbolic.svg",
        "box" | "archive" | "package" | "zipped" => "org.gnome.Boxes-symbolic.svg",
        "lock" | "secure" | "security" => "system-lock-screen-symbolic.svg",
        "calendar" | "date" | "clock" => "org.gnome.Polari-symbolic.svg",
        "network" | "wifi" | "internet" | "connect" => "application-menu-symbolic.svg",
        "power" | "shutdown" | "restart" => "dark-mode-symbolic.svg",
        "search" | "find" | "locate" => "find-location-symbolic.svg",
        "screenshots" | "capture" | "screen" => "screenshooter-symbolic.svg",
        "help" | "question" | "?" => "preferences-desktop-accessibility-symbolic.svg",
        _ => return None,
    };
    
    let theme_path = PathBuf::from("/home/nixp/MacTahoe-icon-theme/bold");
    Some(theme_path.join(mapping))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_theme_variant() {
        let theme = IconTheme::new(IconThemeVariant::MacTahoe);
        assert_eq!(theme.variant(), IconThemeVariant::MacTahoe);
    }

    #[test]
    fn test_icon_theme_default() {
        let theme = IconTheme::new(IconThemeVariant::Default);
        assert_eq!(theme.variant(), IconThemeVariant::Default);
    }

    #[test]
    fn test_resolve_mac_tahoe_icon() {
        assert!(resolve_mac_tahoe_icon("terminal").is_some());
        assert!(resolve_mac_tahoe_icon("browser").is_some());
        assert!(resolve_mac_tahoe_icon("calculator").is_some());
        assert!(resolve_mac_tahoe_icon("nonexistent").is_none());
    }
}
