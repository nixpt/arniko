# Decisions

## arniko-crush: variadic argc for optional args
- **Date:** 2026-06-15
- **Decision:** Use `argc: None` in HostCapSpec for capabilities with optional arguments (button, card, alert, badge, input)
- **Rationale:** Allows programs to call with fewer args; the host capability parses based on args.len(). Fixed argc would require multiple cap names or padding with Null.
- **Tradeoff:** Slightly more complex parsing in cap implementations; no compile-time arity checking.

## arniko-crush: no modification to crush-lang-sdk
- **Date:** 2026-06-15
- **Decision:** arniko-crush exposes its own `register()` function instead of adding a method to HostCapsBuilder
- **Rationale:** Avoids circular dependency; keeps crush-lang-sdk dependency-light. Users call `arniko_crush::register(&mut host_caps)` after building HostCaps.
- **Tradeoff:** Slightly more verbose setup; cleaner separation of concerns.

## capsule-ui: embedded CSS vs runtime fetch
- **Date:** 2026-06-15
- **Decision:** Embed minimal arniko CSS directly in CrushMarkup component rather than fetching at runtime
- **Rationale:** No network dependency; works offline; SSR-compatible. CSS is ~2KB minified.
- **Tradeoff:** Duplicates arniko CSS (could drift if arniko updates). Production apps should load full arniko stylesheet via `arniko.styles` capability and disable `includeStyles`.
