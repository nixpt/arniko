//! # Vendored `BigText` + `PixelSize` — from `joshka/tui-big-text` (MIT/Apache-2.0)
//!
//! ## (1) Provenance + license attribution
//!
//! This crate carries a faithful mirror of the public `BigText` widget
//! and `PixelSize` from upstream `joshka/tui-big-text` v0.4.5. Upstream:
//! <https://github.com/joshka/tui-big-text>.
//!
//! Original upstream `tui-big-text` is dual-licensed under MIT OR Apache-2.0 —
//! see `LICENSE-MIT`, `LICENSE-APACHE`, and `NOTICE` at the root of this
//! crate for the attribution.
//!
//! ## (2) Adaptations from upstream
//!
//! Compared to upstream `tui-big-text` v0.4.5:
//!
//! - **No `derive_builder` proc macro**: Upstream uses `derive_builder` to
//!   generate `BigTextBuilder`. This vendored mirror provides manual builder
//!   methods on `BigText` directly.
//! - **No `itertools` dep**: All iteration uses standard library methods.
//! - **Ratatui 0.30**: Updated from upstream's `ratatui 0.27` to workspace
//!   `ratatui 0.30`.
//!
//! ## (3) Public surface
//!
//! * [`BigText`] — A widget that displays one or more lines of text using
//!   8×8 pixel characters (via the `font8x8` crate). The size of each pixel
//!   can be controlled via [`PixelSize`].
//! * [`PixelSize`] — Controls how many character cells represent a single
//!   pixel of the 8×8 font: `Full`, `HalfHeight`, `HalfWidth`, `Quadrant`,
//!   `ThirdHeight`, or `Sextant`.

use std::cmp::min;

use font8x8::UnicodeFonts;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Line, Style};
use ratatui::text::StyledGrapheme;
use ratatui::widgets::Widget;

// ─── `PixelSize` — glyph scaling ──────────────────────────────────────────

/// Controls how many character cells are used to represent a single pixel of
/// the 8×8 font.
///
/// | Variant | Pixels per cell (H×V) | Typical use |
/// |---|---|---|
/// | `Full` | 1×1 | Clear blocky text (default) |
/// | `HalfHeight` | 1×2 | Compact vertical, half-height rows |
/// | `HalfWidth` | 2×1 | Compact horizontal, half-width columns |
/// | `Quadrant` | 2×2 | Dense text with quadrant block chars |
/// | `ThirdHeight` | 1×3 | Very dense vertical (unicode support needed) |
/// | `Sextant` | 2×3 | Densest, 6 sub-pixels per cell |
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum PixelSize {
    /// Each pixel from the 8×8 font occupies one full character cell.
    #[default]
    Full,
    /// Each pixel occupies the upper or lower half of a character cell.
    HalfHeight,
    /// Each pixel occupies the left or right half of a character cell.
    HalfWidth,
    /// Each pixel occupies one quadrant of a character cell (2×2 per cell).
    Quadrant,
    /// Each pixel occupies one third of a character cell (1×3 per cell).
    ThirdHeight,
    /// Each pixel occupies one sextant of a character cell (2×3 per cell).
    Sextant,
}

impl PixelSize {
    /// Returns the number of font pixels per character cell as `(horizontal, vertical)`.
    const fn pixels_per_cell(self) -> (u16, u16) {
        match self {
            PixelSize::Full => (1, 1),
            PixelSize::HalfHeight => (1, 2),
            PixelSize::HalfWidth => (2, 1),
            PixelSize::Quadrant => (2, 2),
            PixelSize::ThirdHeight => (1, 3),
            PixelSize::Sextant => (2, 3),
        }
    }

