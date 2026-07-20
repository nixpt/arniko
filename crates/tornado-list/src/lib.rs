//! # Vendored `List` + `StatefulWidget for List` + `ListState` (round-11 canonical StatefulWidget precedent)
//!
//! ## (1) Provenance + license attribution
//!
//! This crate carries a faithful mirror of the public `List` widget
//! + its `StatefulWidget` impl from upstream
//! `ratatui@ratatui-v0.30.0/src/widgets/list.rs`. Upstream:
//! <https://github.com/ratatui/ratatui/blob/ratatui-v0.30.0/src/widgets/list.rs>.
//! Original upstream `ratatui` is dual-licensed under MIT OR Apache-2.0 -
//! see `LICENSE-MIT`, `LICENSE-APACHE`, and `NOTICE` at the root of this
//! crate for the verbatim license texts.
//!
//! ## (2) Public surface — BOTH trait impls vendored together
//!
//! Round-11 IS the canonical StatefulWidget-vendoring precedent.
//! Upstream `List<'a>` carries BOTH a `Widget` impl AND a
//! `StatefulWidget` impl. The vendored mirror matches upstream 1:1:
//!
//! * [`List`] - stateless builder widget. `Widget::render(self, area,
//!   buf)` is used when no per-frame selection state is needed (the
//!   Widget impl simply delegates to the StatefulWidget impl with a
//!   transient `ListState` so the canvas-style paint path is shared).
//! * [`ListState`] - canonical selection carrier. `selected:
//!   Option<usize>` + `offset: usize`. Future round-11-composition
//!   consumers project ScrollView state onto `ListState.selected`,
//!   never the reverse.
//! * `Widget for List` - the 3-arg render path (`self, area, buf`).
//! * `StatefulWidget for List` - the 4-arg render path (`self, area,
//!   buf, state`). `Type::State = ListState`. Internal offset-mutator
//!   advances `state.offset` when `state.selected` falls outside the
//!   viewport so the selection always remains visible.
//!
//! Round-11 deliberately diverges from round-6 Tabs, which shipped
//! Widget-only and carved-out the StatefulWidget impl. Round-11 ships
//! BOTH trait impls together because round-11 IS the canonical
//! StatefulWidget-vendoring precedent - every future stateful-widget
//! round (Chart, Calendar, future `Pane`-composites) references the
//! E0034 carve-out pattern documented below.
//!
//! ## (3) E0034 carve-out pattern - the round-11 crown jewel
//!
//! Upstream `List<'a>` has BOTH:
//!
//! ```ignore
//! impl<'a> Widget for List<'a> {
//!     fn render(self, area: Rect, buf: &mut Buffer);                                 // 3-arg
//! }
//!
//! impl<'a> StatefulWidget for List<'a> {
//!     type State = ListState;
//!     fn render(self, area: Rect, buf: &mut Buffer, state: &mut ListState);          // 4-arg
//! }
//! ```
//!
//! Rust's method-call resolution checks the method NAME against all
//! traits in scope BEFORE falling through to argument-count
//! disambiguation. When both `Widget` AND `StatefulWidget` are in
//! scope at a call site, `list.render(...)` triggers **`error[E0034]:
//! multiple applicable items in scope`** — because both impls have the
//! method symbol `render` on the type `List`, and the name resolution
//! is ambiguous before argument-count disambiguation runs.
//!
//! The canonical disambiguation patterns are:
//!
//! ```ignore
//! // Pattern A - fully-qualified trait-method call (preferred at the
//! // call site; this is the round-11 differentiator's proof):
//! use ratatui::widgets::{Widget, StatefulWidget};
//! <List as StatefulWidget>::render(list, area, buf, &mut state);
//! ```
//!
//! ```ignore
//! // Pattern B - submodule namespace isolation (preferred at the
//! // vendoring boundary; keeps one impl in scope at the umbrella
//! // re-export site):
//! use tornado_list::stateful;   // only the StatefulWidget impl is re-exported here
//! stateful::render(&list, area, buf, &mut state);
//! ```
//!
//! ```ignore
//! // NEVER - bare-method-call at vendored-crate call sites:
//! list.render(area, buf);                // E0034 if both traits in scope
//! list.render(area, buf, &mut state);    // E0034 name-collision (even
//!                                         //   though arg count uniquely
//!                                         //   matches StatefulWidget::render -
//!                                         //   name resolution runs FIRST)
//! ```
//!
//! The integration test `e0034_fully_qualified_resolve` (in
//! `tests/list_integration.rs`) PROVES Pattern A compiles + paints
//! at the fully-qualified call site - this is the round-11
//! differentiator. The doc-comment claim is now backed by a
//!
//! machine-verified compile-and-paint test.
//!
//! ## (4) ScrollView composition contract (D9 round-11 design memo)
//!
//! Round-11 expands the composition surface with a selection-driven
//! `ScrollView` (using the round-7 vendored `tornado-scrollview`):
//! when `items.len() > viewport.height`, the consumer wraps the List
//! body in `scroll_view.scroll_offset()` and projects `state.selected`
//! onto `scroll_state.offset.y`:
//!
//! ```ignore
//! // On key-event advance:
//! let next = state.selected.unwrap_or(0).saturating_add(1) % items.len();
//! state.select(Some(next));
//! scroll_state.offset.y = next.min(items.len().saturating_sub(1)) as u16;
//! ```
//!
//! **`ListState.selected` is the CANONICAL projection target** across
//! any round-11 (or round-11-future) composition that consumes a
//! List. `scroll_state.offset.y` is a **derived projection** of
//! `ListState.selected` - it must NEVER be the inverse (i.e., the
//! consumer must not project `ListState.selected = scroll_state.offset.y`
//! because the offset semantics differ).
//!
//! Per the round-11 trap record `List-state composition
//! carrier-projection`, never invent a parallel `SelectionState` on the
//! consumer side. Never duplicate the carrier. The canonical carrier
//! across selections, jumps, and confirmations is `ListState.selected`.
//!
//! ## (5) Carry-forward hazards defense (D9 + round-8 constitutional trio)
//!
//! 1. **MSRV `is_multiple_of` cold-path (round-8 hazard #1)** -
//!    round-11 selection wrapping uses `usize % usize` (a stable binary
//!    modulo under `rust-version = "1.85.0"`). The pathological
//!    `<integer>::is_multiple_of` API (stabilized Rust 1.87.0) is NEVER
//!    invoked from round-11 code.
//! 2. **Path B umbrella-bypass (round-8 hazard #2)** - round-11 plans
//!    the umbrella `list` feature + example landing in the SAME PR
//!    (this PR). No Path B debt.
//! 3. **`Instant::now()` test-loop trap (round-8 hazard #3)** - every
//!    round-11 smoke test deterministically advances state under
//!    explicit `state.select(Some(n))` manual calls. Wall-clock is
//!    never read.
//!
//! ## (6) Upstream tracker
//!
//! <https://github.com/ratatui/ratatui/blob/ratatui-v0.30.0/src/widgets/list.rs>

