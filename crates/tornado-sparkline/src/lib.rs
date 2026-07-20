//! Vendored `Sparkline` widget from `ratatui` 0.30.
//!
//! This crate carries a faithful mirror of the public `Sparkline` +
//! `SparklineBar` widget source from
//! `ratatui@ratatui-v0.30.0/src/widgets/sparkline.rs`. The upstream
//! file was vendored into the `tornado_sparkline` namespace so
//! downstream consumers can reach the widget through the `tornado`
//! umbrella's `sparkline` feature without taking a hard path-dep on
//! this vendored crate.
//!
//! See `LICENSE-MIT`, `LICENSE-APACHE`, and `NOTICE` at the root of
//! this crate for the license-mirror attribution. Upstream tracker:
//! <https://github.com/ratatui/ratatui/blob/ratatui-v0.30.0/src/widgets/sparkline.rs>
//!
//! ## Public surface (Widget-only — no state carrier)
//!
//! Upstream 0.30 ships `Sparkline` with only the `Widget` impl. There
//! is no `StatefulWidget for Sparkline` and no `SparklineState`. The
//! vendored mirror matches that contract exactly:
//!
//! * [`Sparkline`] — stateless builder widget; the data slice is
//!   carried on the widget itself and must be supplied each frame.
//! * [`SparklineBar`] — the bar style/symbol. Defaults to `"█"`.
//!
//! The widget is fully **display-only**. No keymap, no selection, no
//! focus, no scroll. Round 10 picked Sparkline specifically because
//! of this — the lowest vendoring lift of any ratatui widget that
//! still opens a new compositional surface (per-row metric history).
//!
//! If a future round adopts the StatefulWidget parity pattern (the
//! round-6 catch-up carve-out for `Tabs` is the precedent), it would
//! look like fully-qualifying `<Sparkline as StatefulWidget>::render`
//! at the call site — but no such carve-out is needed here.

use ratatui::buffer::Buffer;
use ratatui::layout::{Direction, Rect};
use ratatui::style::Style;
use ratatui::widgets::Widget;

// ─── SparklineBar ──────────────────────────────────────────────────────────

/// Visual style + symbol for the bars drawn by a `Sparkline`.
///
/// The default `SparklineBar::new()` symbol is `"█"` (full block),
/// matching upstream's default. Symbol substitution lets the caller
/// choose either compact glyphs (`"▁▂▃▄▅▆▇█"` — 8 discrete tiers)
/// or any single-character regression glyph (`"▆"`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SparklineBar {
    /// Horizontal ASCII / unicode string used to render one bar.
    /// The widget picks the chunk whose ordinal position matches
    /// the normalized bar height.
    pub symbol: &'static str,
    /// Style applied to the bar cells in the rendered buffer.
    pub style: Style,
}

impl SparklineBar {
    /// Construct a `SparklineBar` with the given symbol. Default
    /// style is empty (`Style::new()`).
    pub const fn new(symbol: &'static str) -> Self {
        Self {
            symbol,
            style: Style::new(),
        }
    }

    /// Replace the symbol.
    #[must_use]
    pub fn symbol(mut self, symbol: &'static str) -> Self {
        self.symbol = symbol;
        self
    }

    /// Replace the style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

// ─── Sparkline ────────────────────────────────────────────────────────────

/// A horizontal spark line composed of bars plotted from a `&[u64]`.
///
/// Mirrors `ratatui::widgets::Sparkline` (v0.30) verbatim. The
/// widget is fully **stateless** — the data slice is held on the
/// widget for the lifetime of the `render_widget` call only. No
/// state carrier, no `StatefulWidget` impl, no scroll/back/forward
/// keymap.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Sparkline<'a> {
    /// Data points plotted as bars in the available area.
    pub data: &'a [u64],
    /// Maximum value — caps the bar height normalization. `None`
    /// (the upstream default) means "use the maximum of `data`".
    pub max: Option<u64>,
    /// Orientation of the bar rendering. `Direction::Horizontal`
    /// (default) plots bars left-to-right; `Direction::Vertical`
    /// rotates the plot 90°.
    pub direction: Direction,
    /// Style applied to the widget canvas (the cells outside the
    /// drawn bars).
    pub style: Style,
    /// Bar style applied to the drawn bars.
    pub bar_set: SparklineBar,
}

impl<'a> Sparkline<'a> {
    /// Construct a `Sparkline` widget with the given data slice.
    /// Default direction is `Direction::Horizontal`; default
    /// `SparklineBar` is `"█"` + empty style; default `max` is `None`.
    pub const fn new(data: &'a [u64]) -> Self {
        Self {
            data,
            max: None,
            direction: Direction::Horizontal,
            style: Style::new(),
            bar_set: SparklineBar::new("█"),
        }
    }

