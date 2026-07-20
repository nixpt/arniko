# tornado-list

Vendored `List` (builder widget) + `StatefulWidget for List` +
`ListState` (selection carrier) from `ratatui` 0.30.

This crate carries a faithful mirror of the public `List` widget
surface from `ratatui@ratatui-v0.30.0/src/widgets/list.rs` plus
the canonical StatefulWidget-vendoring precedent (round-11 design
memo D2): BOTH `Widget for List` AND `StatefulWidget for List`
vendored together, plus `ListState`.

The round-11 differentiator is the integration test
`e0034_fully_qualified_resolve` (in `tests/list_integration.rs`).
This test proves that Pattern A — the fully-qualified
trait-method call `<List as StatefulWidget>::render(list, area, buf, &mut state)`
— compiles and paints at the call site when both `Widget` and
`StatefulWidget` are in scope. Bare-method calls like
`list.render(area, buf)` would trigger `error[E0034]: multiple
applicable items in scope` — see the carve-out pattern documented
in `src/lib.rs`.

See `.dejavue/timeline.jsonl` (under `/workspace/projects/arniko/`)
for the round-11 design memo captures + the E0034 / List-composition
trap records that anchor this round as the canonical StatefulWidget-
vendoring precedent for any future round.

Upstream tracker: <https://github.com/ratatui/ratatui/blob/ratatui-v0.30.0/src/widgets/list.rs>
