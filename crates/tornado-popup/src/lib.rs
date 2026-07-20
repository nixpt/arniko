//! # Vendored `Popup` + `PopupState` — from `joshka/tui-popup` (MIT)
//!
//! ## (1) Provenance + license attribution
//!
//! This crate carries a faithful mirror of the public `Popup` widget
//! + `PopupState` from upstream
//! `joshka/tui-popup` v0.4.2. Upstream:
//! <https://github.com/joshka/tui-popup>.
//!
//! Original upstream `tui-popup` is MIT-licensed — see `LICENSE-MIT` and
//! `NOTICE` at the root of this crate for the attribution.
//!
//! ## (2) Adaptations from upstream
//!
//! Compared to upstream `tui-popup` v0.4.2:
//!
//! - **Body type simplified**: The upstream `Popup<'content, W: SizedWidgetRef>`
//!   is generic over any body widget. This vendored mirror uses
//!   `Popup<'a>` with `body: Text<'a>` instead, because the upstream's
//!   `SizedWidgetRef` trait depends on `WidgetRef` (an unstable ratatui
//!   feature gated behind `unstable-widget-ref`). Using `Text<'a>` avoids
//!   the unstable dependency while supporting the common case (text popups).
//! - **No proc-macro deps**: Upstream uses `derive-getters` and
//!   `derive_setters`. This vendored mirror provides manual builder methods.
//! - **No `crossterm` feature**: The upstream `crossterm` feature
//!   (`PopupState::handle_mouse_event`) is omitted. Consumers can still
//!   call `mouse_down`/`mouse_up`/`mouse_drag` directly.
//! - **Standard `Widget` / `StatefulWidget` traits**: Upstream uses
//!   `WidgetRef` / `StatefulWidgetRef`. This mirror uses the standard
//!   `Widget for &T` / `StatefulWidget for &T` reference-based impls.
//!
//! ## (3) Public surface
//!
//! * [`Popup`] — A simple popup widget with a bordered box, title, and
//!   body text. Implements `Widget for &Popup` (stateless) and
//!   `StatefulWidget for &Popup` (stateful, with `PopupState` for
//!   position tracking and dragging).
//! * [`PopupState`] — State carrier for the popup. Tracks the last
//!   rendered area and optional drag state for mouse-move support.
//! * [`DragState`] — Whether the popup is currently being dragged.

use std::cmp::min;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Style};
use ratatui::symbols::border::Set;
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Clear, StatefulWidget, Widget};

// ─── `Popup` — builder widget ─────────────────────────────────────────────

/// A simple popup widget with a bordered box, title, and body text.
///
/// The popup automatically sizes itself based on the body text dimensions
/// and centers itself within the render area.
///
/// # Example
///
/// ```rust
/// use ratatui::prelude::*;
/// use tornado_popup::Popup;
///
/// let popup = Popup::new("Press any key to exit")
///     .title("demo")
///     .style(Style::new().white().on_blue());
/// ```
#[derive(Debug, Clone)]
pub struct Popup<'a> {
    /// The body text (the main content displayed inside the popup).
    pub body: Text<'a>,
    /// Optional title rendered in the top border.
    pub title: Line<'a>,
    /// Style applied to the entire popup (fill + border).
    pub style: Style,
    /// Which borders to draw (default: `Borders::ALL`).
    pub borders: Borders,
    /// The symbols used to render the border lines.
    pub border_set: Set<'a>,
    /// Style applied to the border lines.
    pub border_style: Style,
}

impl<'a> Popup<'a> {
    /// Create a new popup with the given body text and all borders.
    ///
    /// Accepts any type that implements `Into<Text<'a>>`, such as `&str`,
    /// `String`, `Line<'a>`, or `Text<'a>`.
    pub fn new(body: impl Into<Text<'a>>) -> Self {
        Self {
            body: body.into(),
            borders: Borders::ALL,
            border_set: Set::default(),
            border_style: Style::default(),
            title: Line::default(),
            style: Style::default(),
        }
    }

