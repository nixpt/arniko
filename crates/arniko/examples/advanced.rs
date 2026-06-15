//! Advanced Arniko example with compositor and Mustang effects
//!
//! This example demonstrates the complete visual effects pipeline including
//! glass morphism, transforms, and GPU-accelerated rendering.

use arniko::config::ThemeConfig;
use arniko::mustang::{
    ColorAdjustParams, CompositeResult, Compositor, CompositorConfig, Effect, MustangCompositor,
    MustangConfig, Region, TransformParams,
};
use arniko::ArnikoApp;

fn main() {
    println!("🎨 Arniko Advanced Visual Effects Demo");
    println!("=====================================");

    // Demonstrate glass morphism effects
    demo_glass_morphism();

    // Demonstrate advanced compositor features
    demo_advanced_compositor();

    // Demonstrate Mustang GPU acceleration
    demo_mustang_gpu();

    // Demonstrate theme-aware effects
    demo_theme_effects();
}

fn demo_glass_morphism() {
    println!("\n🪟 Glass Morphism Effects");
    println!("-------------------------");

    let html = ArnikoApp::html()
        .html_content(
            r#"
            <div class="glass-container">
                <h1>Glass Morphism Panel</h1>
                <p>This panel uses backdrop-filter blur and transparency for a glass effect.</p>
                <div class="glass-button">Glass Button</div>
                <div class="glass-card">
                    <h3>Glass Card</h3>
                    <p>Card with glass morphism styling and blur effects.</p>
                </div>
            </div>
        "#,
        )
        .style(
            r#"
            body { 
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                padding: 40px;
                min-height: 100vh;
                display: flex;
                align-items: center;
                justify-content: center;
            }
            .glass-container {
                background: rgba(255, 255, 255, 0.1);
                backdrop-filter: blur(20px);
                border: 1px solid rgba(255, 255, 255, 0.2);
                border-radius: 20px;
                padding: 30px;
                max-width: 500px;
                box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
            }
            .glass-container h1 {
                color: rgba(255, 255, 255, 0.9);
                margin-bottom: 15px;
                font-size: 2em;
                font-weight: 600;
            }
            .glass-container p {
                color: rgba(255, 255, 255, 0.7);
                margin-bottom: 20px;
                line-height: 1.6;
            }
            .glass-button {
                background: rgba(255, 255, 255, 0.2);
                backdrop-filter: blur(10px);
                border: 1px solid rgba(255, 255, 255, 0.3);
                border-radius: 12px;
                padding: 12px 24px;
                color: rgba(255, 255, 255, 0.9);
                font-weight: 500;
                cursor: pointer;
                transition: all 0.3s ease;
            }
            .glass-button:hover {
                background: rgba(255, 255, 255, 0.3);
                transform: translateY(-2px);
            }
            .glass-card {
                background: rgba(255, 255, 255, 0.05);
                backdrop-filter: blur(15px);
                border: 1px solid rgba(255, 255, 255, 0.1);
                border-radius: 15px;
                padding: 20px;
                margin-top: 20px;
            }
            .glass-card h3 {
                color: rgba(255, 255, 255, 0.8);
                margin-bottom: 10px;
                font-size: 1.2em;
            }
            .glass-card p {
                color: rgba(255, 255, 255, 0.6);
                font-size: 0.9em;
                line-height: 1.5;
            }
        "#,
        )
        .render();

    println!("Generated HTML with glass morphism effects:");
    println!("{}", html);
}

