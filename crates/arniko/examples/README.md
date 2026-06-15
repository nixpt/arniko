# Arniko Examples

Run any example with `cargo run --example <name>`, e.g.:

```bash
cargo run --example basic
```

## Example index

| Example | Features | What it shows |
|---|---|---|
| `basic` | `components` | Standalone rendering of Button, Card, Alert, Badge with variants, sizes, classes, and `Component::metadata()` |
| `builder` | `html` | `ArnikoHtmlBuilder` API: `.title()`, `.base_styles()`, `.component()`, `.html_content()`, `.style()`, `.include_arniko_styles()`, and `Button::link()` |
| `launch` | `launch` | Native window launch via `ArnikoApp::html().launch()` with Alert, Card, Button, Badge |
| `advanced` | `gpu` | Glass morphism, Compositor, Mustang GPU acceleration, theme-aware effects (supersedes the removed `effects.rs`) |
| `blur_panel` | `reactive,gpu` | Mustang blur halo wired into a reactive bliss window |
| `reactive_counter` | `reactive` | Signal + View + EventRouter — increment/decrement/reset counter |
| `computed_counter` | `reactive` | `Signal::derive`, `Computed::from2`, `Computed::map` — derived signals with step control |
| `todo_list` | `reactive` | `For<T, R>` reactive list rendering — add/remove/clear items with auto-updating DOM |
