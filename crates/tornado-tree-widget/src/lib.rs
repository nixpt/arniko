//! # Vendored `Tree` + `TreeItem` + `TreeState` — from `EdJoPaTo/tui-rs-tree-widget` (MIT)
//!
//! ## (1) Provenance + license attribution
//!
//! This crate carries a faithful mirror of the public `Tree` widget,
//! `TreeItem`, and `TreeState` from upstream
//! `EdJoPaTo/tui-rs-tree-widget` v0.24.0. Upstream:
//! <https://github.com/EdJoPaTo/tui-rs-tree-widget>.
//!
//! Original upstream `tui-tree-widget` is MIT-licensed — see `LICENSE-MIT`
//! and `NOTICE` at the root of this crate for the attribution.
//!
//! ## (2) Adaptations from upstream
//!
//! Compared to upstream `tui-tree-widget` v0.24.0:
//!
//! - **Import paths**: Upstream uses `ratatui_core` and `ratatui_widgets`
//!   directly. This mirror uses the umbrella `ratatui` crate (workspace dep).
//! - **No re-exports**: Upstream re-exports `Block`, `Scrollbar`, and
//!   `ScrollbarState` for convenience. This mirror does not re-export them —
//!   consumers access them through `ratatui::widgets` directly.
//!
//! ## (3) Public surface
//!
//! * [`Tree`] — A stateful widget for rendering tree data structures.
//! * [`TreeItem`] — A node in the tree, with identifier, text content, and
//!   optional children.
//! * [`TreeState`] — State carrier for selection, open/close tracking, and
//!   scroll offset.
//! * [`Flattened`] — A flat view of all currently-visible tree items.

use std::collections::HashSet;

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, Scrollbar, ScrollbarState, StatefulWidget, Widget};
use unicode_width::UnicodeWidthStr;

// ─── `TreeItem` — tree node ───────────────────────────────────────────────

/// One item inside a [`Tree`].
///
/// Can have zero or more `children`.
///
/// The generic argument `Identifier` is used to keep the state like the
/// currently selected or opened [`TreeItem`]s in the [`TreeState`].
///
/// It needs to be unique among its siblings but can be used again on parent
/// or child [`TreeItem`]s.
#[derive(Debug, Clone)]
pub struct TreeItem<'text, Identifier> {
    pub(crate) identifier: Identifier,
    pub(crate) text: Text<'text>,
    pub(crate) children: Vec<Self>,
}

impl<'text, Identifier> TreeItem<'text, Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    /// Create a new `TreeItem` without children.
    #[must_use]
    pub fn new_leaf<T>(identifier: Identifier, text: T) -> Self
    where
        T: Into<Text<'text>>,
    {
        Self {
            identifier,
            text: text.into(),
            children: Vec::new(),
        }
    }

    /// Create a new `TreeItem` with children.
    ///
    /// # Errors
    ///
    /// Errors when there are duplicate identifiers in the children.
    pub fn new<T>(
        identifier: Identifier,
        text: T,
        children: Vec<Self>,
    ) -> std::io::Result<Self>
    where
        T: Into<Text<'text>>,
    {
        let identifiers = children
            .iter()
            .map(|item| &item.identifier)
            .collect::<HashSet<_>>();
        if identifiers.len() != children.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "The children contain duplicate identifiers",
            ));
        }

        Ok(Self {
            identifier,
            text: text.into(),
            children,
        })
    }

    /// Get a reference to the identifier.
    #[must_use]
    pub const fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    /// Get the children.
    #[must_use]
    pub fn children(&self) -> &[Self] {
        &self.children
    }

    /// Get a reference to a child by index.
    #[must_use]
    pub fn child(&self, index: usize) -> Option<&Self> {
        self.children.get(index)
    }

    /// Get a mutable reference to a child by index.
    #[must_use]
    pub fn child_mut(&mut self, index: usize) -> Option<&mut Self> {
        self.children.get_mut(index)
    }

    /// The height of this item in terminal rows.
    #[must_use]
    pub fn height(&self) -> usize {
        self.text.height()
    }

    /// Add a child to this item.
    ///
    /// # Errors
    ///
    /// Errors when the `identifier` of the `child` already exists in children.
    pub fn add_child(&mut self, child: Self) -> std::io::Result<()> {
        let existing = self
            .children
            .iter()
            .map(|item| &item.identifier)
            .collect::<HashSet<_>>();
        if existing.contains(&child.identifier) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "identifier already exists in the children",
            ));
        }
        self.children.push(child);
        Ok(())
    }
}

