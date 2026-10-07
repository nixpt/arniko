# exo-bliss-net

An Exosphere-flavoured `NetProvider` for the [Bliss](https://github.com/nixpt/bliss-engine)
rendering engine: routes resource requests by URL scheme.

| scheme | handler |
|---|---|
| `data:` | `DataSchemeHandler` |
| `file://` | `FileSchemeHandler` |
| `http://`, `https://` | `HttpSchemeHandler` (reqwest; `http-fallback` feature, on by default) |
| `capsule://` | `CapsuleSchemeHandler` (`capsule` feature; placeholder, currently a no-op) |

```rust,ignore
let provider = exo_bliss_net::ExoNetProvider::builder()
    // add handlers / a fallback provider / a request policy
    .build();
```

Part of [arniko](https://github.com/nixpt/arniko), which enables it through its
`networking` feature.

## License

MIT OR Apache-2.0.
