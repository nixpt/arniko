//! Integration tests for `tornado-list`.
//!
//! Uses the public surface through the `tornado_list` namespace.
//! The first test, `e0034_fully_qualified_resolve`, IS the round-11
//! differentiator — it proves the E0034 carve-out pattern documented
//! at the top of `crates/tornado-list/src/lib.rs` is not just a
//! doc-comment claim: Pattern A (fully-qualified trait-method call)
//! compiles + paints at the call site when both `Widget` and
//! `StatefulWidget` are in scope.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
// BOTH trait impls MUST be in scope at the test-module level for
// `e0034_fully_qualified_resolve` to actually exercise the E0034
// ambiguity and to prove the fully-qualified form works.
use ratatui::widgets::{Block, StatefulWidget, Widget};
use ratatui::text::Text;
use tornado_list::{List, ListState};

fn ti<S: Into<String>>(s: S) -> Text<'static> {
    Text::from(s.into())
}

// ─── 1. THE ROUND-11 DIFFERENTIATOR ────────────────────────────────────────

/// Proves Pattern A compiles + paints at the fully-qualified call
/// site when BOTH `Widget` and `StatefulWidget` are in scope at the
/// test module level. This is the round-11 crown-jewel proof that
/// the E0034 carve-out pattern is not just a doc-comment claim.
#[test]
fn e0034_fully_qualified_resolve() {
    let list = List::new(vec![ti("alpha"), ti("beta"), ti("gamma")]);
    let mut state = ListState::new();
    state.select(Some(1));

    let area = Rect::new(0, 0, 30, 4);
    let mut buf = Buffer::empty(area);

    // Pattern A — fully-qualified. THIS compiles. A bare-method-call
    // form (`list.render(area, buf, &mut state)`) at this site would
    // fail with E0034 "multiple applicable items in scope" because
    // BOTH `Widget` and `StatefulWidget` are in scope at the top of
    // this test module — see the `use ratatui::widgets::{StatefulWidget,
    // Widget};` import. The test passing proves the canonical
    // round-11 disambiguation works at the consumer's call site.
    <List as StatefulWidget>::render(list, area, &mut buf, &mut state);

    // The StatefulWidget render must NOT have panicked. Secondary
    // contract: state should reflect the item select above.
    assert_eq!(state.selected, Some(1));

    // At least one cell in the rendered buffer must carry content
    // (the highlight symbol on the selected row OR the item text).
    let mut found_content = false;
    for y in 0..area.height {
        for x in 0..area.width {
            let cell = buf.cell((x, y)).unwrap();
            if !cell.symbol().is_empty() && cell.symbol() != " " {
                found_content = true;
                break;
            }
        }
        if found_content {
            break;
        }
    }
    assert!(found_content, "render should paint at least one symbol");
}

// ─── 2. Widget render (3-arg stateless path) ────────────────────────────────

#[test]
fn integration_widget_render_does_not_panic() {
    let list = List::new(vec![ti("a"), ti("b"), ti("c"), ti("d"), ti("e")]);
    let area = Rect::new(0, 0, 30, 6);
    let mut buf = Buffer::empty(area);
    <List as Widget>::render(list, area, &mut buf); // 3-arg Widget::render — the default path
}

// ─── 3. StatefulWidget render (4-arg) advances state.offset ────────────────

#[test]
fn integration_stateful_render_advances_state_offset() {
    let items: Vec<Text> = (0..20)
        .map(|i| ti(format!("item-{i:02}")))
        .collect();
    let list = List::new(items);
    let area = Rect::new(0, 0, 20, 5); // viewport height = 5

    let mut buf = Buffer::empty(area);
    let mut state = ListState::new();

    // Initially at top.
    state.select(Some(0));
    StatefulWidget::render(list.clone(), area, &mut buf, &mut state);
    assert_eq!(state.offset, 0);

    // Advance selection past the visible window - offset must advance
    // so the selection remains visible (D9 scroll composition contract).
    state.select(Some(15));
    StatefulWidget::render(list, area, &mut buf, &mut state);
    assert!(
        state.offset > 0,
        "selected=15 in 20-item list, viewport h=5; offset should advance; got {}",
        state.offset
    );
}

// ─── 4. Highlight symbol applies ──────────────────────────────────────────

#[test]
fn integration_highlight_symbol_applies() {
    let list = List::new(vec![ti("alpha"), ti("beta"), ti("gamma")])
        .highlight_symbol(">>> ")
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    let mut state = ListState::new();
    state.select(Some(1));
    let area = Rect::new(0, 0, 20, 4);
    let mut buf = Buffer::empty(area);
    StatefulWidget::render(list, area, &mut buf, &mut state);

    // The ">>>" prefix symbol should appear at x=0 of the row where
    // selected=1 (visible row position 1 since offset starts at 0).
    let row_y: u16 = 1;
    let mut row = String::new();
    for x in 0..area.width {
        if let Some(cell) = buf.cell((x, row_y)) {
            row.push_str(cell.symbol());
        }
    }
    assert!(
        row.starts_with('>'),
        "highlight symbol prefix should be at row {row_y} x=0; got {row:?}"
    );
}

// ─── 5. Multi-byte items render ───────────────────────────────────────────

#[test]
fn integration_items_with_multibyte_render() {
    let items = vec![ti("▁▂▃▄ small"), ti("▅▆▇█ large")];
    let list = List::new(items);
    let area = Rect::new(0, 0, 30, 4);
    let mut buf = Buffer::empty(area);
    let mut state = ListState::new();
    state.select(Some(0));
    StatefulWidget::render(list, area, &mut buf, &mut state);
    // No panic — primary contract.
    let row0 = {
        let mut s = String::new();
        for x in 0..area.width {
            s.push_str(buf[(x, 0)].symbol());
        }
        s
    };
    assert!(
        row0.contains('▁') || !row0.trim().is_empty(),
        "should render content: {row0:?}"
    );
}

// ─── 6. Block top border visible ──────────────────────────────────────────

#[test]
fn integration_block_top_border_visible() {
    let list = List::new(vec![ti("a"), ti("b")]).block(Block::bordered());
    let area = Rect::new(0, 0, 10, 6);
    let mut buf = Buffer::empty(area);
    <List as Widget>::render(list, area, &mut buf);
    let mut row0 = String::new();
    for x in 0..area.width {
        row0.push_str(buf[(x, 0)].symbol());
    }
    assert!(
        row0.chars().any(|c| c == '─' || c == '╭' || c == '╮'),
        "top border should be visible: {row0:?}"
    );
}
