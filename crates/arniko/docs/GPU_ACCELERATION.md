# Arniko GPU Acceleration (Mustang)

Arniko integrates the **Mustang** compositor to provide hardware-accelerated visual effects that go beyond standard browser capabilities.

## Architecture

Mustang sits between the Arniko components and the final rendering backend (e.g., Vello). It processes specialized `Effect` types emitted by components and renders them using GPU compute shaders.

### Key Capabilities

1. **Backdrop Blur**: Real-time Gaussian blur used for premium "frosted glass" effects.
2. **2D Transforms**: High-performance scaling, rotation, and translation that doesn't trigger CPU layout recalculations.
3. **Regional Clipping**: Secure, capability-gated masking of content areas, integrated with Exosphere's safety model.
4. **Color Adjustments**: GPU-side color matrix operations for real-time theme shifts and visual feedback.

## Integration

To enable GPU acceleration, the Arniko app must be initialized in `Native` or `Hybrid` mode:

```rust
let app = ArnikoApp::native()
    .with_gpu(true)
    .component(my_component);
```

### Performance Optimization

- **Effect Caching**: Mustang automatically caches complex filter chains to minimize redundant GPU passes.
- **Zero-Copy Pipelines**: Scenes are composed directly in GPU memory using Vello, minimizing data transfer between CPU and RAM.