// ─── `Flattened` — flat tree view ─────────────────────────────────────────

/// A flattened item of all currently-visible [`TreeItem`]s.
///
/// Generated via [`TreeState::flatten`].
#[must_use]
pub struct Flattened<'text, Identifier> {
    /// Full path of identifiers from root to this item.
    pub identifier: Vec<Identifier>,
    /// The underlying tree item.
    pub item: &'text TreeItem<'text, Identifier>,
}

impl<Identifier> Flattened<'_, Identifier> {
    /// Zero-based depth. Depth 0 means top-level with no indentation.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.identifier.len().saturating_sub(1)
    }
}

/// Recursively flatten visible items, collecting open children.
fn flatten<'text, Identifier>(
    open_identifiers: &HashSet<Vec<Identifier>>,
    items: &'text [TreeItem<'text, Identifier>],
    current: &[Identifier],
) -> Vec<Flattened<'text, Identifier>>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    let mut result = Vec::new();
    for item in items {
        let mut child_identifier = current.to_vec();
        child_identifier.push(item.identifier.clone());

        let child_result = open_identifiers
            .contains(&child_identifier)
            .then(|| flatten(open_identifiers, &item.children, &child_identifier));

        result.push(Flattened {
            identifier: child_identifier,
            item,
        });

        if let Some(mut child_result) = child_result {
            result.append(&mut child_result);
        }
    }
    result
}

// ─── `TreeState` — selection + open/close state ───────────────────────────

/// Keeps the selection, open/close state, and scroll offset for a [`Tree`].
#[must_use]
#[derive(Debug)]
pub struct TreeState<Identifier> {
    pub(crate) offset: usize,
    pub(crate) opened: HashSet<Vec<Identifier>>,
    pub(crate) selected: Vec<Identifier>,
    pub(crate) ensure_selected_in_view_on_next_render: bool,
    pub(crate) last_area: Rect,
    pub(crate) last_biggest_index: usize,
    pub(crate) last_identifiers: Vec<Vec<Identifier>>,
    pub(crate) last_rendered_identifiers: Vec<(u16, Vec<Identifier>)>,
}

impl<Identifier> Default for TreeState<Identifier> {
    fn default() -> Self {
        Self {
            offset: 0,
            opened: HashSet::new(),
            selected: Vec::new(),
            ensure_selected_in_view_on_next_render: false,
            last_area: Rect::ZERO,
            last_biggest_index: 0,
            last_identifiers: Vec::new(),
            last_rendered_identifiers: Vec::new(),
        }
    }
}

