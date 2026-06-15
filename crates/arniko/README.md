<p align="center">
  <img src="assets/arniko_logo.png" width="200" alt="Arniko Logo">
</p>

Arniko provides a comprehensive UI framework for Exosphere capsules, combining HTML component generation, native window management, and GPU-accelerated visual effects via the Mustang compositor.


## 🚀 Status

**Phase 1: Foundation** ✅ **COMPLETE**
- ✅ Basic crate structure and configuration
- ✅ Core component library (Button, Card, Input, Alert, Badge)
- ✅ Component trait system for unified rendering
- ✅ HTML string generation with embedded CSS
- ✅ Configuration system for different rendering modes
- ✅ Basic testing and validation

**Phase 2: Compositor Integration** ✅ **COMPLETE**
- ✅ CPU-based compositor for synthetic CSS effects
- ✅ Effect types (blur, transform, color adjust, clip)
- ✅ Region handling and geometry operations
- ✅ Quality presets and configuration options
- ✅ Theme-aware compositor integration
- ✅ Security gating and component effects

**Phase 3: Mustang GPU Compositor** ✅ **COMPLETE**
- ✅ GPU-accelerated effect compositor architecture
- ✅ Scene-native effects (blur, transforms)
- ✅ Vello scene integration
- ✅ GPU compute shader framework
- ✅ Effect caching and performance optimization
- ✅ Hybrid CPU/GPU processing modes

**Phase 4: Window Management** ✅ **COMPLETE**
- ✅ Cross-platform window abstractions (Window, WindowConfig, WindowManager)
- ✅ Event handling system (WindowEvent, KeyboardEvent, PointerEvent)
- ✅ Multi-backend rendering support (HTML, Native, Hybrid)
- ✅ Frame buffer integration for GPU rendering
- ✅ Window lifecycle management (create, show, hide, close)
- ✅ Input event processing with modifiers and key states

**Phase 5: Bliss-Core & Networking** 🚧 **IN PROGRESS**
- ✅ "Blissification": Successful repo/branch rename from DioxusLabs/Blitz to nixpt/Bliss-Engine.
- ✅ Exosphere-native Networking (`exo-bliss-net` support for `exo://` and `capsule://`).
- ✅ Event Bridging: DOM event propagation to Exosphere ServiceEventBus.
- 🚧 Stable Hybrid Pipeline: Coordinating Mustang GPU effects with Bliss-DOM layout.

## 📦 Features

### Current Features (v0.3.0)

- **HTML Components**: Generate HTML strings with embedded CSS
- **Component System**: Unified trait for all UI components
- **Styling**: Complete CSS framework with dark theme support
- **Configuration**: Flexible configuration for different use cases
- **Builder Pattern**: Fluent API for component construction
- **Compositor Effects**: CPU-based post-processing for synthetic CSS features
- **GPU Acceleration**: Hardware-accelerated effects via Mustang compositor
- **Visual Effects**: Blur, transforms, color adjustments, security clipping
- **Window Management**: Cross-platform window creation and management
- **Event Handling**: Comprehensive input event system (keyboard, pointer, window)
- **Multi-Backend Rendering**: HTML, Native, and Hybrid rendering modes
- **Frame Buffer Integration**: GPU-accelerated frame buffer for native rendering
- **Performance**: Effect caching and optimized processing pipelines

### Components Available

- **Button**: Multiple variants (Default, Destructive, Outline, Secondary, Ghost, Accent) and sizes
- **Card**: Title and body content with custom styling
- **Input**: Form inputs with various types and validation states
- **Alert**: Notification messages with different severity levels
- **Badge**: Status indicators and labels
- **MetricCard**: Data display with icons and trends
- **ProgressBar**: Progress indicators with labels and colors
- **StatusGrid**: Grid layout for status indicators
- **Tooltip**: Hover tooltips with positioning
- **Spinner**: Loading indicators
- **Skeleton**: Content placeholders
- **Separator**: Visual dividers
- **Kbd**: Keyboard key display

### Visual Effects (New)

- **Backdrop Blur**: GPU-accelerated Gaussian blur for glass morphism effects
- **2D Transforms**: Scale, translate, rotate operations with GPU acceleration
- **Color Adjustment**: Real-time color multipliers and offsets
- **Security Clipping**: Capability-based content masking
- **Scene Composition**: Zero-copy Vello scene integration
- **Effect Caching**: Performance optimization for repeated effects

## 🛠️ Usage

### HTML-only Usage (Lightweight)

```rust
use arniko::{ArnikoApp, Button, ButtonVariant, Card};

let html = ArnikoApp::html()
    .title("My App")                                               // sets <title>
    .component(Button::new("Click me").variant(ButtonVariant::Accent))  // component via trait
    .component(Card::new().title("Status").body("Online"))
    .html_content("<p>Raw HTML block</p>")                        // or raw HTML
    .base_styles(true)                                             // * { box-sizing... }
    .style("body { font-family: Arial, sans-serif; }")            // extra CSS
    .include_arniko_styles(true)                                   // on by default
    .render();
```

### Component Usage