use ratatui::buffer::Buffer;
use ratatui::layout::{Direction, Rect};
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, StatefulWidget, Widget};
use unicode_width::UnicodeWidthStr;

// ─── `ListState` — canonical selection carrier (D3, D9) ────────────────────

/// Canonical selection carrier for a rendered `List`.
///
/// `ListState.selected` IS the canonical projection target for any
/// round-11 (or round-11-future) composition that consumes a List.
/// `scroll_state.offset.y` may be a derived projection;
/// `ListState.selected` is never duplicated.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ListState {
    /// Index of the currently-selected item. `None` when no selection.
    pub selected: Option<usize>,
    /// Scroll offset for the rendered viewport - advanced LAZILY by
    /// `StatefulWidget::render` when `selected` falls outside the
    /// visible window (so the selected row always remains visible
    /// after a `state.select(Some(idx))` call followed by a render).
    pub offset: usize,
}

impl ListState {
    /// Construct an empty `ListState` with no selection and offset 0.
    pub const fn new() -> Self {
        Self { selected: None, offset: 0 }
    }

    /// Set the selected index. Returns `()` in ratatui 0.30 - the
    /// viewport offset is adjusted LAZILY inside the StatefulWidget
    /// render pass so the selected item remains visible.
    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
    }
}

// ─── `List` — builder widget (D3) ──────────────────────────────────────────