impl<Identifier> TreeState<Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    /// Get the current scroll offset.
    #[must_use]
    pub const fn get_offset(&self) -> usize {
        self.offset
    }

    /// Get all opened node identifier paths.
    #[must_use]
    pub const fn opened(&self) -> &HashSet<Vec<Identifier>> {
        &self.opened
    }

    /// Get the currently selected identifier path.
    #[must_use]
    pub fn selected(&self) -> &[Identifier] {
        &self.selected
    }

    /// Get a flat list of all currently viewable [`TreeItem`]s.
    #[must_use]
    pub fn flatten<'text>(
        &self,
        items: &'text [TreeItem<'text, Identifier>],
    ) -> Vec<Flattened<'text, Identifier>> {
        flatten(&self.opened, items, &[])
    }

    /// Select the given identifier path. Returns `true` when the selection changed.
    pub fn select(&mut self, identifier: Vec<Identifier>) -> bool {
        self.ensure_selected_in_view_on_next_render = true;
        let changed = self.selected != identifier;
        self.selected = identifier;
        changed
    }

    /// Open a tree node. Returns `true` when it was closed and has been opened.
    pub fn open(&mut self, identifier: Vec<Identifier>) -> bool {
        if identifier.is_empty() {
            false
        } else {
            self.opened.insert(identifier)
        }
    }

    /// Close a tree node. Returns `true` when it was open and has been closed.
    pub fn close(&mut self, identifier: &[Identifier]) -> bool {
        self.opened.remove(identifier)
    }

    /// Toggle a tree node open/close state.
    /// Returns `true` when a node is opened or closed.
    pub fn toggle(&mut self, identifier: Vec<Identifier>) -> bool {
        if identifier.is_empty() {
            false
        } else if self.opened.contains(&identifier) {
            self.close(&identifier)
        } else {
            self.open(identifier)
        }
    }

    /// Toggle the currently selected tree node open/close state.
    /// Returns `true` when a node is opened or closed.
    pub fn toggle_selected(&mut self) -> bool {
        if self.selected.is_empty() {
            return false;
        }
        self.ensure_selected_in_view_on_next_render = true;
        if self.opened.remove(&self.selected) {
            return true;
        }
        self.open(self.selected.clone())
    }

    /// Close all open nodes. Returns `true` when any node was closed.
    pub fn close_all(&mut self) -> bool {
        if self.opened.is_empty() {
            false
        } else {
            self.opened.clear();
            true
        }
    }

    /// Select the first visible node. Returns `true` when selection changed.
    pub fn select_first(&mut self) -> bool {
        let identifier = self.last_identifiers.first().cloned().unwrap_or_default();
        self.select(identifier)
    }

    /// Select the last visible node. Returns `true` when selection changed.
    pub fn select_last(&mut self) -> bool {
        let new_identifier = self.last_identifiers.last().cloned().unwrap_or_default();
        self.select(new_identifier)
    }

    /// Move the selection relative to the current position.
    ///
    /// `change_function` receives `Option<usize>` (the current index, or
    /// `None` if nothing is selected) and returns the new index.
    pub fn select_relative<F>(&mut self, change_function: F) -> bool
    where
        F: FnOnce(Option<usize>) -> usize,
    {
        let identifiers = &self.last_identifiers;
        let current_index = identifiers
            .iter()
            .position(|id| id == &self.selected);
        let new_index = change_function(current_index).min(self.last_biggest_index);
        let new_identifier = identifiers.get(new_index).cloned().unwrap_or_default();
        self.select(new_identifier)
    }

    /// Get the identifier rendered at the given position on the last render.
    #[must_use]
    pub fn rendered_at(&self, position: Position) -> Option<&[Identifier]> {
        if !self.last_area.contains(position) {
            return None;
        }
        self.last_rendered_identifiers
            .iter()
            .rev()
            .find(|(y, _)| position.y >= *y)
            .map(|(_, identifier)| identifier.as_ref())
    }

    /// Click at a position: select what's there, or toggle if already selected.
    /// Returns `true` when the state changed.
    pub fn click_at(&mut self, position: Position) -> bool {
        if let Some(identifier) = self.rendered_at(position) {
            if identifier == self.selected {
                self.toggle_selected()
            } else {
                self.select(identifier.to_vec())
            }
        } else {
            false
        }
    }

    /// Ensure the selected item is visible on the next render.
    pub const fn scroll_selected_into_view(&mut self) {
        self.ensure_selected_in_view_on_next_render = true;
    }

    /// Scroll up by `lines`. Returns `true` when the offset changed.
    pub const fn scroll_up(&mut self, lines: usize) -> bool {
        let before = self.offset;
        self.offset = self.offset.saturating_sub(lines);
        before != self.offset
    }

    /// Scroll down by `lines`. Returns `true` when the offset changed.
    pub fn scroll_down(&mut self, lines: usize) -> bool {
        let before = self.offset;
        self.offset = self
            .offset
            .saturating_add(lines)
            .min(self.last_biggest_index);
        before != self.offset
    }

    /// Handle the up arrow key. Returns `true` when selection changed.
    pub fn key_up(&mut self) -> bool {
        self.select_relative(|current| {
            current.map_or(usize::MAX, |c| c.saturating_sub(1))
        })
    }

    /// Handle the down arrow key. Returns `true` when selection changed.
    pub fn key_down(&mut self) -> bool {
        self.select_relative(|current| {
            current.map_or(0, |c| c.saturating_add(1))
        })
    }

    /// Handle the left arrow key: close node or move to parent.
    /// Returns `true` when selection or open state changed.
    pub fn key_left(&mut self) -> bool {
        self.ensure_selected_in_view_on_next_render = true;
        let mut changed = self.opened.remove(&self.selected);
        if !changed {
            changed = self.selected.pop().is_some();
        }
        changed
    }

    /// Handle the right arrow key: open the selected node.
    /// Returns `true` when the node was opened.
    pub fn key_right(&mut self) -> bool {
        if self.selected.is_empty() {
            return false;
        }
        self.ensure_selected_in_view_on_next_render = true;
        self.open(self.selected.clone())
    }
}

