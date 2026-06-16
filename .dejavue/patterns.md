# Patterns

## Cross-repo host capability registration
- arniko-crush depends on crush-lang-sdk via relative path dep
- `arniko_crush::register(&mut host_caps)` — no modification to crush-lang-sdk
- Pattern: optional features in SDK crates for cross-repo integrations

## Variadic capability arguments
- Use `argc: None` in HostCapSpec for caps with optional args
- Parse based on `args.len()` inside the capability implementation
- Program declares cap name; VM passes all pushed values

## CSS bridge for React/HTML interop
- capsule-ui's `CrushMarkup` renders arniko HTML with embedded minimal CSS
- Exosphere CSS variable overrides applied via `.crush-markup-wrap` class
- Production apps load full arniko stylesheet via `arniko.styles` capability
