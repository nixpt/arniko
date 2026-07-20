//! Stateful scrolling view widget for `ratatui`.
//!
//! Vendored from [`tui-scrollview`](https://crates.io/crates/tui-scrollview)
//! v0.6.7 — see `LICENSE-MIT`, `LICENSE-APACHE`, and `NOTICE` at the root
//! of this crate for full attribution. The source was originally written by
//! Josh McKinney (a.k.a. Joshka) and is dual-licensed under `MIT OR
//! Apache-2.0`.
//!
//! The public surface (`ScrollView`, `ScrollViewState`, `ScrollbarVisibility`)
//! is taken verbatim from upstream `tui-scrollview/src/{lib.rs,
//! scroll_view.rs, state.rs}`. Only path imports were migrated:
//! `ratatui_core::*` → `ratatui::*` and `ratatui_widgets::*` → `ratatui::*`
//! (the unstable 0.29-era subcrate split that has been reverted in
//! `ratatui` 0.30).
//!
//! Tornado wiring: `tornado` re-exports this crate as
//! `tornado::scroll`, so downstream apps can write
//! `tornado::scroll::ScrollView::new(content_size)` directly.
//!
//! # Co-composition with the rest of the shell
//!
//! The scroll container composes naturally with the other borrow-debris
//! siblings:
//!
//! - **`status_bar`**: at the foot of a scrolling log view, the user
//!   attaches `tornado::widget::status_bar(...)` to a `Rect::new(0,
//!   height - 1, width, 1)` underneath the `ScrollView`.
//! - **`HyperlinkTarget`**: anchor offsets map onto the scroll offset.
//!   The scroll view exposes the visible `Rect` (via `state.page_size`);
//!   anchored log rows can be sanity-checked by comparing `target.line_index
//!   - state.offset.y` against the viewport bounds.
//! - **`Spinner`**: `SpinnerState::tick(Duration)` is independent of the
//!   scroll state; both can coexist on the same `Frame` as a refresh
//!   indicator while the user scrolls.
//!
//! # Stateful scroller shape
//!
//! ```ignore
//! use std::time::Duration;
//! use tornado::{scroll::ScrollView, scroll::ScrollViewState};
//! use ratatui::layout::Size;
//!
//! let mut view = ScrollView::new(Size::new(100, 30));
//! view.render_widget(
//!     Paragraph::new("Lorem ipsum dolor sit amet ..."),
//!     Rect::new(0, 0, 100, 30),
//! );
//! let mut state = ScrollViewState::default();
//!
//! fn draw(&mut self, frame: &mut Frame) {
//!     frame.render_stateful_widget(&self.view, frame.area(), &mut self.state);
//!     self.state.scroll_down();  // attach to key events / mousewheel
//! }
//! ```

mod scroll_view;
mod state;

pub use scroll_view::{ScrollView, ScrollbarVisibility};
pub use state::ScrollViewState;