// ─── `Tree` — the main widget ─────────────────────────────────────────────

/// A stateful tree widget built from [`TreeItem`]s.
///
/// The generic `Identifier` is used to track selection and open/close state
/// in [`TreeState`]. See [`TreeItem`] for more on identifiers.
///
/// # Example
///
/// ```rust
/// use tornado_tree_widget::{Tree, TreeItem, TreeState};
/// use ratatui::widgets::Block;
///
/// let mut state = TreeState::<usize>::default();
/// let leaf = TreeItem::new_leaf(1usize, "leaf");
/// let root = TreeItem::new(0usize, "root", vec![leaf]).unwrap();
/// let tree = Tree::new(&[root]).unwrap().block(Block::bordered().title("Tree"));
/// ```
#[must_use]
#[derive(Debug, Clone)]
pub struct Tree<'a, Identifier> {
    items: &'a [TreeItem<'a, Identifier>],
    block: Option<Block<'a>>,
    scrollbar: Option<Scrollbar<'a>>,
    style: Style,
    highlight_style: Style,
    highlight_symbol: &'a str,
    node_closed_symbol: &'a str,
    node_open_symbol: &'a str,
    node_no_children_symbol: &'a str,
}

impl<'a, Identifier> Tree<'a, Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    /// Create a new `Tree` from the given items.
    ///
    /// # Errors
    ///
    /// Errors when there are duplicate identifiers at the top level.
    pub fn new(items: &'a [TreeItem<'a, Identifier>]) -> std::io::Result<Self> {
        let identifiers: HashSet<_> = items.iter().map(|item| &item.identifier).collect();
        if identifiers.len() != items.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "The items contain duplicate identifiers",
            ));
        }
        Ok(Self {
            items,
            block: None,
            scrollbar: None,
            style: Style::new(),
            highlight_style: Style::new(),
            highlight_symbol: "",
            node_closed_symbol: "\u{25b6} ", // ▶
            node_open_symbol: "\u{25bc} ",   // ▼
            node_no_children_symbol: "  ",
        })
    }

    /// Set the block surrounding the tree.
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set the optional scrollbar.
    #[must_use]
    pub const fn experimental_scrollbar(mut self, scrollbar: Option<Scrollbar<'a>>) -> Self {
        self.scrollbar = scrollbar;
        self
    }

    /// Set the base style.
    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the highlight style for the selected item.
    #[must_use]
    pub const fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    /// Set the symbol drawn in front of the selected item.
    #[must_use]
    pub const fn highlight_symbol(mut self, symbol: &'a str) -> Self {
        self.highlight_symbol = symbol;
        self
    }

    /// Set the symbol drawn in front of a closed node.
    #[must_use]
    pub const fn node_closed_symbol(mut self, symbol: &'a str) -> Self {
        self.node_closed_symbol = symbol;
        self
    }

    /// Set the symbol drawn in front of an open node.
    #[must_use]
    pub const fn node_open_symbol(mut self, symbol: &'a str) -> Self {
        self.node_open_symbol = symbol;
        self
    }

    /// Set the symbol drawn in front of a node without children.
    #[must_use]
    pub const fn node_no_children_symbol(mut self, symbol: &'a str) -> Self {
        self.node_no_children_symbol = symbol;
        self
    }

}