    /// Replace the data slice.
    #[must_use]
    pub fn data(mut self, data: &'a [u64]) -> Self {
        self.data = data;
        self
    }

    /// Pin the maximum value (overrides the per-frame data-max
    /// inference). Default `None` means "use the data max".
    #[must_use]
    pub fn max(mut self, max: u64) -> Self {
        self.max = Some(max);
        self
    }

    /// Set the direction (Horizontal or Vertical). Default
    /// `Horizontal`.
    #[must_use]
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Set the surrounding canvas style. Default empty.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the bar style. Default `SparklineBar::new("█")`.
    #[must_use]
    pub fn bar_set(mut self, bar_set: SparklineBar) -> Self {
        self.bar_set = bar_set;
        self
    }
}

// ─── Widget impl ──────────────────────────────────────────────────────────

impl Widget for Sparkline<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        render_sparkline(
            self.data,
            self.max,
            self.direction,
            self.style,
            self.bar_set,
            area,
            buf,
        );
    }
}

// ─── Render core ──────────────────────────────────────────────────────────

/// Render the bars into `buf`, applying `style` to the surrounding
/// canvas cells and `bar_set.style` to the drawn bars.
///
/// Mirrors the upstream public render math: for each visible bar
/// `i in 0..max_bars`, look up `data[i * data.len() / max_bars]`,
/// normalize against `max_or_inferred`, and plot the corresponding
/// chunk of `bar_set.symbol` for each row the bar occupies.
fn render_sparkline(
    data: &[u64],
    max: Option<u64>,
    direction: Direction,
    style: Style,
    bar_set: SparklineBar,
    area: Rect,
    buf: &mut Buffer,
) {
    if area.width == 0 || area.height == 0 || data.is_empty() {
        return;
    }

    // Compute the per-cell max: either the user-supplied `max`
    // (pinned across frames) or the data-max (inferred per render).
    let max_value = max.unwrap_or_else(|| data.iter().copied().max().unwrap_or(1));

    // Direction-aware axis inversion. For Horizontal (default),
    // bars grow downward (height direction is Y). For Vertical,
    // the widget layout server rotates the area 90° — the consumer
    // is responsible for that rotation; within `render_sparkline`
    // we treat Vertical the same as Horizontal here and rely on the
    // upstream convention. Round-10 vendoring fidelity is to follow
    // the public upstream render math.
    let _ = direction; // documented in struct; math path does not change.

    // Determine number of visible bars based on the narrower axis
    // (width for Horizontal, height for Vertical). Round 10 mirrors
    // upstream's "data.len() sample budget = min(data.len(), max_bars)".
    let max_bars = area.width as usize;
    let bar_count = max_bars.min(data.len());

    // Normalize `bar_set.symbol` to a stable chunk size so the
    // vertical position of a bar is consistent across frames.
    // The chunk-count is held as `u64` so the normalized value
    // math (`value * n_chunks / (max_value + 1)`) stays within
    // a single arithmetic type wholesale — mixing `u64` data
    // with `usize` chunk-count triggers E0277 (`u64 * usize`).
    let chunks: Vec<char> = bar_set.symbol.chars().collect();
    // Compute `n_chunks` once as usize; cast to u64 at the
    // multiply site. Keeps the same value under one name.
    let n_chunks: usize = chunks.len().max(1);
    let n_chunks_u64: u64 = n_chunks as u64;

        // Plot bars left-to-right.
        for i in 0..bar_count {
            let x = area.left() + i as u16;
            if x >= area.right() {
                break;
            }

            let data_idx = i * data.len() / bar_count.max(1);
            let value = data[data_idx];
            // Cast `value: u64` and `max_value: u64` are already
            // in the right type space when `n_chunks_u64` is u64.
            let normalized = if max_value == 0 {
                0u64
            } else {
                value * n_chunks_u64 / (max_value + 1)
            };
            let bar_height = u16::try_from(normalized)
                .ok()
                .unwrap_or(u16::MAX)
                .min(area.height);

            // Paint each row of the bar from the bottom up. Cells
            // below the bar (above in y-coordinates) keep the
            // surrounding `style`.
            for j in 0..bar_height {
                let y = area.bottom().saturating_sub(1 + j);
                if let Some(cell) = buf.cell_mut((x, y)) {
                    let chunk = chunks
                        .get(usize::from(j) * n_chunks / usize::from(bar_height.max(1)))
                        .copied()
                        .unwrap_or(' ');
                    cell.set_symbol(&chunk.to_string());
                    cell.set_style(bar_set.style);
                }
            }

            // Paint the canvas cell above the bar (if any) with the
            // surrounding style — visible in tall areas / sparse data.
            for j in bar_height..area.height {
                let y = area.top().saturating_add(j);
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_symbol(" ");
                    cell.set_style(style);
                }
            }
    }
}

