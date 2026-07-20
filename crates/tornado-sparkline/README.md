# tornado-sparkline

> **⚠️ Stateless Widget mirror (intentional).**
>
> Upstream ratatui 0.30 ships `Sparkline` as a `Widget` impl with no
> `StatefulWidget` counterpart (no `SparklineState` structure exists).
> **This vendoring mirrors that contract exactly** — only the `Widget`
> impl is exposed, no state carrier, no selection, no scrollable
> sub-region. Round 10 picked Sparkline specifically for its
> stateless surface (lowest vendoring lift per design memo D2).
>
> If a future round adopts a stateful widget (e.g., `List`, `Chart`)
> the StatefulWidget parity pattern will be documented in that
> sibling's `NOTICE` (compare with `crates/tornado-tabs` for the
> StatefulWidget carve-out precedent).

Vendored copy of the `Sparkline` and `SparklineBar` widgets from
`ratatui` 0.30, dual-licensed under `MIT OR Apache-2.0`. See
`LICENSE-MIT`, `LICENSE-APACHE`, and `NOTICE` for the full license
text and upstream attribution.

## What's vendored

* `Sparkline<'a>` — the statless widget builder, fed a `&'a [u64]`
  data slice. Exposes `style(Style)`, `bar_set(SparklineBar)`,
  `direction(Direction)`, `max(u64)` builder methods. The `Widget`
  impl renders the bars in cells across the widget's area.
* `SparklineBar` — the bar style. Carries `symbol(&str)` (defaults
  to `█`) and `style(Style)`. Builder-method chaining returns `Self`.

Exposed through the `tornado` umbrella's `sparkline` feature as:

* `tornado::sparkline::Sparkline` (raw path)
* `tornado::sparkline::SparklineBar` (raw path)
* `tornado::widget::Sparkline` (semantic alias, no rename — `Sparkline`
  is the upstream verb and does not need a TabNav-style rename)
* `tornado::widget::SparklineBar` (semantic alias)

## Usage (via the umbrella)

```rust
use tornado::widget::{Sparkline, SparklineBar};

let data: &[u64] = &[1, 4, 2, 8, 5, 12, 7, 3, 9, 6, 11, 4];
let sparkline = Sparkline::new(data)
    .style(Style::default().fg(Color::DarkGray))
    .bar_set(SparklineBar::new("▁▂▃▄▅▆▇█"));

frame.render_widget(sparkline, meter_area);
```

## Direct usage

```rust
use tornado_sparkline::{Sparkline, SparklineBar};

let sparkline = Sparkline::new(&[1u64, 2, 3, 4])
    .max(10)
    .style(my_style)
    .bar_set(SparklineBar::new("▆").style(my_bar_style));

frame.render_widget(sparkline, area);
```

## Tests

`cargo test -p tornado-sparkline` runs the 6 inline smoke tests +
6 gateway integration tests:

1. `sparkline_new_preserves_data` — `Sparkline::new(&[…])` round-trips
   the data slice.
2. `sparkline_max_default` — `Sparkline::max(u64)` clamps tall values
   to the area height.
3. `sparkline_direction_horizontal` — `.direction(Direction::Horizontal)`
   orients the bar rendering.
4. `sparkline_style_applied_to_widget` — `.style(Style)` paints the
   surrounding canvas.
5. `sparkline_bar_set_applied_to_bars` — `.bar_set(SparklineBar)`
   carries the bar symbol from `SparklineBar::new("…")`.
6. `sparkline_default_symbol` — the default `SparklineBar::new()` is
   `"█"` (render sanity check).

End-to-end coverage lives in `examples/multi-tab-log`, which slots
a 12-col Sparkline into the right flank of the existing 2-row
`status_bar` to report the rolling row-append cadence of the active
tab.

## Upstream

`ratatui` v0.30.0: https://github.com/ratatui/ratatui

Upstream widget file:
https://github.com/ratatui/ratatui/blob/ratatui-v0.30.0/src/widgets/sparkline.rs
