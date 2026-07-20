//! Integration test for `tornado` + `tornado-tabs` (`--features tabs`).
//!
//! Guards the tab widget end-to-end:
//!  * `TabNav` is reachable via the `tornado::tabs` re-export and the
//!    widget alias at `tornado::widget::TabNav`.
//!  * The fluent builder chain compiles through the re-export.
//!  * `select(idx)` actually selects the active tab; the indicator
//!    `▸` appears left of the selected label.
//!  * `indicator(None)` removes the indicator glyph without breaking
//!    the rest of the render.
//!  * The vendored upstream render-exit paths (empty tabs and
//!    insufficient height) are reachable through the warehouse-side
//!    re-export.
//!
//! Compiled only when the `tabs` feature is enabled (see
//! `required-features` in `Cargo.toml`).

#![cfg(feature = "tabs")]

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::symbols;
use ratatui::widgets::Widget;
use tornado::tabs::TabNav;

fn line_str(buf: &Buffer, y: u16) -> String {
    let area = buf.area;
    (area.x..area.right())
        .map(|x| buf[(x, y)].symbol().to_string())
        .collect()
}

#[test]
fn tab_nav_widget_is_reachable_through_tornado_tabs() {
    // Reachability + render path: build the widget through the
    // warehouse-side re-export, render it, and confirm the rendered
    // area has visible content at known coordinates.
    let area = Rect::new(0, 0, 30, 3);
    let mut buf = Buffer::empty(area);
    TabNav::new(&["Files", "Search", "Settings"], 0).render(area, &mut buf);

    // First tab is active; its top-left corner should be the rounded
    // `╭` glyph at (0, 0), and the label "Files" should appear in the
    // middle row.
    assert_eq!(buf[(0, 0)].symbol(), "╭");
    let mid_line = line_str(&buf, 1);
    assert!(mid_line.contains("Files"));
}

#[test]
fn widget_alias_tab_nav_compiles_through_re_export() {
    // Reachability compile-only check: `tornado::widget::TabNav`
    // resolves to the same type as `tornado::tabs::TabNav`. The
    // assignment below forces the type identity through
    // monomorphization. Mirrors the `widget::scroller::ScrollView`
    // convention from round 5.
    let _alias: tornado::widget::TabNav = TabNav::new(&["a"], 0);
}

#[test]
fn select_changes_active_tab_indicator() {
    // tab_width = label.len() + 8. "Files" (len=5) → 13 wide, occupies cols
    // 0..=12 (left_x=0, right_x=12); "Search" (len=6) → 14 wide, occupies
    // cols 13..=26 (left_x=13, right_x=26). The indicator glyph `▸` is drawn
    // at `left_x + 2`, so it sits at column 2 for the first tab and column
    // 15 for the second.
    let indicator_x_first: u16 = 2;
    let indicator_x_second: u16 = 15;
    let area = Rect::new(0, 0, 30, 3);

    let mut buf_first = Buffer::empty(area);
    TabNav::new(&["Files", "Search"], 0).render(area, &mut buf_first);
    assert_eq!(buf_first[(indicator_x_first, 1)].symbol(), "▸");

    let mut buf_second = Buffer::empty(area);
    TabNav::new(&["Files", "Search"], 1).render(area, &mut buf_second);
    assert_eq!(buf_second[(indicator_x_second, 1)].symbol(), "▸");
}

#[test]
fn indicator_can_be_disabled() {
    let area = Rect::new(0, 0, 20, 3);
    let mut buf = Buffer::empty(area);
    TabNav::new(&["Tab"], 0)
        .indicator(None)
        .render(area, &mut buf);
    let mid_line = line_str(&buf, 1);
    // With indicator(None), the `▸` glyph should NOT appear in the
    // middle row.
    assert!(!mid_line.contains("▸"));
}

#[test]
fn empty_tabs_render_a_blank_buffer() {
    // Upstream's early-exit on `tabs.is_empty()` is preserved after
    // the path migration. Verify the buffer at the call exit is
    // identical to a fresh empty buffer.
    let area = Rect::new(0, 0, 30, 3);
    let mut buf = Buffer::empty(area);
    let expected = buf.clone();
    TabNav::new(&[], 0).render(area, &mut buf);
    assert_eq!(buf, expected);
}

#[test]
fn insufficient_height_renders_a_blank_buffer() {
    // Same early-exit path, but for the insufficient-height guard.
    let area = Rect::new(0, 0, 30, 2);
    let mut buf = Buffer::empty(area);
    let expected = buf.clone();
    TabNav::new(&["Tab"], 0).render(area, &mut buf);
    assert_eq!(buf, expected);
}

#[test]
fn square_border_set_switches_corners() {
    let area = Rect::new(0, 0, 20, 3);
    let mut buf = Buffer::empty(area);
    TabNav::new(&["Tab"], 0)
        .border_set(symbols::border::PLAIN)
        .render(area, &mut buf);
    // PL corner on the first row left-most cell.
    assert_eq!(buf[(0, 0)].symbol(), "┌");
}

#[test]
fn builder_chain_with_all_setters_compiles() {
    // Reachability + builder API surface: style + highlight_style +
    // highlight_bold + border_style + indicator + border_set all
    // fluent. This test alone proves the entire public chain
    // compiles through the warehouse-side re-export.
    let _ = TabNav::new(&["Files", "Settings"], 1)
        .style(Style::new().fg(Color::DarkGray))
        .highlight_style(Style::new().fg(Color::Cyan))
        .highlight_bold(false)
        .border_style(Style::new().fg(Color::Gray))
        .indicator(Some(">"))
        .border_set(symbols::border::PLAIN);
}
