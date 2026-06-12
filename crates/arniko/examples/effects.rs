//! Test example for Arniko with compositor and Mustang effects
//!
//! This example demonstrates advanced visual effects capabilities.

#[cfg(feature = "gpu")]
use arniko::{ArnikoApp, Button, Card, Component};
#[cfg(feature = "gpu")]
use arniko::{
    ColorAdjustParams, Effect, MustangCompositor, MustangConfig, MustangMode, Region,
    TransformParams,
};

fn main() {
    #[cfg(feature = "gpu")]
    {
        println!("🐎 Testing Arniko with Mustang GPU Compositor");

        // Test basic HTML generation
        let html = ArnikoApp::html()
            .component(Button::new("Glass Button").class("glass-button"))
            .component(
                Card::new()
                    .title("Glass Card")
                    .body("With blur effect")
                    .class("glass-card"),
            )
            .style(
                "
                body { 
                    font-family: Arial, sans-serif; 
                    padding: 20px; 
                    background: #1a1a1a;
                }
                .glass-button {
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    border: 1px solid rgba(255, 255, 255, 0.2);
                }
                .glass-card {
                    background: rgba(255, 255, 255, 0.05);
                    backdrop-filter: blur(15px);
                    border: 1px solid rgba(255, 255, 255, 0.1);
                }
            ",
            )
            .render();

        println!("Generated HTML with glass effects:");
        println!("{}", html);

        // Test Mustang compositor
        test_mustang_compositor();
    }

    #[cfg(not(feature = "gpu"))]
    {
        println!("⚠️  GPU features not enabled. Use --features gpu to test Mustang compositor.");

        // Test basic compositor without GPU
        test_basic_compositor();
    }
}

#[cfg(feature = "gpu")]
fn test_mustang_compositor() {
    println!("\n=== Testing Mustang GPU Compositor ===");

    let mut mustang = MustangCompositor::new(MustangConfig::gpu_accelerated());

    // Create test effects
    let blur_effect = Effect::blur(".glass-panel", 10.0, 800, 600);
    let transform_effect = Effect::transform(
        ".rotating-card",
        TransformParams {
            scale_x: 1.1,
            scale_y: 1.1,
            rotate_degrees: 15.0,
            ..Default::default()
        },
        800,
        600,
    );

    let effects = vec![blur_effect, transform_effect];

    println!("Created {} effects", effects.len());
    println!("Blur effect: native={}", blur_effect.is_native());
    println!("Transform effect: native={}", transform_effect.is_native());

    // Test effect caching
    mustang.cache_effects("test-component", effects.clone());
    let cached = mustang.get_cached_effects("test-component");

    println!("Cached effects: {}", cached.is_some());
    if let Some(cached) = cached {
        println!("Cached {} effects", cached.len());
    }

    // Test performance stats
    let stats = mustang.get_stats();
    println!("Stats: {:?}", stats);
}

#[cfg(not(feature = "gpu"))]
fn test_basic_compositor() {
    println!("\n=== Testing Basic Compositor ===");

    use arniko::{Compositor, CompositorConfig, Effect, Region};

    let compositor = Compositor::new();
    let buffer = vec![255u8; 100 * 100 * 4]; // RGBA buffer
    let effects = vec![Effect::blur(".test", 5.0, 100, 100)];

    match compositor.composite(&buffer, 100, 100, &effects) {
        Ok(result) => {
            println!("Composited successfully:");
            println!("  - Buffer size: {} bytes", result.buffer.len());
            println!("  - Dimensions: {}x{}", result.width, result.height);
            println!("  - Effects applied: {}", result.effects_applied);
            println!("  - Processing time: {}ms", result.processing_time_ms);
        }
        Err(e) => {
            println!("Compositing failed: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "gpu")]
    use super::*;

    #[test]
    fn test_mustang_compositor_creation() {
        let mustang = MustangCompositor::default();
        assert_eq!(mustang.config().mode, MustangMode::GpuAccelerated);
    }

    #[test]
    fn test_effect_creation() {
        let blur = Effect::blur(".test", 10.0, 800, 600);
        assert!(blur.is_native());
        assert!(!blur.requires_gpu_compute());

        let color_adjust = Effect::color_adjust(".test", ColorAdjustParams::default());
        assert!(!color_adjust.is_native());
        assert!(color_adjust.requires_gpu_compute());
    }

    #[test]
    fn test_compositor_basic() {
        use arniko::{Compositor, CompositorConfig, Effect};

        let compositor = Compositor::new();
        let buffer = vec![0u8; 50 * 50 * 4];
        let effects = vec![];

        let result = compositor.composite(&buffer, 50, 50, &effects).unwrap();
        assert_eq!(result.width, 50);
        assert_eq!(result.height, 50);
        assert_eq!(result.effects_applied, 0);
    }

    #[test]
    fn test_compositor_config() {
        use arniko::{Compositor, CompositorConfig, QualityPreset};

        let config = CompositorConfig {
            max_blur_radius: 25.0,
            enable_blur: true,
            enable_transforms: false,
            quality: QualityPreset::Low,
        };

        let compositor = Compositor::with_config(config);
        assert_eq!(compositor.config().max_blur_radius, 25.0);
        assert!(compositor.config().enable_blur);
        assert!(!compositor.config().enable_transforms);
        assert_eq!(compositor.config().quality, QualityPreset::Low);
    }
}
