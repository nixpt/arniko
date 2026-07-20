# tornado-tabs

Vendored copy of the `Tabs` widget from `ratatui` 0.30, dual-licensed
under `MIT OR Apache-2.0`. See `LICENSE-MIT`, `LICENSE-APACHE`, and
`NOTICE` for the full license text and upstream attribution.

> **⚠️ Widget-Only Mirror (intentional).**
>
> Upstream 0.30 ships `Tabs` with both a `Widget` impl and a
> `StatefulWidget` impl backed by a `TabsState` struct.
> **This vendored mirror exposes the `Widget` impl only.**
>
> When the umbrella's `tabs` feature is active, callers reach for the
> widget through
> `tornado::widget::TabNav` — this is the type that downstream
> consumers (including the round-8 `multi-tab-log` example) use,
> carrying the selected index via `.select(usize)` directly on the
> widget.
>
> `TabsState` is **not** exposed. If your consumer wants the upstream
> state-carrier semantics, manage selection in a single
> `usize` and pass it through `.select(idx)` on each frame — this is
> the round-8 pattern and is the recommended consumer contract for
> this vendoring.
>
> If a future round resumes StatefulWidget, the disambiguation pattern
> is documented in `src/lib.rs` (search for "Round 6 vendoring note").

## What's vendored

The `Tabs<'a>` builder widget — only, no `TabsState` and no
`StatefulWidget for Tabs` impl. Re-exported through the `tornado`
umbrella's `tabs` feature as:

* `tornado::tabs::Tabs` (raw ratatui path)
* `tornado::widget::TabNav` (semantic alias; round-8 and forward)

## Usage (via the umbrella)

```rust
use tornado::widget::TabNav;

let tabs = TabNav::new(vec![Line::from("build"), Line::from("tests")])
    .select(0)
    .divider("│")
    .style(Style::default().fg(Color::DarkGray))
    .highlight_style(Style::default().fg(Color::Cyan).bold());

frame.render_widget(tabs, tabnav_area);
```

## Direct usage

```rust
use tornado_tabs::Tabs;

let tabs = Tabs::new(titles)
    .select(active_idx)
    .divider("│")
    .style(inactive_style)
    .highlight_style(active_style);

frame.render_widget(tabs, area);
```

## Tests

`cargo test -p tornado-tabs` runs the 6 inline smoke tests in
`src/lib.rs` plus the 6 gateway integration tests in
`tests/tabs_integration.rs` (exercises the public surface from a
sibling-crate location, the same way downstream consumers reach it).

1. `tabs_new_preserves_titles` — `Tabs::new(titles)` round-trips the
   given titles in the rendered output.
2. `tabs_select_updates_active` — `.select(idx)` mutates the active
   index.
3. `tabs_divider_customization` — `.divider("...")` overwrites the
   default `"|"` divider.
4. `tabs_style_applied_to_inactive` — `.style(inactive)` paints
   non-selected titles.
5. `tabs_highlight_style_applied_to_active` — `.highlight_style(active)`
   paints the currently selected title.
6. `tabs_padding_default` — default `" "` padding separates titles
   from the divider.

End-to-end coverage lives in `examples/multi-tab-log`, which composes
`Tabs` over a `Vec<TabLog>` for a 5-stream concurrent log viewer.

## Upstream

`ratatui` v0.30.0: https://github.com/ratatui/ratatui

Upstream widget file:
https://github.com/ratatui/ratatui/blob/ratatui-v0.30.0/src/widgets/tabs.rs
