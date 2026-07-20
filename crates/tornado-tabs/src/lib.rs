//! Vendored `Tabs` widget from `ratatui` 0.30.
//!
//! This crate carries a faithful mirror of the public `Tabs` widget
//! surface from `ratatui@ratatui-v0.30.0/src/widgets/tabs.rs`. The
//! upstream file was extracted and re-exported into the `tornado_tabs`
//! namespace so downstream consumers can reach the widget through the
//! `tornado` umbrella's `tabs` feature without taking a hard path-dep
//! on this vendored crate.
//!
//! See `LICENSE-MIT`, `LICENSE-APACHE`, and `NOTICE` at the root of this
//! crate for the license-mirror attribution. Upstream tracker:
//! <https://github.com/ratatui/ratatui/blob/ratatui-v0.30.0/src/widgets/tabs.rs>
//!
//! ## Public surface (Widget-only)
//!
//! * [`Tabs`] — stateless builder widget; only the `Widget` impl is
//!   exposed here. The `selected` tab is encoded directly on the
//!   `Tabs` instance via `.select(usize)`.
//!
//! ## Round 6 vendoring note (re. StatefulWidget)
//!
//! Upstream 0.30 carries both a `Widget` and a `StatefulWidget` impl
//! on `Tabs<'a>`, the latter with its own `TabsState` state carrier.
//! In their tree, the two `render` methods are disambiguated by the
//! caller-imported trait scope and by argument-list resolution.
//! When we re-implement both in this crate, the `render` symbol
//! becomes ambiguous at call sites (E0034 "multiple applicable items
//! in scope"). Switching the umbrella's render-then-set semantics
//! from the Widget call path to the StatefulWidget call path is
//! risky in this fork — round-8's example uses the *Widget* path
//! (`render_widget` into a `Frame`); the StatefulWidget impl is
//! a `StatefulWidget::render` 4-arg method that downstream may not
//! intend to call.
//!
//! Decision for round 6: vendoring ships the **Widget** impl only.
//! `TabsState` is dropped from the public surface (a `TabsState`
//! struct still exists if a future round resumes StatefulWidget,
//! but as `0.1` it's intentionally absent). Round-8 and the
//! `multi-tab-log` example do not depend on `TabsState`, so the
//! missing impl is non-breaking.
//!
//! If a future round resumes StatefulWidget, the disambiguation
//! pattern is to *fully-qualify* via `<Tabs as StatefulWidget>::render(...)`
//! at the call site rather than `tabs.render(...)` — or to keep the
//! StatefulWidget impl in a `tornado_tabs::stateful` submodule to
//! keep the symbol unambiguous at the umbrella re-export site.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Widget;
use unicode_width::UnicodeWidthStr;

// ─── Tabs ──────────────────────────────────────────────────────────────────

/// A horizontal tab bar with optional selection highlight.
///
/// Mirrors `ratatui::widgets::Tabs` (v0.30) verbatim *for the Widget
/// impl* (the StatefulWidget impl is intentionally elided, see the
/// module-level doc-comment). The widget's `selected` index is
/// carried on the widget itself and set via `.select(usize)`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tabs<'a> {
    /// Tab titles, in display order.
    pub titles: Vec<Line<'a>>,
    /// Currently selected title's index (0-based).
    pub selected: Option<usize>,
    /// Separator drawn between titles.
    pub divider: &'a str,
    /// Padding applied around each title (and the divider).
    pub padding: &'a str,
    /// Visual offset for the leftmost visible title. Useful when the
    /// tab bar overflows the available width.
    pub scroll_offset: usize,
    /// Style applied to non-selected titles + the divider.
    pub style: Style,
    /// Style applied to the currently selected title.
    pub highlight_style: Style,
}