impl<Identifier> StatefulWidget for Tree<'_, Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    type State = TreeState<Identifier>;

    #[expect(clippy::too_many_lines)]
    fn render(self, full_area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_style(full_area, self.style);

        // Apply block boundaries
        let area = self.block.as_ref().map_or(full_area, |block| {
            let inner = block.inner(full_area);
            block.render(full_area, buf);
            inner
        });

        state.last_area = area;
        state.last_rendered_identifiers.clear();
        if area.width < 1 || area.height < 1 {
            return;
        }

        let visible = state.flatten(self.items);
        state.last_biggest_index = visible.len().saturating_sub(1);
        if visible.is_empty() {
            return;
        }
        let available_height = area.height as usize;

        // Determine if we need to scroll to show the selected item
        let ensure_index = if state.ensure_selected_in_view_on_next_render
            && !state.selected.is_empty()
        {
            visible
                .iter()
                .position(|f| f.identifier == state.selected)
        } else {
            None
        };

        let mut start = state.offset.min(state.last_biggest_index);
        if let Some(idx) = ensure_index {
            start = start.min(idx);
        }

        // Compute the visible range
        let mut end = start;
        let mut height = 0;
        for item_height in visible.iter().skip(start).map(|f| f.item.height()) {
            if height + item_height > available_height {
                break;
            }
            height += item_height;
            end += 1;
        }

        // Ensure selected item is visible
        if let Some(ensure_idx) = ensure_index {
            while ensure_idx >= end {
                height = height.saturating_add(visible[end].item.height());
                end += 1;
                while height > available_height {
                    height = height.saturating_sub(visible[start].item.height());
                    start += 1;
                }
            }
        }

        state.offset = start;
        state.ensure_selected_in_view_on_next_render = false;

        // Render scrollbar (inline: we own self, so we can move self.scrollbar)
        if let Some(scrollbar) = self.scrollbar {
            let max_visible = visible.len().saturating_sub(height);
            let mut scrollbar_state =
                ScrollbarState::new(max_visible).position(start).viewport_content_length(height);
            let scrollbar_area = Rect {
                y: area.y,
                height: area.height,
                x: full_area.x,
                width: full_area.width,
            };
            scrollbar.render(scrollbar_area, buf, &mut scrollbar_state);
        }

        let blank_symbol = " ".repeat(self.highlight_symbol.width());

        let mut current_height = 0u16;
        let has_selection = !state.selected.is_empty();
        for flattened in visible.iter().skip(state.offset).take(end - start) {
            let Flattened { identifier, item } = flattened;

            let x = area.x;
            let y = area.y + current_height;
            let item_h = item.height() as u16;
            current_height += item_h;

            let row_area = Rect {
                x,
                y,
                width: area.width,
                height: item_h,
            };

            let text = &item.text;
            let item_style = text.style;

            let is_selected = state.selected == *identifier;

            // Render highlight symbol or blank space
            let after_highlight = if has_selection {
                let symbol = if is_selected {
                    self.highlight_symbol
                } else {
                    &blank_symbol
                };
                let (x, _) =
                    buf.set_stringn(x, y, symbol, area.width as usize, item_style);
                x
            } else {
                x
            };

            // Render depth indent + node symbol
            let indent_width = flattened.depth() * 2;
            let after_indent = {
                let (after_indent_x, _) = buf.set_stringn(
                    after_highlight,
                    y,
                    &" ".repeat(indent_width),
                    indent_width,
                    item_style,
                );
                let node_symbol = if item.children.is_empty() {
                    self.node_no_children_symbol
                } else if state.opened.contains(identifier) {
                    self.node_open_symbol
                } else {
                    self.node_closed_symbol
                };
                let max_w = area.width.saturating_sub(after_indent_x - x) as usize;
                let (x, _) =
                    buf.set_stringn(after_indent_x, y, node_symbol, max_w, item_style);
                x
            };

            // Render the item text
            let text_area = Rect {
                x: after_indent,
                width: area.width.saturating_sub(after_indent - x),
                ..row_area
            };
            text.render(text_area, buf);

            if is_selected {
                buf.set_style(row_area, self.highlight_style);
            }

            state
                .last_rendered_identifiers
                .push((row_area.y, identifier.clone()));
        }

        state.last_identifiers = visible
            .into_iter()
            .map(|f| f.identifier)
            .collect();
    }
}