```rust
use arniko::{Button, ButtonSize, ButtonVariant, Card, Alert, Badge, BadgeVariant};

// Standard button
let button = Button::new("Click me")
    .variant(ButtonVariant::Accent)
    .size(ButtonSize::Lg)
    .class("custom-button");

// Link button (renders as `<a>` element)
let link_btn = Button::link("Visit Docs", "/docs")
    .variant(ButtonVariant::Outline)
    .render();
// → <a href="/docs" class="arniko-btn arniko-btn-outline">Visit Docs</a>

// Link button with small size
let small_link = Button::link("Cancel", "/cancel")
    .variant(ButtonVariant::Ghost)
    .size(ButtonSize::Sm)
    .class("inline-link")
    .render();

// Card with content
let card = Card::new()
    .title("Dashboard")
    .body("System status: Online")
    .class("status-card");

// Alert notification
let alert = Alert::new("Operation completed")
    .variant(arniko::alert::AlertVariant::Success);

// Badge indicator
let badge = Badge::new("Active")
    .variant(BadgeVariant::Success);
```

## 🏗️ Architecture

```
Arniko Framework
├── Components (Multi-renderer)
│   ├── HTML string generation
│   ├── Native UI elements
│   └── GPU-accelerated effects
├── Mustang (GPU Compositor)
│   ├── Hardware-accelerated effects
│   └── Scene composition (Vello)
├── Compositor (Effects Processor)
│   ├── CSS synthetic features
│   └── Post-processing
├── Windowing (Native Windows)
│   ├── Cross-platform window management
│   ├── Window lifecycle (create, show, hide, close)
│   └── Event handling system
└── Rendering (Multi-Backend)
    ├── HTML rendering backend
    ├── Native GPU rendering backend
    └── Hybrid rendering pipeline
```

## 🔮 Roadmap

### Phase 5: Bliss-Core & Networking 🚧 **IN PROGRESS**
- [x] Repository "Blissification" (Fork, Rename, Branch Cleanup)
- [x] `exo-bliss-net` Integration (XIP and Capsule scheme support)
- [x] DOM-to-Bus Event Bridge
- [ ] Unified Scene Scheduling (Mustang + Bliss-DOM synchronization)
- [ ] Direct DOM Mutation API (dom-capability native path)

### Phase 6: Advanced Features (Upcoming)
- [ ] **Mustang Animation System**: Dedicated GPU passes for high-performance motion.
- [ ] **Accessibility**: First-class AccessKit integration for screen readers.
- [ ] **Developer Experience**: Real-time effect hot-reloading and Mustang shader debugger.
- [ ] **Advanced Caching**: Regional dirty-rect tracking for sparse scene updates.

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

Run component tests specifically:

```bash
cargo test --package arniko
```

## 📝 Examples

### Full page with ArnikoHtmlBuilder

```rust
use arniko::{ArnikoApp, Button, ButtonVariant, Card, Badge, BadgeVariant};

let html = ArnikoApp::html()
    .title("System Dashboard")
    .base_styles(true)
    .component(
        Card::new()
            .title("Server Status")
            .body("All systems operational")
    )
    .component(
        Button::link("Open Dashboard", "/dashboard")
            .variant(ButtonVariant::Accent)
    )
    .component(
        Badge::new("Online")
            .variant(BadgeVariant::Success)
    )
    .style(r#"
        body { font-family: system-ui, sans-serif; padding: 32px; }
        h1 { color: #a78bfa; }
    "#)
    .render();

// Produces a complete <!DOCTYPE html> with:
// - <title>System Dashboard</title>
// - Arniko base component styles (.arniko-btn, .arniko-card, etc.)
// - CSS reset (* { box-sizing... })
// - Custom page styles
// - All component HTML in the body
```

### Builder with component and raw HTML

```rust
use arniko::ArnikoApp;

let html = ArnikoApp::html()
    .title("Hybrid Page")
    .include_arniko_styles(true)  // on by default
    .style("p { color: #71717a; }")            // extra CSS
    .html_content("<h1>Welcome</h1>")           // raw HTML block
    .html_content("<p>Mixed with components</p>")
    .render();
```

### Link button standalone

```rust
use arniko::{Button, ButtonVariant, ButtonSize};

// As an <a> link styled like a button
let link = Button::link("Visit", "/page")
    .variant(ButtonVariant::Outline)
    .render();

// Small ghost link button
let small = Button::link("Cancel", "/cancel")
    .variant(ButtonVariant::Ghost)
    .size(ButtonSize::Sm)
    .render();
```

### Chaining all builder options

```rust
use arniko::ArnikoApp;

let html = ArnikoApp::html()
    .title("My App")
    .base_styles(true)                           // CSS reset
    .include_arniko_styles(true)                 // Arniko components CSS
    .style("body { max-width: 800px; }")        // extra CSS
    .html_content("<header>App Header</header>")
    .render();
```

## 📚 Documentation

- **Design system**: `docs/DESIGN_SYSTEM.md` (tokens, themes, CSS classes)
- **Component catalog**: `docs/COMPONENTS.md`
- **Examples source**: See `examples/` directory (`basic.rs`, `builder.rs`, `advanced.rs`) for runnable usage patterns
- **Architecture**: Detailed design documentation in `docs/`

## 🤝 Contributing

This is a gradual migration project. When contributing:

1. **Keep existing system intact**: Don't break bliss-ui or capsule-ui-rs
2. **Follow the migration plan**: Implement features in the planned phases
3. **Test thoroughly**: Ensure all components work as expected
4. **Document changes**: Update documentation for new features

## 📄 License

MIT License - see LICENSE file for details

## 🙏 Acknowledgments

Arniko builds upon the excellent work of:
- **bliss-ui**: HTML component library foundation
- **capsule-ui-rs**: Window management patterns
- **super-surfer**: Compositor and Mustang GPU effects
- **bliss-core**: Browser engine integration
- **Exosphere team**: Agent-OS architecture and vision