    /// Set the title rendered in the top border.
    #[must_use]
    pub fn title(mut self, title: impl Into<Line<'a>>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the style applied to the entire popup.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set which borders to draw.
    #[must_use]
    pub fn borders(mut self, borders: Borders) -> Self {
        self.borders = borders;
        self
    }

    /// Set the symbols used to render the border lines.
    #[must_use]
    pub fn border_set(mut self, border_set: Set<'a>) -> Self {
        self.border_set = border_set;
        self
    }

    /// Set the style applied to the border lines.
    #[must_use]
    pub fn border_style(mut self, border_style: Style) -> Self {
        self.border_style = border_style;
        self
    }
}

// ─── `Widget for &Popup` — stateless render ───────────────────────────────

impl Widget for &Popup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = PopupState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

// ─── `StatefulWidget for &Popup` — stateful render ─────────────────────────

impl StatefulWidget for &Popup<'_> {
    type State = PopupState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = if let Some(next) = state.area.take() {
            // respect a previously-set position, but clamp to screen
            let width = min(next.width, area.width);
            let height = min(next.height, area.height);
            let x = next.x.clamp(buf.area.x, area.right().saturating_sub(width));
            let y = next.y.clamp(buf.area.y, area.bottom().saturating_sub(height));
            Rect::new(x, y, width, height)
        } else {
            // auto-center based on body dimensions
            let border_height = usize::from(self.borders.intersects(Borders::TOP))
                + usize::from(self.borders.intersects(Borders::BOTTOM));
            let border_width = usize::from(self.borders.intersects(Borders::LEFT))
                + usize::from(self.borders.intersects(Borders::RIGHT));

            let height = self
                .body
                .height()
                .saturating_add(border_height)
                .try_into()
                .unwrap_or(area.height);
            let width = self
                .body
                .width()
                .saturating_add(border_width)
                .try_into()
                .unwrap_or(area.width);
            centered_rect(width, height, area)
        };

        state.area.replace(area);

        // Clear the area before rendering the popup
        Clear.render(area, buf);

        // Build the block border + title + style
        let block = Block::default()
            .borders(self.borders)
            .border_set(self.border_set)
            .border_style(self.border_style)
            .title(self.title.clone())
            .style(self.style);
        let inner = block.inner(area);

        // Render the block by reference so we can reuse it for inner area
        Widget::render(&block, area, buf);

        // Render the body text inside the block's inner area
        Widget::render(&self.body, inner, buf);
    }
}

// ─── `PopupState` — selection carrier ──────────────────────────────────────

/// State carrier for a [`Popup`].
///
/// Tracks the last-rendered screen position so the popup can be re-drawn
/// at the same location across frames. Also supports mouse-based drag
/// via [`DragState`].
#[derive(Clone, Debug, Default)]
pub struct PopupState {
    /// The last rendered area of the popup. `None` indicates the popup
    /// has not been rendered yet (will be auto-centered on first render).
    pub area: Option<Rect>,
    /// Current drag state for mouse-based repositioning.
    pub drag_state: DragState,
}

impl PopupState {
    /// Move the popup by the given offset.
    pub fn move_by(&mut self, x: i32, y: i32) {
        if let Some(area) = self.area {
            self.area.replace(Rect {
                x: i32::from(area.x)
                    .saturating_add(x)
                    .try_into()
                    .unwrap_or(area.x),
                y: i32::from(area.y)
                    .saturating_add(y)
                    .try_into()
                    .unwrap_or(area.y),
                ..area
            });
        }
    }

    /// Move the popup to an absolute position.
    pub fn move_to(&mut self, x: u16, y: u16) {
        if let Some(area) = self.area {
            self.area.replace(Rect { x, y, ..area });
        }
    }

    /// Begin dragging when the mouse is pressed inside the popup area.
    pub fn mouse_down(&mut self, col: u16, row: u16) {
        if let Some(area) = self.area {
            if area.contains((col, row).into()) {
                self.drag_state = DragState::Dragging {
                    col_offset: col.saturating_sub(area.x),
                    row_offset: row.saturating_sub(area.y),
                };
            }
        }
    }

    /// End dragging on mouse release.
    pub fn mouse_up(&mut self, _col: u16, _row: u16) {
        self.drag_state = DragState::NotDragging;
    }