    /// Returns the single character that represents the pixels at `(row, col)` in the glyph,
    /// given this pixel-size resolution.
    fn symbol_for_position(self, glyph: &[u8; 8], row: usize, col: i32) -> char {
        match self {
            PixelSize::Full => match glyph[row] & (1 << col) {
                0 => ' ',
                _ => '█',
            },
            PixelSize::HalfHeight => {
                let top = glyph[row] & (1 << col);
                let bottom = glyph[row + 1] & (1 << col);
                symbol_half_height(top, bottom)
            }
            PixelSize::HalfWidth => {
                let left = glyph[row] & (1 << col);
                let right = glyph[row] & (1 << (col + 1));
                symbol_half_width(left, right)
            }
            PixelSize::Quadrant => {
                let top_left = glyph[row] & (1 << col);
                let top_right = glyph[row] & (1 << (col + 1));
                let bottom_left = glyph[row + 1] & (1 << col);
                let bottom_right = glyph[row + 1] & (1 << (col + 1));
                symbol_quadrant(top_left, top_right, bottom_left, bottom_right)
            }
            PixelSize::ThirdHeight => {
                let top = glyph[row] & (1 << col);
                let middle = if row + 1 < glyph.len() {
                    glyph[row + 1] & (1 << col)
                } else {
                    0
                };
                let bottom = if row + 2 < glyph.len() {
                    glyph[row + 2] & (1 << col)
                } else {
                    0
                };
                symbol_third_height(top, middle, bottom)
            }
            PixelSize::Sextant => {
                let top_left = glyph[row] & (1 << col);
                let top_right = glyph[row] & (1 << (col + 1));
                let (middle_left, middle_right) = if row + 1 < glyph.len() {
                    (
                        glyph[row + 1] & (1 << col),
                        glyph[row + 1] & (1 << (col + 1)),
                    )
                } else {
                    (0, 0)
                };
                let (bottom_left, bottom_right) = if row + 2 < glyph.len() {
                    (
                        glyph[row + 2] & (1 << col),
                        glyph[row + 2] & (1 << (col + 1)),
                    )
                } else {
                    (0, 0)
                };
                symbol_sextant(
                    top_left,
                    top_right,
                    middle_left,
                    middle_right,
                    bottom_left,
                    bottom_right,
                )
            }
        }
    }
}

// ─── `BigText` — builder widget ───────────────────────────────────────────

/// Displays one or more lines of text using 8×8 pixel characters.
///
/// Uses the [`font8x8`] crate to render each character as a pixel-glyph.
/// The [`PixelSize`] controls how large each pixel appears.
///
/// # Example
///
/// ```rust
/// use ratatui::prelude::*;
/// use tornado_big_text::{BigText, PixelSize};
///
/// let big = BigText::new(vec![
///     Line::from("Hello".red()),
///     Line::from("World".blue()),
/// ])
/// .pixel_size(PixelSize::Full)
/// .style(Style::new().white());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BigText<'a> {
    /// The lines of text to display in large form.
    pub lines: Vec<Line<'a>>,
    /// Base style applied to the entire widget (overridable per-line).
    pub style: Style,
    /// How many terminal cells per font pixel (default: `Full`).
    pub pixel_size: PixelSize,
    /// Horizontal alignment of the text within the render area.
    pub alignment: Alignment,
}

impl<'a> BigText<'a> {
    /// Create a new `BigText` widget with the given lines.
    ///
    /// Accepts any type that can be converted into `Vec<Line<'a>>`.
    pub fn new(lines: impl Into<Vec<Line<'a>>>) -> Self {
        Self {
            lines: lines.into(),
            style: Style::default(),
            pixel_size: PixelSize::default(),
            alignment: Alignment::default(),
        }
    }

    /// Set the base style for the entire widget.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the pixel size (how many terminal cells per font pixel).
    #[must_use]
    pub fn pixel_size(mut self, pixel_size: PixelSize) -> Self {
        self.pixel_size = pixel_size;
        self
    }

    /// Set the horizontal alignment of the text.
    #[must_use]
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}

// ─── `Widget for BigText` — render ────────────────────────────────────────