impl<Identifier> Widget for Tree<'_, Identifier>
where
    Identifier: Clone + Eq + core::hash::Hash,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = TreeState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod flatten_tests {
    use super::*;

    #[test]
    fn depth_works() {
        let mut open = HashSet::new();
        open.insert(vec!["b"]);
        open.insert(vec!["b", "d"]);
        let depths: Vec<_> = flatten(&open, &example_items(), &[])
            .into_iter()
            .map(|f| f.depth())
            .collect();
        assert_eq!(depths, [0, 0, 1, 1, 2, 2, 1, 0]);
    }

    #[test]
    fn flatten_nothing_open_is_top_level() {
        check_flatten(&HashSet::new(), &["a", "b", "h"]);
    }

    #[test]
    fn flatten_wrong_open_is_only_top_level() {
        let mut open = HashSet::new();
        open.insert(vec!["a"]);
        open.insert(vec!["b", "d"]);
        check_flatten(&open, &["a", "b", "h"]);
    }

    #[test]
    fn flatten_one_is_open() {
        let mut open = HashSet::new();
        open.insert(vec!["b"]);
        check_flatten(&open, &["a", "b", "c", "d", "g", "h"]);
    }

    #[test]
    fn flatten_all_open() {
        let mut open = HashSet::new();
        open.insert(vec!["b"]);
        open.insert(vec!["b", "d"]);
        check_flatten(&open, &["a", "b", "c", "d", "e", "f", "g", "h"]);
    }

    fn check_flatten(open: &HashSet<Vec<&'static str>>, expected: &[&'static str]) {
        let items = example_items();
        let result = flatten(open, &items, &[]);
        let actual: Vec<_> = result
            .into_iter()
            .map(|f| f.identifier.into_iter().next_back().unwrap())
            .collect();
        assert_eq!(actual, expected);
    }

    fn example_items() -> Vec<TreeItem<'static, &'static str>> {
        vec![
            TreeItem::new_leaf("a", "Alfa"),
            TreeItem::new(
                "b",
                "Bravo",
                vec![
                    TreeItem::new_leaf("c", "Charlie"),
                    TreeItem::new(
                        "d",
                        "Delta",
                        vec![
                            TreeItem::new_leaf("e", "Echo"),
                            TreeItem::new_leaf("f", "Foxtrot"),
                        ],
                    )
                    .expect("unique"),
                    TreeItem::new_leaf("g", "Golf"),
                ],
            )
            .expect("unique"),
            TreeItem::new_leaf("h", "Hotel"),
        ]
    }
}

