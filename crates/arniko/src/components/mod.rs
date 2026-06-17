//! UI Components for Arniko
//!
//! This module provides a comprehensive set of UI components
//! that can render to HTML strings or native UI elements.
//!
//! ## Icon Theme Support
//!
//! Arniko supports configurable icon themes including:
//! - Default (emoji or built-in system icons)
//! - MacTahoe (macOS Tahoe-style icons with 8 color variants)
//!
//! Set the icon theme via the [`IconThemeVariant`] configuration option.

pub mod alert;
pub mod alert_panel;
pub mod badge;
pub mod bar_chart;
pub mod button;
pub mod card;
pub mod empty_state;
pub mod feed;
pub mod file_tree;
pub mod icon_theme;
pub mod input;
pub mod kbd;
pub mod keyboard_shortcuts;
pub mod metric_card;
pub mod panel;
pub mod progress_bar;
pub mod progress_ring;
pub mod separator;
pub mod skeleton;
pub mod sparkline;
pub mod spinner;
pub mod splash_screen;
pub mod status_badge;
pub mod status_grid;
pub mod styles;
pub mod svg_bar_chart;
pub mod svg_line_chart;
mod svg_util;
pub mod theme_toggle;
pub mod toast;
pub mod tooltip;

// Re-export all components for convenience
pub use alert::*;
pub use alert_panel::*;
pub use badge::*;
pub use bar_chart::*;
pub use button::*;
pub use card::*;
pub use empty_state::*;
pub use feed::*;
pub use file_tree::*;
pub use input::*;
pub use kbd::*;
pub use keyboard_shortcuts::*;
pub use metric_card::*;
pub use panel::*;
pub use progress_bar::*;
pub use progress_ring::*;
pub use separator::*;
pub use skeleton::*;
pub use sparkline::*;
pub use spinner::*;
pub use splash_screen::*;
pub use status_badge::*;
pub use status_grid::*;
pub use svg_bar_chart::*;
pub use svg_line_chart::*;
pub use theme_toggle::*;
pub use toast::*;
pub use tooltip::*;

/// The default Arniko stylesheet. Include this in your HTML `<style>` tag.
/// Assembled at compile time from per-component CSS files in the [`styles`] module.
pub use styles::ARNIKO_STYLES;

/// Escape HTML-special characters in user-provided strings so they render
/// inert when interpolated into HTML output. Prevents XSS.
///
/// Escapes: `&` → `&amp;`, `<` → `&lt;`, `>` → `&gt;`, `"` → `&quot;`
pub fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}
