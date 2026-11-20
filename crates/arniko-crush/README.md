# arniko-crush

Exposes [Arniko](https://github.com/nixpt/arniko) UI components as host
capabilities for the Crush runtime. A Crush capsule builds HTML UI by calling
capability functions such as `arniko.button` and `arniko.card`, then embeds the
returned markup in its output.

```text
.func main
PUSH_STR "Click me"
PUSH_STR "accent"
CAP_CALL "arniko.button" 2
CAP_CALL "io.print" 1
HALT
```

The host registers the capabilities:

```rust,no_run
use crush_lang_sdk::HostCapsBuilder;

let mut host_caps = HostCapsBuilder::new().build();
arniko_crush::register(&mut host_caps);
```

## `dom.*` caps (unreleased)

`register_dom(&mut caps, &host)` adds `dom.*` capabilities bound to one
[noonoo](https://github.com/nixpt/noonoo) document held by a `DomHost`:
`dom.query`, `dom.query_all`, `dom.text`, `dom.attr`, `dom.tag`,
`dom.inner_html`, `dom.set_text`, `dom.set_attr` (the same contract as
surfer's `dom.*` caps), plus `dom.set_style` and `dom.remove_style`, which
edit inline style so the next style computation sees the change.

```rust,no_run
use crush_lang_sdk::HostCapsBuilder;

let host = arniko_crush::DomHost::from_html("<p id=greeting>hi</p>");
let mut caps = HostCapsBuilder::new().build();
arniko_crush::register_dom(&mut caps, &host);
```

A host from `DomHost::new` / `from_html` is ungated (an empty gate with an
explicit `AllowAll`). `DomHost::with_gate(gate, authorizer, subject)` gates it
with [`noonoo-caps`](https://crates.io/crates/noonoo-caps): the nearest guard
at or above a node decides, writes need `Write`, reads need `Read`, and a denied
call returns an `Err` naming the cap and node and leaves the document alone.
Call `with_gate` before `register_dom`.

This is the first step of moving arniko from bliss-engine to noonoo. It uses
the noonoo 0.20 crates from crates.io (`noonoo-dom`, `-html`, `-query`,
`-style`, `-css`, `-caps`).

## License

MIT OR Apache-2.0.