#[cfg(test)]
mod render_tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    #[must_use]
    #[track_caller]
    fn render_tree(
        width: u16,
        height: u16,
        state: &mut TreeState<&'static str>,
    ) -> Buffer {
        let items = example_items();
        let tree = Tree::new(&items).unwrap();
        let area = Rect::new(0, 0, width, height);
        let mut buffer = Buffer::empty(area);
        StatefulWidget::render(tree, area, &mut buffer, state);
        buffer
    }

    fn example_items() -> Vec<TreeItem<'static, &'static str>> {
        vec![
            TreeItem::new_leaf("a", "Alfa"),
            TreeItem::new(
                "b",
                "Bravo",
                vec![
                    TreeItem::new_leaf("c", "Charlie"),
                    TreeItem::new(
                        "d",
                        "Delta",
                        vec![
                            TreeItem::new_leaf("e", "Echo"),
                            TreeItem::new_leaf("f", "Foxtrot"),
                        ],
                    )
                    .expect("unique"),
                    TreeItem::new_leaf("g", "Golf"),
                ],
            )
            .expect("unique"),
            TreeItem::new_leaf("h", "Hotel"),
        ]
    }

    #[test]
    fn does_not_panic() {
        let _ = render_tree(0, 0, &mut TreeState::default());
        let _ = render_tree(10, 0, &mut TreeState::default());
        let _ = render_tree(0, 10, &mut TreeState::default());
        let _ = render_tree(10, 10, &mut TreeState::default());
    }

    #[test]
    fn nothing_open() {
        let buffer = render_tree(10, 4, &mut TreeState::default());
        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "  Alfa    ",
            "▶ Bravo   ",
            "  Hotel   ",
            "          ",
        ]);
        assert_eq!(buffer, expected);
    }

    #[test]
    fn depth_one() {
        let mut state = TreeState::default();
        state.open(vec!["b"]);
        let buffer = render_tree(13, 7, &mut state);
        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "  Alfa       ",
            "▼ Bravo      ",
            "    Charlie  ",
            "  ▶ Delta    ",
            "    Golf     ",
            "  Hotel      ",
            "             ",
        ]);
        assert_eq!(buffer, expected);
    }

    #[test]
    fn depth_two() {
        let mut state = TreeState::default();
        state.open(vec!["b"]);
        state.open(vec!["b", "d"]);
        let buffer = render_tree(15, 9, &mut state);
        #[rustfmt::skip]
        let expected = Buffer::with_lines([
            "  Alfa         ",
            "▼ Bravo        ",
            "    Charlie    ",
            "  ▼ Delta      ",
            "      Echo     ",
            "      Foxtrot  ",
            "    Golf       ",
            "  Hotel        ",
            "               ",
        ]);
        assert_eq!(buffer, expected);
    }
}

#[cfg(test)]
mod item_tests {
    use super::*;

    #[test]
    #[should_panic = "duplicate identifiers"]
    fn tree_item_new_errors_with_duplicate_identifiers() {
        let item = TreeItem::new_leaf("same", "text");
        let another = item.clone();
        let _ = TreeItem::new("root", "Root", vec![item, another]).unwrap();
    }

    #[test]
    #[should_panic = "identifier already exists"]
    fn tree_item_add_child_errors_with_duplicate_identifiers() {
        let item = TreeItem::new_leaf("same", "text");
        let another = item.clone();
        let mut root = TreeItem::new("root", "Root", vec![item]).unwrap();
        root.add_child(another).unwrap();
    }

    #[test]
    #[should_panic = "duplicate identifiers"]
    fn tree_new_errors_with_duplicate_identifiers() {
        let item = TreeItem::new_leaf("same", "text");
        let another = item.clone();
        let items = [item, another];
        let _: Tree<&str> = Tree::new(&items).unwrap();
    }
}