fn demo_advanced_compositor() {
    println!("\n⚙️ Advanced Compositor Features");
    println!("----------------------------");

    // Create compositor with custom configuration
    let config = CompositorConfig {
        max_effects: 64,
        gpu_enabled: true,
    };

    let mut compositor = Compositor::with_config(config.clone());

    // Create test buffer (RGBA format)
    let width = 800;
    let height = 600;
    let buffer = create_test_buffer(width, height);

    // Create advanced effects
    let effects = vec![
        // Multi-pass blur effect
        Effect::blur(".glass-panel", 25.0, width, height),
        // Transform with rotation and scale
        Effect::transform(
            ".rotating-element",
            TransformParams {
                scale_x: 1.2,
                scale_y: 1.2,
                rotate_degrees: 45.0,
                translate_x: 100.0,
                translate_y: 50.0,
                pivot_x: 0.5,
                pivot_y: 0.5,
            },
            width,
            height,
        ),
        // Color adjustment for cyberpunk theme
        Effect::color_adjust(
            ".cyberpunk-element",
            ColorAdjustParams {
                red_multiplier: 1.5,
                green_multiplier: 0.5,
                blue_multiplier: 2.0,
                red_offset: 0.2,
                green_offset: -0.1,
                blue_offset: 0.3,
            },
        ),
        // Security clipping
        Effect::clip(Region::new(100.0, 100.0, 600.0, 400.0)),
    ];

    println!("Compositor Configuration:");
    println!("  - Max effects: {}", config.max_effects);
    println!("  - GPU enabled: {}", config.gpu_enabled);

    println!("\nEffects to apply:");
    for (i, effect) in effects.iter().enumerate() {
        println!(
            "  {}. {:?} - Native: {}",
            i + 1,
            effect.effect_type,
            effect.is_native()
        );
    }

    // Apply effects
    match compositor.composite(&buffer, width, height, &effects) {
        Ok(result) => {
            println!("\nCompositing Results:");
            println!("  - Buffer size: {} bytes", result.buffer.len());
        }
        Err(e) => {
            println!("\nCompositing failed: {}", e);
        }
    }
}

fn demo_mustang_gpu() {
    println!("\n🐎 Mustang GPU Compositor");
    println!("-----------------------");

    // Create Mustang compositor with GPU acceleration
    let config = MustangConfig::gpu_accelerated()
        .enable_caching(true)
        .max_cache_size(100)
        .enable_debug(true);

    let mut mustang = MustangCompositor::new(config);

    // Create Vello scene
    let mut scene = vello::Scene::new();

    // Add some basic content to the scene
    use vello::peniko::{Color, Fill};
    scene.fill(
        Fill::NonZero,
        vello::kurbo::Affine::IDENTITY,
        Color::from_rgba8(50, 50, 100, 255),
        None,
        &vello::kurbo::Rect::new(0.0, 0.0, 800.0, 600.0),
    );

    // Create GPU-accelerated effects
    let effects = vec![
        Effect::blur(".background", 30.0, 800, 600),
        Effect::transform(
            ".floating-element",
            TransformParams {
                scale_x: 1.1,
                scale_y: 1.1,
                rotate_degrees: 15.0,
                ..Default::default()
            },
            800,
            600,
        ),
    ];

    println!("Mustang Configuration:");
    println!("  - Mode: {:?}", mustang.config().mode);
    println!("  - Caching enabled: {}", mustang.config().enable_caching);
    println!("  - Max cache size: {}", mustang.config().max_cache_size);
    println!("  - Debug mode: {}", mustang.config().enable_debug);

    println!("\nApplying {} GPU effects to scene:", effects.len());

    // Apply effects to scene
    let result = mustang.apply_scene_effects(&mut scene, &effects, (800, 600));

    println!("\nScene Effect Results:");
    println!("  - Native effects applied: {}", result.native_applied);
    println!("  - Deferred effects: {}", result.deferred_count());

    // Process deferred effects
    if !result.deferred_effects.is_empty() {
        println!(
            "\nProcessing {} deferred effects with GPU compute...",
            result.deferred_count()
        );
        println!("  (Deferred effects would be handled by the GPU compute pipeline)");
    }

    // Get performance statistics
    let stats = mustang.get_stats();
    println!("\nPerformance Statistics:");
    println!("  - Cached components: {}", stats.cached_components);
    println!("  - Processing mode: {:?}", stats.mode);
}