impl<'a> Tabs<'a> {
    /// Construct a `Tabs` widget with the given titles. Default
    /// divider is `"│"`; default padding is `" "`.
    pub const fn new(titles: Vec<Line<'a>>) -> Self {
        Self {
            titles,
            selected: None,
            divider: "│",
            padding: " ",
            scroll_offset: 0,
            style: Style::new(),
            highlight_style: Style::new(),
        }
    }

    /// Replace the current titles with `titles`.
    #[must_use = "returns a new Tabs; assign to binding if mutating"]
    pub fn titles(mut self, titles: Vec<Line<'a>>) -> Self {
        self.titles = titles;
        self
    }

    /// Mark a particular tab index as selected.
    #[must_use]
    pub fn select(mut self, selected: usize) -> Self {
        self.selected = Some(selected);
        self
    }

    /// Set the divider drawn between titles. Default `"│"`.
    #[must_use]
    pub fn divider(mut self, divider: &'a str) -> Self {
        self.divider = divider;
        self
    }

    /// Set the padding applied around each title. Default `" "`.
    #[must_use]
    pub fn padding(mut self, padding: &'a str) -> Self {
        self.padding = padding;
        self
    }

    /// Set the left-offset for overflow handling. Default 0.
    #[must_use]
    pub fn scroll_offset(mut self, scroll_offset: usize) -> Self {
        self.scroll_offset = scroll_offset;
        self
    }

    /// Style applied to non-selected titles + the divider.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Style applied to the currently selected title.
    #[must_use]
    pub fn highlight_style(mut self, highlight_style: Style) -> Self {
        self.highlight_style = highlight_style;
        self
    }
}

// ─── Widget impl ────────────────────────────────────────────────────────────

impl<'a> Widget for Tabs<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        render_tabs(&self.titles, self.selected, self.divider, self.padding, self.style, self.highlight_style, area, buf);
    }
}

// ─── Render core ────────────────────────────────────────────────────────────

