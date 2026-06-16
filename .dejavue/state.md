# State

## Current: Phase 1 complete — capsule-ui portable Crush version

Expanded arniko-crush with 12 new host capabilities covering all portable capsule-ui components.

### New arniko components added

| Component | File | Description |
|-----------|------|-------------|
| StatusBadge | `status_badge.rs` | Badge with colored dot + pulse animation |
| EmptyState | `empty_state.rs` | Placeholder for empty content |
| Panel | `panel.rs` | Container with title bar + body |

### New arniko-crush host caps (29 total, 12 new)

| Capability | Args | Description |
|---|---|---|
| `arniko.separator` | none | Horizontal rule |
| `arniko.spinner` | [size] | Loading spinner (default/sm/lg) |
| `arniko.kbd` | key | Keyboard shortcut display |
| `arniko.status_badge` | label, [variant], [pulse] | Status indicator badge |
| `arniko.empty_state` | title, [icon], [desc], [action] | Empty content placeholder |
| `arniko.panel` | [title], [body], [closable] | Container panel |
| `arniko.toast` | message, [variant] | Toast notification |
| `arniko.metric_card` | title, value, [color] | Metric display card |
| `arniko.progress_bar` | value, [max], [color] | Progress indicator |
| `arniko.tooltip` | text, tooltip, [position] | CSS-only tooltip |
| `arniko.glass_card` | [title], [body] | Glass morphism card |

### Glass morphism CSS variant

Added `.arniko-glass` class to `card.css`:
- `background: rgba(0,0,0,0.3)`
- `border: 1px solid rgba(255,255,255,0.1)`
- `backdrop-filter: blur(12px)`

### CrushMarkup CSS bridge updated

Full Exosphere → arniko variable mapping:
- `--arniko-bg-surface` → `var(--card)`
- `--arniko-text-primary` → `var(--foreground)`
- `--arniko-accent` → `var(--primary)`
- `--arniko-success` → `var(--success)`
- `--arniko-error` → `var(--destructive)`
- All component-specific overrides for theme consistency

### Test results

- arniko-crush: 29/29 tests pass
- arniko components: 40/40 tests pass
- crush-lang-sdk: 11/11 tests pass + 2 doctests
- TypeScript: compiles cleanly
