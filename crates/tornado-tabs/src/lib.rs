//! Bordered tab navigation widget for `ratatui`.
//!
//! Vendored from [`tui-tabs`](https://crates.io/crates/tui-tabs) v0.1.1 —
//! see `LICENSE-MIT`, `LICENSE-APACHE`, and `NOTICE` at the root of this
//! crate for full attribution. The source was originally written by
//! Josh Harsono and is dual-licensed under `MIT OR Apache-2.0`.
//!
//! The public surface (`TabNav`) is taken verbatim from upstream
//! `tui-tabs/src/lib.rs`. Only the import path was migrated: the
//! upstream crate declared deps on `ratatui_core = "0.1.0"` (the
//! unstable 0.29-era subcrate split), and tornado ships merged
//! `ratatui = "0.30"`. All `ratatui_core::*` imports were rewritten
//! to point at the unified `ratatui::*` crate. All behaviour, public
//! API, and tests are unchanged.
//!
//! # Co-composition with the scratch-shell siblings
//!
//! `TabNav` composes with the rest of the borrow-debris series:
//!
//! - **`ScrollView`**: nav-bar tabs above a scrolling log view;
//!   `select()` does not touch the scroll offset (independent state).
//! - **`Spinner`**: while the user is on a "loading" tab, the tab
//!   indicator `▸` cycles through spinners or coexists with a
//!   spinner on the same row.
//! - **`Hyperlink`**: when a tab label is also a link target, the
//!   `highlight_style` chain produces an OSC 8 link in the active tab
//!   label.
//!
//! Tornado wiring: `tornado::tabs::TabNav` reaches through
//! `pub use tornado_tabs as tabs;` re-exported at tornado's crate
//! root. The widget-only convenience lives at `tornado::widget::tabs::TabNav`.
//!
//! # Stateful caller pattern
//!
//! `TabNav` consumes itself by value (typical Ratatui 0.30 idiom).
//! Apps that store the widget either rebuild it each frame
//! (`TabNav::new(&titles, current_idx).highlight_style(...)`) or
//! hold it by reference and render with
//! `frame.render_widget(&tabnav, area)` using
//! `Widget for &TabNav` (added by the migration; see NOTICE).

pub mod tabs;

pub use tabs::TabNav;