    /// Update position while dragging.
    pub fn mouse_drag(&mut self, col: u16, row: u16) {
        if let DragState::Dragging {
            col_offset,
            row_offset,
        } = self.drag_state
        {
            if let Some(area) = self.area {
                let x = col.saturating_sub(col_offset);
                let y = row.saturating_sub(row_offset);
                self.area.replace(Rect { x, y, ..area });
            }
        }
    }

    /// Returns a reference to the last rendered area, if any.
    pub fn area(&self) -> Option<Rect> {
        self.area
    }

    /// Returns a reference to the current drag state.
    pub fn drag_state(&self) -> &DragState {
        &self.drag_state
    }
}

// ─── `DragState` — drag tracking ──────────────────────────────────────────

/// Whether the popup is currently being dragged by the mouse.
///
/// Used by [`PopupState`] to track mouse-initiated repositioning.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum DragState {
    /// The popup is not being dragged.
    #[default]
    NotDragging,
    /// The popup is being dragged; offsets track the cursor anchor.
    Dragging {
        /// Horizontal offset from the popup's left edge to the cursor.
        col_offset: u16,
        /// Vertical offset from the popup's top edge to the cursor.
        row_offset: u16,
    },
}

// ─── Geometry helpers ─────────────────────────────────────────────────────

/// Create a rectangle centered within the given area.
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    Rect {
        x: area.width.saturating_sub(width) / 2,
        y: area.height.saturating_sub(height) / 2,
        width: min(width, area.width),
        height: min(height, area.height),
    }
}

// ─── Smoke tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::{Color, Style};

    fn collect_row(buf: &Buffer, y: u16) -> String {
        let mut s = String::new();
        for x in 0..buf.area.width {
            s.push_str(&buf[(x, y)].symbol().to_string());
        }
        s
    }

    /// 1. Popup construction with default values.
    #[test]
    fn popup_new_defaults() {
        let popup = Popup::new("hello");
        assert_eq!(popup.borders, Borders::ALL);
        assert_eq!(popup.style, Style::default());
        assert!(popup.title.width() == 0);
    }

    /// 2. Popup renders body text.
    #[test]
    fn popup_renders_body_text() {
        let popup = Popup::new("Hello!");
        let area = Rect::new(0, 0, 30, 5);
        let mut buf = Buffer::empty(area);
        Widget::render(&popup, area, &mut buf);

        // Row 1 (y=1) is the first content row inside the border.
        // "Hello!" should be visible somewhere in the buffer.
        // The popup auto-centers within the 30x5 area. With body "Hello!"
        // (width=6, height=1) + borders (2w, 2h), the popup occupies an
        // 8x3 rect centered at y=1. The body text renders at y=2.
        let row2 = collect_row(&buf, 2);
        assert!(row2.contains("Hello!"), "body text should render: {row2:?}");
    }

    /// 3. PopupState default state.
    #[test]
    fn popup_state_default() {
        let state = PopupState::default();
        assert!(state.area.is_none());
        assert_eq!(state.drag_state, DragState::NotDragging);
    }

    /// 4. Title renders in the top border.
    #[test]
    fn popup_title_renders() {
        // Use a wide body so the popup has room for the title.
        let body = "body text that is long enough for the title";
        let popup = Popup::new(body).title("My Title");
        let area = Rect::new(0, 0, 60, 5);
        let mut buf = Buffer::empty(area);
        Widget::render(&popup, area, &mut buf);

        // The popup auto-centers. The top border (with title) renders at
        // the popup's top row in the buffer.
        let row1 = collect_row(&buf, 1);
        assert!(
            row1.contains("My Title"),
            "title should appear in top border: {row1:?}"
        );
    }

    /// 5. PopupState move_by and move_to work.
    #[test]
    fn popup_state_move() {
        let mut state = PopupState::default();

        // Move before any render: no-op (area is None)
        state.move_by(5, 10);
        assert!(state.area.is_none());

        // Set an area, then move by offset
        state.area = Some(Rect::new(10, 10, 20, 5));
        state.move_by(5, 10);
        assert_eq!(state.area.unwrap().x, 15);
        assert_eq!(state.area.unwrap().y, 20);

        // Move to absolute position
        state.area = Some(Rect::new(10, 10, 20, 5));
        state.move_to(30, 40);
        assert_eq!(state.area.unwrap().x, 30);
        assert_eq!(state.area.unwrap().y, 40);
    }
}