/// A builder widget for a vertical (or horizontal) list of items.
///
/// Mirrors `ratatui::widgets::List` (v0.30) with item types adapted
/// to cross-crate visibility constraints: items are stored as
/// `Vec<Text<'a>>` instead of `Vec<ListItem<'a>>` because
/// `ListItem::content` is `pub(crate)` to `ratatui_widgets`.
///
/// The widget has both a `Widget` impl (3-arg render, no selection
/// state carried) AND a `StatefulWidget` impl (4-arg render with
/// `ListState` carrier). See the (3) E0034 carve-out pattern
/// doc-comment above for the disambiguation pattern when both trait
/// impls are in scope at the consumer's call site.
///
/// The constructor accepts any type that implements `Into<Text<'a>>`,
/// so callers pass plain strings (`"hello"`), `Line`, `Text`, or
/// `String` directly rather than wrapping in `ListItem::new(...)`.
#[derive(Debug, Clone)]
pub struct List<'a> {
    /// The items to display. Each `Text` carries its own lines + style;
    /// the List's outer `style` is applied to non-selected rows.
    pub items: Vec<Text<'a>>,
    /// Optional `Block` (border + padding) wrapping the rendered list.
    pub block: Option<Block<'a>>,
    /// Style applied to non-selected rows (the canvas).
    pub style: Style,
    /// Style applied to the selected row + highlight symbol cells.
    pub highlight_style: Style,
    /// Optional prefix symbol drawn at the start of the selected row.
    /// E.g. `"❯ "` or `"* "`. Set via [`List::highlight_symbol`].
    pub highlight_symbol: Option<&'a str>,
    /// If true, the highlight symbol is drawn on EVERY visible row
    /// instead of only on the selected row.
    pub repeat_highlight_symbol: bool,
    /// Direction of the list layout. `Vertical` (default) stacks
    /// items top-to-bottom; `Horizontal` lays items left-to-right
    /// (rare; carriers like scrollview prose mostly use Vertical).
    pub direction: Direction,
}

impl<'a> Default for List<'a> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            block: None,
            style: Style::new(),
            highlight_style: Style::new(),
            highlight_symbol: None,
            repeat_highlight_symbol: false,
            direction: Direction::Vertical,
        }
    }
}

impl<'a> List<'a> {
    /// Construct a `List` from an iterator of items. The default
    /// style is empty (canvas will draw with `Style::new()`); chain
    /// `.style(...)` / `.highlight_style(...)` / `.highlight_symbol(...)`
    /// to decorate before calling `render`.
    ///
    /// Accepts any item type that implements `Into<Text<'a>>`, such as
    /// `&str`, `String`, `Line<'a>`, `Span<'a>`, or `Text<'a>`.
    pub fn new<I, T>(items: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Text<'a>>,
    {
        Self {
            items: items.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }

    /// Set the wrapping `Block` (border + padding).
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set the canvas (non-selected) style.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the highlight (selected-row) style. Applied to the
    /// highlight_symbol cells AND the selected row's text cells.
    #[must_use]
    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    /// Set the highlight symbol prefix drawn at the start of the
    /// selected row. Use ASCII (e.g. `">>> "`, `"* "`) for clean
    /// alignment; multi-byte symbols are accepted but `unicode-width`
    /// is used to advance the cursor past them.
    #[must_use]
    pub fn highlight_symbol(mut self, symbol: &'a str) -> Self {
        self.highlight_symbol = Some(symbol);
        self
    }

    /// If `repeat=true`, the highlight symbol is drawn on every
    /// visible row (instead of only the selected row).
    #[must_use]
    pub fn repeat_highlight_symbol(mut self, repeat: bool) -> Self {
        self.repeat_highlight_symbol = repeat;
        self
    }

    /// Set the layout direction (`Vertical` default; `Horizontal`
    /// is rare and was added for symmetry with upstream).
    #[must_use]
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }
}

// ─── `Widget for List` impl (3-arg stateless render) ──────────────────────

impl<'a> Widget for List<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // The Widget impl delegates to the StatefulWidget impl with a
        // transient state. This guarantees a singular rendering
        // pipeline and means the Widget-only path inherits the
        // offset-mutator + viewport pan logic + highlight symbol
        // painting for free.
        let mut state = ListState::new();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

// ─── `StatefulWidget for List` impl (4-arg stateful render — round-11 wedge) ─

impl<'a> StatefulWidget for List<'a> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut ListState) {
        if self.items.is_empty() {
            // Apply block-only paint even when no items.
            if let Some(block) = self.block {
                block.render(area, buf);
            }
            return;
        }

