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
use arniko::{ArnikoApp, Button, Card, Variant};

let html = ArnikoApp::html()
    .component(Button::new("Click me").variant(Variant::Accent))
    .component(Card::new().title("Status").body("Online"))
    .style("body { font-family: Arial, sans-serif; }")
    .render();
```

### Component Usage

```rust
use arniko::{Button, Card, Alert, Badge, Variant};

// Button with custom styling
let button = Button::new("Click me")
    .variant(Variant::Accent)
    .size(arniko::button::ButtonSize::Large)
    .class("custom-button");

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
    .variant(arniko::badge::BadgeVariant::Success);
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

## 📚 Documentation

- **Design system**: `docs/DESIGN_SYSTEM.md` (tokens, themes, CSS classes)
- **Component catalog**: `docs/COMPONENTS.md`
- **Examples**: See `examples/` directory for usage patterns
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