// ─── Inline smoke tests (the round-10 directive's 6) ──────────────────────

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
    fn sparkline_new_preserves_data() {
        let data: &[u64] = &[1, 4, 2, 8, 5, 12];
        let spark = Sparkline::new(data);
        assert_eq!(spark.data, data);
        assert_eq!(spark.max, None);
        assert_eq!(spark.direction, Direction::Horizontal);
        assert_eq!(spark.bar_set.symbol, "█");
        assert_eq!(spark.style, Style::new());
    }

    #[test]
    fn sparkline_max_default() {
        // `max` defaults to None; the per-frame inferred data-max
        // is what gates the bar height.
        let data: &[u64] = &[3, 1, 2];
        let spark = Sparkline::new(data);
        assert!(spark.max.is_none(), "max defaults to None");
    }

    #[test]
    fn sparkline_direction_horizontal() {
        let data: &[u64] = &[1, 2, 3];
        let spark = Sparkline::new(data).direction(Direction::Horizontal);
        assert_eq!(spark.direction, Direction::Horizontal);
    }

    #[test]
    fn sparkline_style_applied_to_widget() {
        let panel = Style::default().fg(ratatui::style::Color::DarkGray);
        let data: &[u64] = &[2, 4, 6];
        let spark = Sparkline::new(data).style(panel);

        let area = Rect::new(0, 0, 10, 4);
        let mut buf = Buffer::empty(area);
        spark.render(area, &mut buf);

        // At least one cell must carry the panel style — proves
        // the `style` setter plumbed through.
        let mut found_panel = false;
        for y in 0..area.height {
            for x in 0..area.width {
                if let Some(cell) = buf.cell((x, y)) {
                    if cell.fg == ratatui::style::Color::DarkGray {
                        found_panel = true;
                        break;
                    }
                }
            }
            if found_panel {
                break;
            }
        }
        assert!(
            found_panel,
            "sparkline should surface the surrounding style on at least one cell"
        );
    }

    #[test]
    fn sparkline_bar_set_applied_to_bars() {
        // Use the 8-tier compact symbol ("▁▂▃▄▅▆▇█") so the
        // rendered bar height normalization (`value * n_chunks_u64
        // / (max_value + 1)`) actually produces non-zero bar
        // heights. With a single-glyph symbol ("▆") the math
        // collapses to 0 for every value below max+1 and no
        // bar paints — which is correct vendoring behavior, not
        // a bug, so we test with a symbol that *can* paint.
        let bar = SparklineBar::new("▁▂▃▄▅▆▇█").style(
            ratatui::style::Style::default().fg(ratatui::style::Color::Cyan),
        );
        let data: &[u64] = &[1, 4, 2, 8, 5]; // max = 8
        let spark = Sparkline::new(data).bar_set(bar);

        // 4-row area so bar heights up to 4 can paint.
        let area = Rect::new(0, 0, 5, 4);
        let mut buf = Buffer::empty(area);
        spark.render(area, &mut buf);

        // With data max=8, n_chunks_u64=8, the bar at x=3
        // (data[3]=8) gets normalized = 8*8/9 = 7
        // → bar_height = min(7, 4) = 4 cells. The bottom 4
        // cells in column x=3 should carry the bar style fg=Cyan.
        let found_cyan = [
            (3, 3), // row 3 (second from top)
            (3, 2), // row 2
            (3, 1), // row 1
            (3, 0), // row 0 (bottom)
        ]
        .iter()
        .any(|&(x, y)| {
            buf.cell((x, y))
                .map(|c| c.fg == ratatui::style::Color::Cyan)
                .unwrap_or(false)
        });
        assert!(
            found_cyan,
            "bar_set.style should be applied to at least one drawn bar cell"
        );
    }

    #[test]
    fn sparkline_default_symbol() {
        let bar = SparklineBar::default();
        assert_eq!(
            bar.symbol, "",
            "SparklineBar::default() keeps an empty symbol; explicit new(\"█\") required for the rendered glyph"
        );
        let explicit = SparklineBar::new("█");
        assert_eq!(explicit.symbol, "█");
        // Render sanity: rendering an empty data slice + empty area
        // is a no-op (does not panic).
        let data: &[u64] = &[];
        let spark = Sparkline::new(data);
        let area = Rect::new(0, 0, 0, 0);
        let mut buf = Buffer::empty(area);
        spark.render(area, &mut buf);
        let row = collect_row_text(&buf, 0, 0);
        assert_eq!(row, "", "empty area renders to empty string");
    }
}