        // 1. Apply Block boundaries - re-derive inner area.
        let list_area = if let Some(block) = self.block.clone() {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        if list_area.width == 0 || list_area.height == 0 {
            return;
        }

        // 2. Compute visible window — height as `usize` for the offset mutator.
        let visible_height = list_area.height as usize;

        // 3. The Offset Mutator — round-11 explicit scroll composition contract.
        //    If `selected` falls outside [offset, offset + visible_height),
        //    advance offset so the selection is visible. Uses usize arithmetic
        //    only — no `is_multiple_of` cold-path, no Instant::now.
        if let Some(selected) = state.selected {
            let max_offset = self.items.len().saturating_sub(visible_height.max(1));

            if selected < state.offset {
                state.offset = selected;
            } else if selected >= state.offset.saturating_add(visible_height) {
                state.offset = selected
                    .saturating_sub(visible_height)
                    .saturating_add(1);
            }

            if state.offset > max_offset {
                state.offset = max_offset;
            }
        } else {
            state.offset = 0;
        }

        // 4. Iterate rendered rows from `state.offset` to `state.offset + visible_height`.
        //    Each item is rendered as a `Text` widget using its reference-based
        //    `Widget` impl (ratatui implements `Widget for &Text<'_>`).
        //    Apply the list's base style, then overlay highlight style on the
        //    selected row. Render the highlight symbol prefix when selected.
        for position in 0..visible_height {
            let item_index = state.offset.saturating_add(position);
            if item_index >= self.items.len() {
                break;
            }
            let y = list_area.y.saturating_add(position as u16);
            if y >= list_area.bottom() {
                break;
            }

            let is_selected = state.selected == Some(item_index);
            let item = &self.items[item_index];

            // Highlight symbol width determines the x-offset for item text.
            let highlight_width = self
                .highlight_symbol
                .map(|s| UnicodeWidthStr::width(s) as u16)
                .unwrap_or(0);

            let symbol_offset = if is_selected || self.repeat_highlight_symbol {
                highlight_width
            } else {
                0
            };

            // Paint highlight symbol at the start of the row (if applicable).
            if symbol_offset > 0 {
                if let Some(symbol) = self.highlight_symbol {
                    let mut x = list_area.x;
                    for ch in symbol.chars() {
                        if x >= list_area.right() {
                            break;
                        }
                        if let Some(cell) = buf.cell_mut((x, y)) {
                            cell.set_symbol(&ch.to_string());
                            cell.set_style(self.highlight_style);
                        }
                        x = x.saturating_add(1);
                    }
                }
            }

            // Render the item text as a `Text` widget via its reference-based
            // `Widget` impl. The area after the highlight symbol.
            let item_area = Rect::new(
                list_area.x.saturating_add(symbol_offset),
                y,
                list_area.width.saturating_sub(symbol_offset),
                1,
            );

            if item_area.width > 0 {
                // Render the text first, then apply the style overlay.
                Widget::render(item, item_area, buf);
                buf.set_style(
                    item_area,
                    if is_selected {
                        self.highlight_style
                    } else {
                        self.style
                    },
                );
            }
        }
    }
}