impl Widget for BigText<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (step_x, step_y) = self.pixel_size.pixels_per_cell();
        let glyph_width = 8_u16.div_ceil(step_x);
        let glyph_height = 8_u16.div_ceil(step_y);

        for (line_idx, y) in (area.top()..area.bottom())
            .step_by(glyph_height as usize)
            .enumerate()
        {
            if line_idx >= self.lines.len() {
                break;
            }
            let line = &self.lines[line_idx];
            let big_line_width = line.width() as u16 * glyph_width;
            let offset = match self.alignment {
                Alignment::Center => (area.width / 2).saturating_sub(big_line_width / 2),
                Alignment::Right => area.width.saturating_sub(big_line_width),
                Alignment::Left => 0,
            };

            // Collect graphemes once per line to avoid recomputing for each char.
            let graphemes: Vec<_> = line.styled_graphemes(self.style).collect();

            for (char_idx, x) in (area.left() + offset..area.right())
                .step_by(glyph_width as usize)
                .enumerate()
            {
                if char_idx >= graphemes.len() {
                    break;
                }
                let cell_w = min(area.right().saturating_sub(x), glyph_width);
                let cell_h = min(area.bottom().saturating_sub(y), glyph_height);
                let cell_area = Rect::new(x, y, cell_w, cell_h);

                render_symbol(&graphemes[char_idx], cell_area, buf, &self.pixel_size);
            }
        }
    }
}

// ─── Symbol rendering helpers ─────────────────────────────────────────────

/// Render a single grapheme into a cell by looking up the corresponding 8×8
/// bitmap and setting the buffer cells.
fn render_symbol(grapheme: &StyledGrapheme, area: Rect, buf: &mut Buffer, pixel_size: &PixelSize) {
    buf.set_style(area, grapheme.style);
    let c = grapheme.symbol.chars().next().unwrap_or(' ');
    if let Some(glyph) = font8x8::BASIC_FONTS.get(c) {
        render_glyph(glyph, area, buf, pixel_size);
    }
}

/// Render a single 8×8 glyph into the given area.
fn render_glyph(glyph: [u8; 8], area: Rect, buf: &mut Buffer, pixel_size: &PixelSize) {
    let (step_x, step_y) = pixel_size.pixels_per_cell();

    let mut y = area.top();
    for row in (0..glyph.len()).step_by(step_y as usize) {
        let mut x = area.left();
        for col in (0..8).step_by(step_x as usize) {
            if x < area.right() && y < area.bottom() {
                let cell = buf.get_mut(x, y);
                let symbol = pixel_size.symbol_for_position(&glyph, row, col as i32);
                cell.set_char(symbol);
            }
            x = x.saturating_add(1);
        }
        y = y.saturating_add(1);
    }
}

// ─── Low-level unicode symbol selectors ───────────────────────────────────

fn symbol_half_height(top: u8, bottom: u8) -> char {
    match (top != 0, bottom != 0) {
        (false, false) => ' ',
        (false, true) => '▄',
        (true, false) => '▀',
        (true, true) => '█',
    }
}

fn symbol_half_width(left: u8, right: u8) -> char {
    match (left != 0, right != 0) {
        (false, false) => ' ',
        (false, true) => '▐',
        (true, false) => '▌',
        (true, true) => '█',
    }
}

fn symbol_quadrant(top_left: u8, top_right: u8, bottom_left: u8, bottom_right: u8) -> char {
    let idx = (top_left != 0) as usize
        | ((top_right != 0) as usize) << 1
        | ((bottom_left != 0) as usize) << 2
        | ((bottom_right != 0) as usize) << 3;

    const QUADRANT_SYMBOLS: [char; 16] = [
        ' ', '▘', '▝', '▀', '▖', '▌', '▞', '▛', '▗', '▚', '▐', '▜', '▄', '▙', '▟', '█',
    ];
    QUADRANT_SYMBOLS[idx]
}

fn symbol_third_height(top: u8, middle: u8, bottom: u8) -> char {
    let idx = (top != 0) as usize
        | ((middle != 0) as usize) << 1
        | ((bottom != 0) as usize) << 2;
    const THIRD_SYMBOLS: [char; 8] = [' ', '🬂', '🬋', '🬎', '🬭', '🬰', '🬹', '█'];
    THIRD_SYMBOLS[idx]
}

