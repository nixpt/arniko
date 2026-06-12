# Arniko Design System

> Living documentation for the Arniko UI framework. Covers design tokens, component APIs, themes, and composition patterns.
> 
> **Scope:** HTML/component mode (`html` + `components` features). GPU and networking modes are documented separately in `GPU_ACCELERATION.md`.

---

## Table of Contents

- [Design Tokens](#design-tokens)
- [Component Catalog](#component-catalog)
- [Theme System](#theme-system)
- [CSS Class Reference](#css-class-reference)
- [Composition Patterns](#composition-patterns)
- [Feature Flags](#feature-flags)

---

## Design Tokens

Arniko uses CSS custom properties (variables) for all visual values. Themes override these variables via `ThemeManager::generate_css()`.

### Color Palette

| Token | Dark | Light | Frosted | Cyberpunk | Aurora |
|-------|------|-------|---------|-----------|--------|
| `--bg-void` | `#020205` | `#fafafa` | `#f8fafc` | `#05050a` | `#f5f3ff` |
| `--bg-surface` | `#0a0a12` | `#f4f4f5` | `#e2e8f0` | `#0a0a14` | `#f0f4ff` |
| `--bg-elevated` | `#12121f` | `#ffffff` | `#ffffff` | `#151525` | `#ffffff` |
| `--bg-input` | `#0f0f1a` | `#ffffff` | `#ffffff` | `#101020` | `#ffffff` |
| `--text-primary` | `#f5f5f5` | `#18181b` | `#0f172a` | `#ffffff` | `#1f2937` |
| `--text-secondary` | `#d4d4d8` | `#3f3f46` | `#475569` | `#d1fae5` | `#4b5563` |
| `--text-muted` | `#a1a1aa` | `#71717a` | `#64748b` | `#9ca3af` | `#9ca3af` |
| `--text-accent` | `#c4b5fd` | `#6366f1` | `#3b82f6` | `#22d3ee` | `#ec4899` |
| `--accent` | `#a78bfa` | `#6366f1` | `#3b82f6` | `#22d3ee` | `#ec4899` |
| `--accent-hover` | `#c4b5fd` | `#818cf8` | `#60a5fa` | `#60a5fa` | `#f472b6` |
| `--border` | `rgba(255,255,255,0.06)` | `rgba(0,0,0,0.08)` | `#94a3b8` | `rgba(16,185,129,0.3)` | `rgba(147,51,234,0.25)` |
| `--border-accent` | `rgba(167,139,250,0.3)` | `rgba(99,102,241,0.3)` | `#3b82f6` | `#22d3ee` | `#ec4899` |

### Semantic Status Colors

| Token | Dark | Light |
|-------|------|-------|
| `--status-ok` | `#4ade80` | `#16a34a` |
| `--status-warn` | `#fbbf24` | `#d97706` |
| `--status-error` | `#f87171` | `#dc2626` |
| `--status-info` | `#60a5fa` | `#2563eb` |

### Spacing

Arniko uses a 4px base grid. Common spacing values:

| Value | Usage |
|-------|-------|
| `4px` | Inline padding, badge padding |
| `8px` | Button gap, card padding step |
| `12px` | Input padding, status item padding |
| `16px` | Standard padding, separator margin |
| `20px` | Card padding, metric card padding |
| `24px` | Large card padding |

### Border Radius

| Token | Value | Usage |
|-------|-------|-------|
| Small | `3px` | Kbd keys |
| Default | `4px` | Badges, skeletons |
| Medium | `6px` | Inputs, tooltips |
| Large | `8px` | Buttons, cards, alerts |
| XL | `10px` | Metric card icons |
| Full | `999px` | Progress bars, spinners |

### Shadows (Glassmorphism)

| Token | Value |
|-------|-------|
| `--glass-shadow` | `0 4px 24px rgba(0,0,0,0.2)` (dark) / `0 4px 16px rgba(0,0,0,0.06)` (light) |
| `--glass-shadow-hover` | `0 12px 40px rgba(0,0,0,0.35)` (dark) / `0 8px 32px rgba(0,0,0,0.1)` (light) |

### Typography

| Element | Size | Weight | Color |
|---------|------|--------|-------|
| Card title | `13px` | `400` | `--text-muted` |
| Card body | `16px` | `400` | `--text-primary` |
| Button | `14px` | `500` | inherited |
| Badge | `12px` | `500` | inherited |
| Metric value | `28px` | `700` | `#fff` / `#18181b` |
| Metric title | `12px` | `400` | `--text-muted` |
| Input | `14px` | `400` | `--text-primary` |

---

## Component Catalog

All components implement the `Component` trait:

```rust
pub trait Component {
    fn render(&self) -> String;
    fn metadata(&self) -> ComponentMetadata;
}
```

Every component supports `.class("...")` for custom CSS classes.

### Button

Primary interaction element.

```rust
use arniko::{Button, ButtonVariant, ButtonSize};

let btn = Button::new("Save")
    .variant(ButtonVariant::Accent)
    .size(ButtonSize::Lg)
    .disabled(false)
    .class("ml-2");
```

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(label)` | `&str` | required | Button text |
| `.variant(v)` | `ButtonVariant` | `Default` | Visual style |
| `.size(s)` | `ButtonSize` | `Default` | Size |
| `.disabled(d)` | `bool` | `false` | Disabled state |
| `.class(c)` | `&str` | `""` | Extra CSS classes |

**Variants:** `Default`, `Destructive`, `Outline`, `Secondary`, `Ghost`, `Accent`  
**Sizes:** `Default`, `Sm`, `Lg`, `Icon`

### Card

Content container.

```rust
use arniko::Card;

let card = Card::new()
    .title("System Status")
    .body("All systems operational")
    .class("dashboard-card");
```

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Empty card |
| `.title(t)` | `&str` | `None` | Card header |
| `.body(b)` | `&str` | `None` | Card content |
| `.class(c)` | `&str` | `""` | Extra CSS classes |

### Input

Form input element.

```rust
use arniko::Input;

let input = Input::new()
    .input_type("email")
    .placeholder("user@example.com")
    .name("email")
    .value("")
    .disabled(false);
```

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Empty input |
| `.input_type(t)` | `&str` | `"text"` | HTML input type |
| `.placeholder(p)` | `&str` | `""` | Placeholder text |
| `.value(v)` | `&str` | `""` | Current value |
| `.name(n)` | `&str` | `""` | Input name attribute |
| `.disabled(d)` | `bool` | `false` | Disabled state |
| `.class(c)` | `&str` | `""` | Extra CSS classes |

### Alert

Contextual feedback messages with embedded icons.

```rust
use arniko::{Alert, AlertVariant};

let alert = Alert::new("Deployment successful")
    .variant(AlertVariant::Success);
```

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(msg)` | `&str` | required | Alert message |
| `.variant(v)` | `AlertVariant` | `Info` | Severity style |
| `.class(c)` | `&str` | `""` | Extra CSS classes |

**Variants:** `Info` (ℹ️), `Success` (✅), `Warning` (⚠️), `Error` (❌)

### Badge

Small status indicators.

```rust
use arniko::{Badge, BadgeVariant};

let badge = Badge::new("Live")
    .variant(BadgeVariant::Success);
```

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(text)` | `&str` | required | Badge text |
| `.variant(v)` | `BadgeVariant` | `Default` | Color style |
| `.class(c)` | `&str` | `""` | Extra CSS classes |

**Variants:** `Default`, `Success`, `Warning`, `Error`, `Info`, `Purple`

### MetricCard

Dashboard data display with icon and trend.

```rust
use arniko::{MetricCard, MetricColor, MetricTrend};

let metric = MetricCard::new("CPU Usage", "42%")
    .subtitle("8 cores")
    .icon("🖥️")
    .color(MetricColor::Purple)
    .trend(MetricTrend::Up)
    .class("col-span-2");
```

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(title, value)` | `(&str, &str)` | required | Title and value |
| `.subtitle(s)` | `&str` | `None` | Subtitle text |
| `.icon(i)` | `&str` | `None` | Icon/emoji |
| `.color(c)` | `MetricColor` | `Blue` | Accent color |
| `.trend(t)` | `MetricTrend` | `Stable` | Trend indicator |
| `.class(c)` | `&str` | `""` | Extra CSS classes |

**Colors:** `Blue`, `Green`, `Purple`, `Orange`, `Red`, `Cyan`  
**Trends:** `Stable`, `Up` (↑), `Down` (↓)

### ProgressBar

Progress indicator with label and percentage.

```rust
use arniko::{ProgressBar, ProgressColor};

let bar = ProgressBar::new(73.5)
    .max(100.0)
    .label("Uploading...")
    .color(ProgressColor::Green)
    .show_percentage(true);
```

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(value)` | `f32` | required | Current value |
| `.max(m)` | `f32` | `100.0` | Maximum value |
| `.label(l)` | `&str` | `None` | Label text |
| `.color(c)` | `ProgressColor` | `Accent` | Bar color |
| `.show_percentage(s)` | `bool` | `true` | Show `%` |
| `.class(c)` | `&str` | `""` | Extra CSS classes |

**Colors:** `Accent`, `Blue`, `Green`, `Purple`, `Orange`, `Red`

### StatusGrid + StatusIndicator

Grid of status dots with labels.

```rust
use arniko::{StatusGrid, StatusIndicator, StatusState};

let grid = StatusGrid::new()
    .columns(2)
    .add(StatusIndicator::new("Daemon", "Running").state(StatusState::Active).pulse(true))
    .add(StatusIndicator::new("Mesh", "Degraded").state(StatusState::Warning))
    .add(StatusIndicator::new("Vault", "Locked").state(StatusState::Error));
```

**StatusIndicator:**

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(label, value)` | `(&str, &str)` | required | Label and value |
| `.state(s)` | `StatusState` | `Active` | Dot color |
| `.pulse(p)` | `bool` | `false` | Pulsing animation |
| `.class(c)` | `&str` | `""` | Extra CSS classes |

**StatusGrid:**

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Empty grid |
| `.add(item)` | `StatusIndicator` | — | Add item |
| `.columns(n)` | `u32` | `1` | Grid columns |
| `.class(c)` | `&str` | `""` | Extra CSS classes |

**States:** `Active` (green), `Warning` (yellow), `Error` (red), `Idle` (purple), `Offline` (gray)

### Tooltip

Hover tooltip wrapper.

```rust
use arniko::{Tooltip, TooltipPosition};

let tip = Tooltip::new("Hover me", "This is the tooltip text")
    .position(TooltipPosition::Bottom);
```

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(text, tooltip)` | `(&str, &str)` | required | Visible text + tooltip |
| `.position(p)` | `TooltipPosition` | `Top` | Tooltip position |
| `.class(c)` | `&str` | `""` | Extra CSS classes |

**Positions:** `Top`, `Bottom`, `Left`, `Right`

### Spinner

Loading indicator.

```rust
use arniko::{Spinner, SpinnerSize};

let spinner = Spinner::new().size(SpinnerSize::Lg);
```

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default spinner |
| `.size(s)` | `SpinnerSize` | `Default` | Size |
| `.class(c)` | `&str` | `""` | Extra CSS classes |

**Sizes:** `Default`, `Sm`, `Lg`

### Skeleton + SkeletonCard

Content placeholders for loading states.

```rust
use arniko::{Skeleton, SkeletonCard};

let line = Skeleton::new().width("100%").height("16px");
let card = SkeletonCard::new().class("w-64");
```

**Skeleton:**

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Default skeleton |
| `.width(w)` | `&str` | `"100%"` | CSS width |
| `.height(h)` | `&str` | `"20px"` | CSS height |
| `.class(c)` | `&str` | `""` | Extra CSS classes |

**SkeletonCard:** No builder methods except `.class()`.

### Separator

Visual divider.

```rust
use arniko::Separator;

let sep = Separator::new().class("my-4");
```

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new()` | — | — | Horizontal rule |
| `.class(c)` | `&str` | `""` | Extra CSS classes |

### Kbd

Keyboard key display.

```rust
use arniko::Kbd;

let key = Kbd::new("Ctrl + S").class("text-xs");
```

| Method | Type | Default | Description |
|--------|------|---------|-------------|
| `new(key)` | `&str` | required | Key label |
| `.class(c)` | `&str` | `""` | Extra CSS classes |

---

## Theme System

Arniko ships 6 built-in themes. Themes are applied by generating CSS variable overrides.

```rust
use arniko::{ThemeManager, ThemeMode};

let mut manager = ThemeManager::new();
let css = manager.generate_css(&ThemeMode::Dark, arniko::ARNIKO_STYLES);
```

### Available Themes

| Theme | Description | Best For |
|-------|-------------|----------|
| `Dark` | Default dark, purple accents | General admin dashboards |
| `Light` | Clean light, indigo accents | Daytime use, documentation |
| `System` | Follows `prefers-color-scheme` | User preference respect |
| `Frosted` | White/gray glass, blue accents | Clean minimal UIs |
| `Cyberpunk` | Neon cyan/green on black | High-contrast terminal aesthetic |
| `Aurora` | Pastel pink/purple on light | Soft, friendly interfaces |

### ThemeManager API

| Method | Description |
|--------|-------------|
| `ThemeManager::new()` | Create manager with empty cache |
| `.generate_css(theme, base_css)` | Combine base styles + theme overrides (cached) |
| `.get_cached(theme)` | Retrieve cached CSS if available |
| `.clear_cache()` | Drop all cached themes |

### Usage in HTML

```rust
let themed_css = manager.generate_css(&ThemeMode::Dark, arniko::ARNIKO_STYLES);

let html = format!(r#"
<!DOCTYPE html>
<html>
<head><style>{}</style></head>
<body>
    {}
</body>
</html>
"#, themed_css, my_component.render());
```

---

## CSS Class Reference

All Arniko CSS classes are prefixed with `arniko-` to avoid collisions.

### Layout

| Class | Element | Description |
|-------|---------|-------------|
| `.arniko-card` | `div` | Content container |
| `.arniko-card-title` | `div` | Card header |
| `.arniko-card-body` | `div` | Card content |
| `.arniko-metric-card` | `div` | Metric container |
| `.arniko-metric-header` | `div` | Metric header row |
| `.arniko-metric-info` | `div` | Metric text column |
| `.arniko-metric-icon` | `div` | Metric icon container |
| `.arniko-status-grid` | `div` | CSS grid container |
| `.arniko-status-item` | `div` | Grid cell |
| `.arniko-progress` | `div` | Progress wrapper |
| `.arniko-progress-header` | `div` | Label + percentage row |
| `.arniko-progress-track` | `div` | Progress background track |
| `.arniko-progress-bar` | `div` | Filled bar |

### Buttons

| Class | Description |
|-------|-------------|
| `.arniko-btn` | Base button styles |
| `.arniko-btn-default` | Primary blue |
| `.arniko-btn-destructive` | Red |
| `.arniko-btn-outline` | Transparent with border |
| `.arniko-btn-secondary` | Gray |
| `.arniko-btn-ghost` | Transparent |
| `.arniko-btn-accent` | Purple |
| `.arniko-btn-sm` | Small size |
| `.arniko-btn-lg` | Large size |
| `.arniko-btn-icon` | Square icon button |

### Forms

| Class | Description |
|-------|-------------|
| `.arniko-input` | Text input base |

### Feedback

| Class | Description |
|-------|-------------|
| `.arniko-alert` | Alert container |
| `.arniko-alert-info` | Blue alert |
| `.arniko-alert-success` | Green alert |
| `.arniko-alert-warning` | Yellow alert |
| `.arniko-alert-error` | Red alert |
| `.arniko-badge` | Badge base |
| `.arniko-badge-default` | Gray badge |
| `.arniko-badge-success` | Green badge |
| `.arniko-badge-warning` | Yellow badge |
| `.arniko-badge-error` | Red badge |
| `.arniko-badge-info` | Blue badge |
| `.arniko-badge-purple` | Purple badge |
| `.arniko-spinner` | Loading spinner |
| `.arniko-skeleton` | Shimmer placeholder |
| `.arniko-skeleton-card` | Skeleton card layout |

### Status

| Class | Description |
|-------|-------------|
| `.arniko-status-active` | Green dot |
| `.arniko-status-warning` | Yellow dot |
| `.arniko-status-error` | Red dot |
| `.arniko-status-idle` | Purple dot |
| `.arniko-status-offline` | Gray dot |
| `.arniko-status-pulse` | Pulsing animation |
| `.arniko-trend-up` | Green upward |
| `.arniko-trend-down` | Red downward |
| `.arniko-trend-stable` | Yellow neutral |

### Metric Colors

| Class | Gradient |
|-------|----------|
| `.arniko-metric-blue` | `#3b82f6` → `#06b6d4` |
| `.arniko-metric-green` | `#22c55e` → `#10b981` |
| `.arniko-metric-purple` | `#a855f7` → `#ec4899` |
| `.arniko-metric-orange` | `#f97316` → `#ef4444` |
| `.arniko-metric-red` | `#ef4444` → `#dc2626` |
| `.arniko-metric-cyan` | `#06b6d4` → `#0ea5e9` |

### Progress Colors

| Class | Gradient |
|-------|----------|
| `.arniko-progress-accent` | `#a855f7` → `#ec4899` |
| `.arniko-progress-blue` | `#3b82f6` → `#06b6d4` |
| `.arniko-progress-green` | `#22c55e` → `#10b981` |
| `.arniko-progress-purple` | `#a855f7` → `#8b5cf6` |
| `.arniko-progress-orange` | `#f97316` → `#eab308` |
| `.arniko-progress-red` | `#ef4444` → `#f97316` |

### Misc

| Class | Description |
|-------|-------------|
| `.arniko-separator` | Horizontal rule |
| `.arniko-kbd` | Keyboard key style |
| `.arniko-tooltip` | Tooltip trigger |

---

## Composition Patterns

### Dashboard Panel

```rust
use arniko::{Card, MetricCard, MetricColor, MetricTrend, Badge, BadgeVariant, StatusGrid, StatusIndicator, StatusState};

let panel = Card::new()
    .title("Cluster Overview")
    .body(&format!(
        "{}{}{}{}",
        MetricCard::new("Nodes", "12").color(MetricColor::Blue).render(),
        MetricCard::new("Pods", "84").color(MetricColor::Purple).trend(MetricTrend::Up).render(),
        Badge::new("Healthy").variant(BadgeVariant::Success).render(),
        StatusGrid::new()
            .columns(2)
            .add(StatusIndicator::new("API", "OK").state(StatusState::Active))
            .add(StatusIndicator::new("DB", "Syncing").state(StatusState::Warning).pulse(true))
            .render()
    ));
```

### Form Card

```rust
use arniko::{Card, Input, Button, ButtonVariant};

let form = Card::new()
    .title("Login")
    .body(&format!(
        "{}{}{}",
        Input::new().input_type("email").placeholder("Email").name("email").render(),
        Input::new().input_type("password").placeholder("Password").name("password").render(),
        Button::new("Sign In").variant(ButtonVariant::Accent).render()
    ));
```

### Loading State

```rust
use arniko::{SkeletonCard, Spinner, SpinnerSize};

let loading = format!(
    "{}{}{}",
    Spinner::new().size(SpinnerSize::Lg).render(),
    SkeletonCard::new().render(),
    SkeletonCard::new().render()
);
```

### Alert Stack

```rust
use arniko::{Alert, AlertVariant};

let alerts = vec![
    Alert::new("Service restarted").variant(AlertVariant::Success).render(),
    Alert::new("High memory usage").variant(AlertVariant::Warning).render(),
].join("\n");
```

---

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `html` | ✅ | HTML string generation |
| `components` | ✅ | UI components (requires `html`) |
| `gpu` | ❌ | GPU acceleration via Mustang |
| `networking` | ❌ | Exosphere networking (`exo-bliss-net`) |
| `full` | ❌ | All features enabled |

**Cargo.toml:**

```toml
[dependencies]
arniko = { path = "../arniko", features = ["components"] }          # HTML only
arniko = { path = "../arniko", features = ["full"] }                 # Everything
arniko = { path = "../arniko", default-features = false, features = ["html"] }  # Minimal
```

---

## Notes

- **HTML escaping:** Component `render()` methods do NOT escape HTML in user-provided strings. Sanitize inputs before passing to components.
- **Placeholder file:** `src/components/placeholder_components.rs` contains stub implementations that are shadowed by the real modules in `mod.rs`. It exists for backward compatibility during migration and should not be imported directly.
- **GPU effects:** When `gpu` feature is enabled, `ComponentMetadata.requires_gpu` flags components that benefit from Mustang compositor passes. Most core components set this to `false`.