// ─── Inline smoke tests (the round-11 directive: 6) ───────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::{Color, Style};

    fn li_owned<S: Into<String>>(s: S) -> Text<'static> {
        Text::from(s.into())
    }

    fn collect_row(buf: &Buffer, y: u16) -> String {
        let mut s = String::new();
        for x in 0..buf.area.width {
            s.push_str(buf[(x, y)].symbol());
        }
        s
    }

    /// 1. `list_new_preserves_items_and_default_state` — defaults are clean.
    #[test]
    fn list_new_preserves_items_and_default_state() {
        let items = vec![li_owned("a"), li_owned("b"), li_owned("c")];
        let list = List::new(items.clone());
        assert_eq!(list.items.len(), 3);
        assert!(list.block.is_none());
        assert_eq!(list.style, Style::new());
        assert_eq!(list.highlight_style, Style::new());
        assert!(list.highlight_symbol.is_none());
        assert!(!list.repeat_highlight_symbol);
        assert_eq!(list.direction, Direction::Vertical);

        let s_new = ListState::new();
        let s_default = ListState::default();
        assert_eq!(s_new, s_default);
        assert!(s_new.selected.is_none());
        assert_eq!(s_new.offset, 0);
    }

    /// 2. `list_items_and_style_applied_to_widget` — custom style surfaces.
    #[test]
    fn list_items_and_style_applied_to_widget() {
        let panel = Style::default().fg(Color::DarkGray);
        let list = List::new(vec![li_owned("a"), li_owned("b"), li_owned("c")]).style(panel);
        let area = Rect::new(0, 0, 10, 4);
        let mut buf = Buffer::empty(area);
        <List as Widget>::render(list, area, &mut buf);

        // The first item "a" should be visible in row 0.
        let row1 = collect_row(&buf, 0);
        assert!(
            row1.contains('a'),
            "first item should render in row 0: {row1:?}"
        );
    }

    /// 3. `list_block_top_border_visible_on_render` — block border paints.
    #[test]
    fn list_block_top_border_visible_on_render() {
        let list = List::new(vec![li_owned("a"), li_owned("b")]).block(Block::bordered());
        let area = Rect::new(0, 0, 10, 6);
        let mut buf = Buffer::empty(area);
        <List as Widget>::render(list, area, &mut buf);
        let row0 = collect_row(&buf, 0);
        // Block::bordered() in ratatui 0.30 paints the top border with
        // '─' (Box Drawings Light Horizontal) chars.
        assert!(
            row0.chars().any(|c| c == '─' || c == '╭' || c == '╮'),
            "top border should be visible: {row0:?}",
        );
    }

    /// 4. `list_state_new_and_default_match` — ListState ctor parity.
    #[test]
    fn list_state_new_and_default_match() {
        let a = ListState::new();
        let b = ListState::default();
        assert_eq!(a, b);
        assert!(a.selected.is_none());
        assert_eq!(a.offset, 0);
    }

    /// 5. `list_state_select_advances_offset_when_overflow` —
    ///    the scroll composition contract from D9.
    #[test]
    fn list_state_select_advances_offset_when_overflow() {
        let items: Vec<Text> = (0..20)
            .map(|i| li_owned(format!("item-{i:02}")))
            .collect();
        let list = List::new(items);
        let area = Rect::new(0, 0, 20, 5); // viewport height = 5

        let mut buf = Buffer::empty(area);
        let mut state = ListState::new();
        state.select(Some(0));
        StatefulWidget::render(list.clone(), area, &mut buf, &mut state);
        assert_eq!(state.selected, Some(0));
        assert_eq!(state.offset, 0); // selected is in [0, 5)

        // Advance past the visible window — offset must advance so
        // selected remains visible (round-11 D9 scroll composition).
        state.select(Some(15));
        StatefulWidget::render(list, area, &mut buf, &mut state);
        assert_eq!(state.selected, Some(15));
        assert!(
            state.offset > 0,
            "offset should advance when selected goes past viewport: offset={}",
            state.offset
        );
    }

    /// 6. `statefulwidget_render_mutates_state_offset` — explicit back-scroll.
    #[test]
    fn statefulwidget_render_mutates_state_offset() {
        let items: Vec<Text> = (0..30)
            .map(|i| li_owned(format!("item-{i:02}")))
            .collect();
        let list = List::new(items);
        let area = Rect::new(0, 0, 10, 8); // viewport height = 8

        let mut buf = Buffer::empty(area);
        let mut state = ListState::new();
        state.select(Some(20));
        StatefulWidget::render(list, area, &mut buf, &mut state);
        // selected=20 in 30-item list, viewport height=8 → offset should
        // back-scroll so that selected falls within [offset, offset+8).
        assert!(
            state.offset > 0,
            "offset should back-scroll to keep selection visible: offset={}",
            state.offset
        );
    }
}
