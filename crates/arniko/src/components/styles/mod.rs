//! Per-component CSS stylesheets.
//!
//! Each component has its own `.css` file in this directory.
//! They are concatenated at compile time via `concat!(include_str!(...))`
//! into a single `ARNIKO_STYLES` constant — zero runtime overhead.

/// The complete Arniko component stylesheet, assembled from per-component CSS files.
///
/// Include this in your HTML `<style>` tag.
///
/// The first two entries are the design-token variable definitions and
/// the light-theme overrides. Component CSS files use `var(--arniko-*)`
/// references so themes work out of the box.
pub const ARNIKO_STYLES: &str = concat!(
    include_str!("variables.css"),
    include_str!("theme_light.css"),
    include_str!("button.css"),
    include_str!("card.css"),
    include_str!("input.css"),
    include_str!("badge.css"),
    include_str!("separator.css"),
    include_str!("spinner.css"),
    include_str!("kbd.css"),
    include_str!("metric_card.css"),
    include_str!("progress_bar.css"),
    include_str!("status_grid.css"),
    include_str!("status_badge.css"),
    include_str!("tooltip.css"),
    include_str!("skeleton.css"),
    include_str!("alert.css"),
    include_str!("alert_panel.css"),
    include_str!("toast.css"),
    include_str!("progress_ring.css"),
    include_str!("bar_chart.css"),
    include_str!("keyboard_shortcuts.css"),
    include_str!("theme_toggle.css"),
    include_str!("splash_screen.css"),
    include_str!("svg_bar_chart.css"),
    include_str!("svg_line_chart.css"),
    include_str!("sparkline.css"),
    include_str!("feed.css"),
    include_str!("file_tree.css"),
    include_str!("empty_state.css"),
    include_str!("panel.css"),
);

/// Light-theme CSS variables only (no component styles).
/// Use this when you want light theme without the full arniko component stylesheet.
pub const ARNIKO_LIGHT_THEME: &str = include_str!("theme_light.css");
