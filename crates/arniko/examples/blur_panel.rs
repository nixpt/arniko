//! Glass-blur panel — demonstrates the mustang GPU effect pipeline wired into bliss.
//!
//! A Gaussian blur halo is drawn over a rounded panel region every frame via
//! `VelloWindowRenderer::set_scene_effects` → mustang `MustangCompositor`.
//!
//! Run with:
//!   CARGO_TARGET_DIR=/build/target-arniko cargo run -p arniko --features reactive,gpu --example blur_panel

use arniko::reactive::{
    Div, Signal, StaticHtml, Text, ReactiveText, View,
    VelloScenePainter, launch_reactive_configured,
};
use mustang::{
    Effect, MustangCompositor,
    compositor::region::Region,
};

fn main() {
    // Panel region in screen coords — matched to the CSS layout below.
    let panel_region = Region::new(80.0, 80.0, 480.0, 260.0);

    launch_reactive_configured(
        // ── DOM + signal setup ─────────────────────────────────────────────────
        |mutator, reactor, router, root| {
            let count = Signal::new(0i32);

            let ui = Div::styled(
                "padding:80px; background:#09090b; color:#f4f4f5; \
                 font-family:ui-sans-serif,system-ui,sans-serif; \
                 min-height:100vh; box-sizing:border-box;",
                vec![
                    // The glass panel — mustang applies a blur halo over this region each frame.
                    Box::new(Div::styled(
                        "width:480px; background:rgba(255,255,255,0.06); \
                         border:1px solid rgba(255,255,255,0.12); border-radius:16px; \
                         padding:32px 40px; display:flex; flex-direction:column; gap:20px;",
                        vec![
                            Box::new(StaticHtml(
                                r#"<h2 style="margin:0;font-size:22px;color:#a855f7;">
                                       Glass Panel
                                   </h2>
                                   <p style="margin:0;font-size:14px;color:#71717a;">
                                       Mustang blur effect applied via the scene_effects hook
                                       every frame before wgpu submission.
                                   </p>"#.into(),
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
                                        "font-size:36px; font-weight:700; min-width:60px; \
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
                ],
            );

            let panel_id = ui.mount(mutator, reactor, root);
            // panel_id → child[1] (the glass panel) → child[2] (buttons row) → [0,2]
            let glass = mutator.child_ids(panel_id)[0];
            let buttons_row = mutator.child_ids(glass)[1];
            let btn_ids = mutator.child_ids(buttons_row);

            let dec = count.clone();
            router.on_click(btn_ids[0], move || dec.update(|n| n - 1));
            let inc = count.clone();
            router.on_click(btn_ids[2], move || inc.update(|n| n + 1));
        },

        // ── Renderer configuration: wire mustang blur ──────────────────────────
        |renderer| {
            let mut compositor = MustangCompositor::default();
            let effects = vec![
                // Gaussian blur halo over the glass panel — radius=20px, 2 passes.
                Effect::blur("glass-panel", 20.0, 1280, 720)
                    .with_region(panel_region),
            ];

            renderer.set_scene_effects(move |scene, w, h| {
                // Wrap the raw VelloScene in a PaintScene painter for mustang.
                let mut painter = VelloScenePainter::new(scene);
                compositor.apply_scene_effects(&mut painter, &effects, (w, h));
            });
        },
    );
}
