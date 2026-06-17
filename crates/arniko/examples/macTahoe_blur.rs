//! MacTahoe blur presets example
//!
//! This example demonstrates using MacTahoe-inspired blur presets
//! in Arniko's Mustang GPU compositor.
//!
//! Run with:
//!   CARGO_TARGET_DIR=/build/target-arniko cargo run -p arniko --features reactive,gpu --example macTahoe_blur

use arniko::mustang::{
    Effect, MustangCompositor, MacTahoeBlurPreset, all_presets, BlurPresetBuilder,
};
use arniko::reactive::{
    Div, ReactiveText, Signal, StaticHtml, Text, VelloScenePainter, View,
    launch_reactive_configured,
};

/// Demonstrates MacTahoe blur presets
fn main() {
    launch_reactive_configured(
        |mutator, reactor, router, root, _rt| {
            let count = Signal::new(0i32);

            let ui = Div::styled(
                "padding:80px; background:#09090b; color:#f4f4f5; \
                 font-family:ui-sans-serif,system-ui,sans-serif; \
                 min-height:100vh; box-sizing:border-box;",
                vec![
                    // Header
                    Box::new(Div::styled(
                        "text-align:center; margin-bottom:40px;",
                        vec![Box::new(Div::styled(
                            "font-size:24px; color:#a855f7; font-weight:700;",
                            vec![Box::new(Div::styled(
                                "font-size:14px; color:#71717a;",
                                vec![
                                    Box::new(Div::styled(
                                        "margin-top:10px; color:#a1a1aa;",
                                        vec![],
                                    )),
                                    Box::new(Div::styled(
                                        "margin-top:10px; color:#a1a1aa;",
                                        vec![],
                                    )),
                                ],
                            )),
                        )),
                    )),

                    // Glass panel with blur effect
                    Box::new(Div::styled(
                        "width:500px; background:rgba(255,255,255,0.08); \
                         border:1px solid rgba(255,255,255,0.15); border-radius:20px; \
                         padding:40px; display:flex; flex-direction:column; gap:20px;",
                        vec![
                            Box::new(StaticHtml(
                                r#"<h2 style="margin:0;font-size:22px;color:#a855f7;">
                                       MacTahoe Blur Demo
                                   </h2>
                                   <p style="margin:0;font-size:14px;color:#71717a;">
                                       This panel has a MacTahoe-style frosted glass effect
                                       applied via the Mustang GPU compositor.
                                   </p>"#,
                            )),
                            Box::new(Div::styled(
                                "display:flex; gap:12px; align-items:center;",
                                vec![
                                    Box::new(Div::styled(
                                        "cursor:pointer; padding:10px 20px; background:#a855f7; \
                                         border-radius:8px; font-size:16px; font-weight:600; \
                                         user-select:none;",
                                        vec![Box::new(Text("−".into()))],
                                    )),
                                    Box::new(Div::styled(
                                        "font-size:36px; font-weight:700; min-width:70px; \
                                         text-align:center;",
                                        vec![Box::new(ReactiveText::new(count.clone()))],
                                    )),
                                    Box::new(Div::styled(
                                        "cursor:pointer; padding:10px 20px; background:#a855f7; \
                                         border-radius:8px; font-size:16px; font-weight:600; \
                                         user-select:none;",
                                        vec![Box::new(Text("+".into()))],
                                    )),
                                ],
                            )),
                        ],
                    )),

                    // Preset selector
                    Box::new(Div::styled(
                        "margin-top:40px; background:#18181b; border-radius:12px; padding:20px;",
                        vec![
                            Box::new(Div::styled(
                                "font-size:16px; font-weight:600; color:#e4e4e7; margin-bottom:16px;",
                                vec![],
                            )),
                            Box::new(Div::styled(
                                "display:flex; flex-wrap:wrap; gap:8px;",
                                vec![
                                    // Button to show all presets
                                    Box::new(Div::styled(
                                        "cursor:pointer; padding:8px 12px; background:#374151; \
                                         border-radius:6px; font-size:13px; color:#e4e4e7; \
                                         user-select:none; transition:background 0.2s;",
                                        vec![Box::new(Div::styled(
                                            "cursor:pointer; padding:8px 12px; background:#3b82f6; \
                                             border-radius:6px; font-size:13px; color:#f4f4f5; \
                                             user-select:none;",
                                            vec![],
                                        )),
                                    ]),
                                ],
                            )),
                        ],
                    )),
                ],
            );

            let panel_id = ui.mount(mutator, reactor, root);
            let glass = mutator.child_ids(panel_id)[0];

            let dec = count.clone();
            router.on_click(vec![mutator.child_ids(glass)[0]], move || dec.update(|n| n - 1));
            let inc = count.clone();
            router.on_click(vec![mutator.child_ids(glass)[1]], move || inc.update(|n| n + 1));
        },
        // ── Renderer with MacTahoe blur presets ────────────────────────────────
        |renderer| {
            let mut compositor = MustangCompositor::default();

            // Get the default blur preset for this theme
            let default_preset = arniko::theme::ThemeMode::Frosted.default_blur_preset();
            println!("Default blur preset: {}", default_preset.name());

            // Get all available presets
            let presets = all_presets();
            println!("Available presets:");
            for preset in presets {
                println!("  - {}: {}", preset.name(), preset.description());
            }

            // Create a custom blur effect using the preset
            let blur_effect = Effect::blur("glass-panel", default_preset.radius(), 1280, 720)
                .with_region(
                    arniko::mustang::compositor::region::Region::new(80.0, 80.0, 500.0, 220.0),
                );

            renderer.set_scene_effects(move |scene, w, h| {
                let mut painter = VelloScenePainter::new(scene);
                compositor.apply_scene_effects(&mut painter, &[blur_effect.clone()], (w, h));
            });
        },
    );
}
