# Arniko Component Library

Arniko provides a rich set of UI components designed specifically for Exosphere capsules. These components are theme-aware, responsive, and render to HTML strings. For design tokens, theme system, and full API tables, see `docs/DESIGN_SYSTEM.md`.

## Core Components

### Button
The primary interaction element.
- **Variants**: `Default`, `Destructive`, `Outline`, `Secondary`, `Ghost`, `Accent`
- **Sizes**: `Default`, `Sm`, `Lg`, `Icon`
- **Builder**: `.variant(..)`, `.size(..)`, `.disabled(..)`, `.class(..)`

### Card
Versatile content container.
- **Builder**: `.title(..)`, `.body(..)`, `.class(..)`
- **Usage**:
  ```rust
  Card::new().title("Header").body("Content...");
  ```

### Input
Standard form input element.
- **Types**: set via `.input_type("text"|"password"|"email"|"number"|...)`
- **Builder**: `.placeholder(..)`, `.value(..)`, `.name(..)`, `.disabled(..)`, `.class(..)`

### Alert
Contextual feedback messages.
- **Variants**: `Info`, `Success`, `Warning`, `Error`
- **Builder**: `.variant(..)`, `.class(..)`

### Badge
Small status indicators or labels.
- **Variants**: `Default`, `Success`, `Warning`, `Error`, `Info`, `Purple`
- **Builder**: `.variant(..)`, `.class(..)`

## Layout & Specialized Components

### MetricCard
Dashboard data display with icon and trend.
- **Colors**: `Blue`, `Green`, `Purple`, `Orange`, `Red`, `Cyan`
- **Trends**: `Stable`, `Up`, `Down`

### ProgressBar
Visual progress tracking.
- **Colors**: `Accent`, `Blue`, `Green`, `Purple`, `Orange`, `Red`

### StatusGrid + StatusIndicator
Grid of status indicators.
- **States**: `Active`, `Warning`, `Error`, `Idle`, `Offline`
- **Option**: pulsing indicator via `.pulse(true)`

### Tooltip
Hover tooltip wrapper.
- **Positions**: `Top`, `Bottom`, `Left`, `Right`

### Spinner
Loading indicator.
- **Sizes**: `Default`, `Sm`, `Lg`

### Skeleton + SkeletonCard
Content placeholders for loading states.

### Separator
Horizontal divider.

### Kbd
Keyboard key display.

---

## Styling Architecture

Arniko uses a centralized CSS theme system defined in `arniko::theme`.

- **Tokens**: Colors, spacing, shadows, and typography are all managed via CSS variables.
- **Glassmorphism**: Built-in support for frosted glass effects using the Mustang GPU compositor.
- **Dark Mode**: First-class support for dark themes, toggled via the `.dark` class or system preferences.