fn demo_theme_effects() {
    println!("\n🎨 Theme-Aware Effects");
    println!("---------------------");

    // Create theme-aware compositor
    let mut theme_compositor = ThemeCompositor::new();

    let mut theme_glass = ThemeConfig::dark();
    theme_glass.name = "glass-morphism".to_string();
    let mut theme_cyber = ThemeConfig::dark();
    theme_cyber.name = "cyberpunk".to_string();
    let mut theme_aurora = ThemeConfig::light();
    theme_aurora.name = "aurora".to_string();
    let mut theme_minimal = ThemeConfig::auto();
    theme_minimal.name = "minimal".to_string();

    let themes = vec![
        ("glass-morphism", theme_glass),
        ("cyberpunk", theme_cyber),
        ("aurora", theme_aurora),
        ("minimal", theme_minimal),
    ];

    for (theme_name, theme_config) in themes {
        println!("\nTheme: {}", theme_name);

        // Create test buffer
        let buffer = create_test_buffer(800, 600);

        // Apply theme-aware effects
        match theme_compositor.composite_frame(&buffer, 800, 600, &theme_config) {
            Ok(result) => println!("  ✅ Theme effects applied successfully"),
            Err(e) => println!("  ❌ Theme effects failed: {}", e),
        }

        // Check if theme has effects
        let has_effects = arniko::compositor::integration::theme_has_effects(&theme_config);
        println!("  - Has effects: {}", has_effects);
    }
}

/// Create a test buffer with gradient pattern
fn create_test_buffer(width: u32, height: u32) -> Vec<u8> {
    let mut buffer = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            // Create a gradient pattern
            let r = (x as f32 / width as f32 * 255.0) as u8;
            let g = (y as f32 / height as f32 * 255.0) as u8;
            let b = 128;
            let a = 255;

            buffer.extend_from_slice(&[r, g, b, a]);
        }
    }

    buffer
}

pub struct ThemeCompositor {
    compositor: Compositor,
}

impl ThemeCompositor {
    pub fn new() -> Self {
        Self {
            compositor: Compositor::new(),
        }
    }

    pub fn composite_frame(
        &mut self,
        buffer: &[u8],
        width: u32,
        height: u32,
        theme: &ThemeConfig,
    ) -> anyhow::Result<CompositeResult> {
        let effects = if theme_has_effects(theme) {
            vec![Effect::blur(".theme-glow", 10.0, width, height)]
        } else {
            vec![]
        };
        self.compositor.composite(buffer, width, height, &effects)
    }
}

pub fn theme_has_effects(theme: &ThemeConfig) -> bool {
    theme.name == "glass-morphism" || theme.name == "cyberpunk" || theme.name == "aurora"
}

#[cfg(test)]
mod tests {
    use super::*;
    use arniko::{AlertVariant, BadgeVariant, ButtonVariant, Card};

    #[test]
    fn test_glass_morphism_html() {
        let html = ArnikoApp::html()
            .component(Button::new("Glass").variant(ButtonVariant::Ghost))
            .component(Card::new().title("Glass Card").class("glass"))
            .style("body { background: #1a1a1a; }")
            .render();

        assert!(html.contains("glass-button"));
        assert!(html.contains("glass-card"));
        assert!(html.contains("backdrop-filter"));
    }

    #[test]
    fn test_compositor_effects() {
        let mut compositor = Compositor::new();
        let buffer = vec![255u8; 100 * 100 * 4];
        let effects = vec![
            Effect::blur(".test", 10.0, 100, 100),
            Effect::transform(".test", TransformParams::default(), 100, 100),
        ];

        let result = compositor.composite(&buffer, 100, 100, &effects).unwrap();
        assert_eq!(result.buffer.len(), 100 * 100 * 4);
    }

    #[test]
    fn test_mustang_effects() {
        let mustang = MustangCompositor::default();
        let effects = vec![Effect::blur(".test", 5.0, 100, 100)];

        let mut scene = vello::Scene::new();
        let result = mustang.apply_scene_effects(&mut scene, &effects, (100, 100));

        assert_eq!(result.native_applied, 1);
        assert_eq!(result.deferred_count(), 0);
    }

    #[test]
    fn test_theme_integration() {
        let mut theme_config = ThemeConfig::dark();
        theme_config.name = "glass-morphism".to_string();
        let _theme_compositor = ThemeCompositor::new();

        assert!(theme_has_effects(&theme_config));

        let mut plain_theme = ThemeConfig::dark();
        plain_theme.name = "plain".to_string();
        assert!(!theme_has_effects(&plain_theme));
    }
}