fn symbol_sextant(
    top_left: u8, top_right: u8,
    middle_left: u8, middle_right: u8,
    bottom_left: u8, bottom_right: u8,
) -> char {
    let idx = (top_left != 0) as usize
        | ((top_right != 0) as usize) << 1
        | ((middle_left != 0) as usize) << 2
        | ((middle_right != 0) as usize) << 3
        | ((bottom_left != 0) as usize) << 4
        | ((bottom_right != 0) as usize) << 5;

    const SEXTANT_SYMBOLS: [char; 64] = [
        ' ', '🬀', '🬁', '🬂', '🬃', '🬄', '🬅', '🬆',
        '🬇', '🬈', '🬉', '🬊', '🬋', '🬌', '🬍', '🬎',
        '🬏', '🬐', '🬑', '🬒', '🬓', '▌', '🬔', '🬕',
        '🬖', '🬗', '🬘', '🬙', '🬚', '🬛', '🬜', '🬝',
        '🬞', '🬟', '🬠', '🬡', '🬢', '🬣', '🬤', '🬥',
        '🬦', '🬧', '▐', '🬨', '🬩', '🬪', '🬫', '🬬',
        '🬭', '🬮', '🬯', '🬰', '🬱', '🬲', '🬳', '🬴',
        '🬵', '🬶', '🬷', '🬸', '🬹', '🬺', '🬻', '█',
    ];
    SEXTANT_SYMBOLS[idx]
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::{Color, Style};

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[test]
    fn big_text_new_defaults() {
        let bt = BigText::new(vec![Line::from("Hello")]);
        assert_eq!(bt.lines.len(), 1);
        assert_eq!(bt.pixel_size, PixelSize::Full);
        assert_eq!(bt.style, Style::default());
    }

    #[test]
    fn pixel_size_symbols() {
        // Full: solid pixel → '█', no pixel → ' '
        let glyph = [0xFFu8; 8];
        assert_eq!(PixelSize::Full.symbol_for_position(&glyph, 0, 0), '█');
        let empty = [0x00u8; 8];
        assert_eq!(PixelSize::Full.symbol_for_position(&empty, 0, 0), ' ');

        // Quadrant: all 4 pixels → '█'
        assert_eq!(PixelSize::Quadrant.symbol_for_position(&glyph, 0, 0), '█');

    // Quadrant: top-left only (bit 0 set in the top row)
    let mut one_pixel = [0x00u8; 8];
    one_pixel[0] = 0x01; // top row, bit 0
    assert_eq!(PixelSize::Quadrant.symbol_for_position(&one_pixel, 0, 0), '▘');
    }

    #[test]
    fn render_single_line() -> Result<()> {
        // A minimal render test — check that it doesn't panic and produces
        // something at row 0.
        let big_text = BigText::new(vec![Line::from("X")]);
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 8));
        big_text.render(buf.area, &mut buf);
        let row0: String = (0..10).map(|x| buf[(x, 0)].symbol().to_string()).collect();
        // With PixelSize::Full, the 'X' glyph should draw at least some blocks
        assert!(row0.contains('█'), "row0 should contain block chars: {row0:?}");
        Ok(())
    }

    #[test]
    fn render_multiple_lines() -> Result<()> {
        let big_text = BigText::new(vec![Line::from("A"), Line::from("B")]);
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 16));
        big_text.render(buf.area, &mut buf);
        // Both lines should render (no panic)
        let row0: String = (0..10).map(|x| buf[(x, 0)].symbol().to_string()).collect();
        let row8: String = (0..10).map(|x| buf[(x, 8)].symbol().to_string()).collect();
        assert!(row0.contains('█'), "first line should render: {row0:?}");
        assert!(row8.contains('█'), "second line should render: {row8:?}");
        Ok(())
    }

    #[test]
    fn style_applies() -> Result<()> {
        let big_text = BigText::new(vec![Line::from("A")])
            .style(Style::new().fg(Color::Red));
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 8));
        big_text.render(buf.area, &mut buf);
        let cell = buf.cell((0, 0)).unwrap();
        assert_eq!(cell.style().fg, Some(Color::Red));
        Ok(())
    }
}