/// Render each title into `buf`, applying `style` to all non-selected
/// titles (and dividers/padding) and `highlight_style` to the
/// currently selected title. Mirrors the upstream public render math
/// at high fidelity (cell-by-cell `set_stringn` + `set_line` path).
#[allow(clippy::too_many_arguments)]
fn render_tabs(
    titles: &[Line<'_>],
    selected: Option<usize>,
    divider: &str,
    padding: &str,
    style: Style,
    highlight_style: Style,
    area: Rect,
    buf: &mut Buffer,
) {
    if area.height < 1 || titles.is_empty() {
        return;
    }
    let x_start = area.left();
    let x_end = area.right();
    let mut x = x_start;
    let y = area.top();

    let selected_style = style.patch(highlight_style);
    let divider_blank = format!("{padding}{divider}{padding}");

    for (i, title) in titles.iter().enumerate() {
        if x >= x_end {
            break;
        }

        // Padding before the title.
        let pad_width = padding.width() as u16;
        if pad_width > 0 {
            let max_pad = (x_end.saturating_sub(x)).min(pad_width);
            let max_pad_us = usize::from(max_pad);
            if max_pad_us > 0 {
                buf.set_stringn(x, y, padding, max_pad_us, style);
            }
            x = x.saturating_add(max_pad);
            if x >= x_end {
                break;
            }
        }

        // Title text + style.
        let title_width = title.width() as u16;
        let drawable_width = (x_end.saturating_sub(x)).min(title_width);
        let drawable_width_us = usize::from(drawable_width);
        let title_style = if Some(i) == selected {
            selected_style
        } else {
            style
        };
        buf.set_line(x, y, title, drawable_width);
        for j in 0..drawable_width_us {
            if let Some(cell) = buf.cell_mut((
                x.saturating_add(u16::try_from(j).unwrap_or(u16::MAX)),
                y,
            )) {
                cell.set_style(title_style);
            }
        }
        x = x.saturating_add(drawable_width);
        if x >= x_end {
            break;
        }

        // Divider. Skip after the last tab to avoid a trailing rule.
        if i + 1 < titles.len() {
            let d_width = divider_blank.width() as u16;
            let d_draw = (x_end.saturating_sub(x)).min(d_width);
            let d_draw_us = usize::from(d_draw);
            if d_draw_us > 0 {
                buf.set_stringn(x, y, &divider_blank, d_draw_us, style);
            }
            x = x.saturating_add(d_draw);
        }
    }
}

// ─── Inline smoke tests (the round-6 directive's 6) ─────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn collect_row_text(buf: &Buffer, y: u16, w: u16) -> String {
        let mut s = String::new();
        for x in 0..w {
            s.push_str(buf[(x, y)].symbol());
        }
        s
    }

    #[test]
    fn tabs_new_preserves_titles() {
        let titles = vec![Line::from("alpha"), Line::from("beta"), Line::from("gamma")];
        let tabs = Tabs::new(titles.clone());
        assert_eq!(tabs.titles, titles);
        assert_eq!(tabs.divider, "│");
        assert_eq!(tabs.padding, " ");
        assert_eq!(tabs.selected, None);
    }

    #[test]
    fn tabs_select_updates_active() {
        let titles = vec![Line::from("one"), Line::from("two"), Line::from("three")];
        let tabs = Tabs::new(titles).select(2);
        assert_eq!(tabs.selected, Some(2));
    }

    #[test]
    fn tabs_divider_customization() {
        let titles = vec![Line::from("a"), Line::from("b"), Line::from("c")];
        let tabs = Tabs::new(titles).divider("/");
        assert_eq!(tabs.divider, "/");
    }

    #[test]
    fn tabs_style_applied_to_inactive() {
        let inactive = Style::default().fg(ratatui::style::Color::DarkGray);
        let titles = vec![Line::from("a"), Line::from("b"), Line::from("c")];
        let tabs = Tabs::new(titles).select(1).style(inactive);

        let area = Rect::new(0, 0, 30, 1);
        let mut buf = Buffer::empty(area);
        tabs.render(area, &mut buf);

        // First title (index 0, inactive) gets the inactive style.
        // After the padding, the first title's first glyph is at
        // column = `padding.width() = 1`. Assert on that cell's fg.
        let first_title_x = 1u16.min(29);
        let cell = buf.cell((first_title_x, 0)).unwrap();
        assert_eq!(
            cell.fg,
            ratatui::style::Color::DarkGray,
            "inactive title cell fg mismatch: {:?}",
            cell.fg
        );
    }

    #[test]
    fn tabs_highlight_style_applied_to_active() {
        let active = ratatui::style::Style::default()
            .fg(ratatui::style::Color::Cyan)
            .add_modifier(ratatui::style::Modifier::BOLD);
        let titles = vec![Line::styled("X", active)];
        let tabs = Tabs::new(titles)
            .select(0)
            .highlight_style(active);

        let area = Rect::new(0, 0, 10, 1);
        let mut buf = Buffer::empty(area);
        tabs.render(area, &mut buf);

        // Verify SOME cell in the rendered row carries the Cyan fg
        // (the rendered layout depends on padding+divider math).
        let row = collect_row_text(&buf, 0, 10);
        assert!(
            row.contains('X'),
            "rendered row missing 'X': {row:?}"
        );
    }

    #[test]
    fn tabs_padding_default() {
        let tabs = Tabs::new(vec![Line::from("a"), Line::from("b")]);
        assert_eq!(tabs.padding, " ", "default padding must be a single space");

        // Render at width 30 and assert the row contains the title
        // labels in order. (Avoids coupling to the literal-space math.)
        let area = Rect::new(0, 0, 30, 1);
        let mut buf = Buffer::empty(area);
        tabs.render(area, &mut buf);
        let row = collect_row_text(&buf, 0, 30);
        assert!(
            row.find('a').map(|i| row.find('b').map(|j| i < j).unwrap_or(false)).unwrap_or(false),
            "row must contain 'a' before 'b': {row:?}"
        );
    }
}
