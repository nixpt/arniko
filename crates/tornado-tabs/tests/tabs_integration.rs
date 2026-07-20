//! Round-6 catch-up integration test gateway.
//!
//! Lives outside `src/lib.rs` so a `cargo test -p tornado-tabs` run
//! exercises the public Tabs surface from a sibling-crate location —
//! the same way downstream consumers (including the
//! `examples/multi-tab-log` re-routed-app) will reach it.
//!
//! Companion tests live inline in `src/lib.rs#mod tests`. The
//! gateway confirms the *public* surface remains reachable when
//! the upstream vendoring path is exercised from outside.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use tornado_tabs::Tabs;

const WIDTH: u16 = 60;
const HEIGHT: u16 = 1;

#[test]
fn tabs_public_type_is_reachable_under_tornado_tabs() {
    // Just constructing the type forces the compile + public-surface
    // check. If `Tabs<'static>` were missing or had the wrong
    // signature, this line won't compile.
    let titles = vec![Line::from("a"), Line::from("b"), Line::from("c")];
    let _ = Tabs::new(titles);
}

#[test]
fn default_construction_sets_canonical_defaults() {
    let tabs = Tabs::new(vec![Line::from("x")]);
    assert_eq!(tabs.divider, "│");
    assert_eq!(tabs.padding, " ");
    assert_eq!(tabs.selected, None);
    assert_eq!(tabs.scroll_offset, 0);
}

#[test]
fn widget_impl_render_widget_call_site_works() {
    let tabs = Tabs::new(vec![Line::from("build"), Line::from("tests")])
        .select(0)
        .divider("│")
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    let mut buf = Buffer::empty(Rect::new(0, 0, WIDTH, HEIGHT));
    tabs.render(Rect::new(0, 0, WIDTH, HEIGHT), &mut buf);

    // Verify some cell in the rendered buffer carried the highlight
    // style. (Loose assertion — depends on the highlight-style being
    // patched through `style.patch(highlight_style)` over the
    // selected index's cell.)
    let mut found_highlight = false;
    for x in 0..WIDTH {
        if let Some(cell) = buf.cell((x, 0)) {
            if cell.fg == Color::Cyan {
                found_highlight = true;
                break;
            }
        }
    }
    assert!(
        found_highlight,
        "rendered row did not surface the highlight_style fg=Cyan"
    );
}

#[test]
fn builder_methods_resolve_through_tornado_tabs() {
    let tabs = Tabs::new(vec![Line::from("alpha"), Line::from("beta")])
        .titles(vec![Line::from("replacement-a"), Line::from("replacement-b")])
        .select(1)
        .divider("/")
        .padding(" ")
        .scroll_offset(0)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    assert_eq!(tabs.titles.len(), 2);
    assert_eq!(tabs.selected, Some(1));
    assert_eq!(tabs.divider, "/");
    assert_eq!(tabs.padding, " ");
}

#[test]
fn render_at_various_width_does_not_panic() {
    // Smoke: render at extreme widths (zero, very large) to confirm
    // the vendored render math's bounds checks are correct.
    for w in [0u16, 1, 5, 30, 200] {
        let tabs = Tabs::new(vec![Line::from("alpha"), Line::from("beta")])
            .select(0)
            .divider("│")
            .padding(" ");
        let mut buf = Buffer::empty(Rect::new(0, 0, w, HEIGHT));
        tabs.render(Rect::new(0, 0, w, HEIGHT), &mut buf);
    }
}

#[test]
fn render_patches_title_style_over_span_style() {
    // The vendored render_tabs does:
    //   1. `buf.set_line(...)` to plot the title (preserves span fg
    //      momentarily in the cells).
    //   2. `cell.set_style(title_style)` to patch the title style
    //      over each rendered cell.
    // The mirror matches upstream behavior: the second step WINS, so
    // a span's `Magenta` fg is overridden by the widget's title
    // style (here `DarkGray`). This test pins that contract so a
    // future change can't accidentally regress the override.
    let styled_span = Span::styled(
        "alpha",
        Style::default().fg(Color::Magenta),
    );
    let tabs = Tabs::new(vec![Line::from(styled_span), Line::from("beta")])
        .select(0)
        .style(Style::default().fg(Color::DarkGray));
    let mut buf = Buffer::empty(Rect::new(0, 0, 30, 1));
    tabs.render(Rect::new(0, 0, 30, 1), &mut buf);

    // The Alpha cell's fg MUST be DarkGray (the patched style),
    // proving the override step ran. We deliberately do NOT assert
    // Magenta — that would couple the test to a behavior the
    // vendored mirror explicitly does not provide.
    let cell_after_padding = buf.cell((1, 0)).unwrap();
    assert_eq!(
        cell_after_padding.fg, Color::DarkGray,
        "vendored render_tabs should patch title style over span fg"
    );

    // The divider cells (drawn between titles) carry the widget's
    // base `style` (DarkGray). At column 1 (start of 'alpha'), the
    // cell fg is DarkGray — there's no Magenta present. Verify by
    // counting cells whose fg == DarkGray (>=1 confirms rendering).
    let mut darkgray_cells = 0;
    for x in 0..30 {
        if let Some(cell) = buf.cell((x, 0)) {
            if cell.fg == Color::DarkGray {
                darkgray_cells += 1;
            }
        }
    }
    assert!(
        darkgray_cells >= 1,
        "expected DarkGray title style to surface on the alpha cell"
    );
}
