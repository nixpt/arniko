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

## License

MIT OR Apache-2.0.
