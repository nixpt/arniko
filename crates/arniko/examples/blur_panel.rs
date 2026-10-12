//! Glass-blur panel — demonstrates the mustang GPU effect pipeline wired into bliss.
//!
//! A Gaussian blur halo is drawn over a rounded panel region every frame via
//! `VelloWindowRenderer::set_scene_effects` → mustang `MustangCompositor`.
//!
//! Uses `launch_reactive_scheduled` (mustang 0.4 scheduling): the effect
//! hook consumes the per-frame dirty state via `SceneScheduler::take_dirty()`.
//! The vello scene is rebuilt fresh every frame, so scene-native effects are
//! re-encoded every frame — what the scheduler decides is whether anything
//! CHANGED (full vs region-tracked vs clean), surfaced here as diagnostics
//! and ready for region-scoped application + GPU-compute gating downstream.
//!
//! Run with:
//!   CARGO_TARGET_DIR=/build/target-arniko cargo run -p arniko --features reactive,gpu --example blur_panel

use arniko::mustang::compositor::region::Region;
use arniko::mustang::{Effect, MustangCompositor, SceneDirtyState};
use arniko::reactive::{
    Div, ReactiveText, Signal, StaticHtml, Text, VelloScenePainter, View,
    launch_reactive_scheduled,
};

/// Apply the effect list to a fresh scene, scoped by the dirty state.
fn apply_effects(
    compositor: &mut MustangCompositor,
    painter: &mut VelloScenePainter<'_, '_>,
    effects: &[Effect],
    dirty: &SceneDirtyState,
    w: u32,
    h: u32,
) {
    let scoped: Vec<Effect> = if dirty.full {
        effects.to_vec()
    } else {
        effects
            .iter()
            .filter(|e| {
                dirty
                    .regions
                    .iter()
                    .any(|r| r.intersects(&e.region))
            })
            .cloned()
            .collect()
    };
    let _ = compositor.apply_scene_effects(painter, &scoped, (w, h));
}

fn main() {
    // Panel region in screen coords — matched to the CSS layout below.
    let panel_region = Region::new(80.0, 80.0, 480.0, 260.0);

    launch_reactive_scheduled(
        // ── DOM + signal setup ─────────────────────────────────────────────────
        |mutator, reactor, router, root, _rt, _scheduler| {
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
                                   </p>"#
                                    .into(),
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

            let (panel_id, _scope) = ui.mount(mutator, reactor, root);
            // panel_id → child[1] (the glass panel) → child[2] (buttons row) → [0,2]
            let glass = mutator.child_ids(panel_id)[0];
            let buttons_row = mutator.child_ids(glass)[1];
            let btn_ids = mutator.child_ids(buttons_row);

            let dec = count.clone();
            router.on_click(btn_ids[0], move || dec.update(|n| n - 1));
            let inc = count.clone();
            router.on_click(btn_ids[2], move || inc.update(|n| n + 1));
        },
        // ── Renderer configuration: wire mustang blur + the scheduler ──────────
        |renderer, scheduler| {
            let mut compositor = MustangCompositor::default();
            let effects = vec![
                // Gaussian blur halo over the glass panel — radius=20px, 2 passes.
                Effect::blur("glass-panel", 20.0, 1280, 720).with_region(panel_region),
            ];
            let sched = scheduler.clone();

            renderer.set_scene_effects(move |scene, w, h| {
                // Consume the frame's dirty state: reactive flushes + direct
                // DOM mutations coalesce into one cycle per frame.
                let dirty = sched.take_dirty();
                if !dirty.is_dirty() {
                    eprintln!("[blur-panel] frame: clean (effects re-encoded, no DOM change)");
                } else if dirty.full {
                    eprintln!("[blur-panel] frame: FULL dirty — re-check every effect");
                } else {
                    eprintln!(
                        "[blur-panel] frame: {} region(s) dirty — scoped re-check",
                        dirty.regions.len()
                    );
                }
                // The scene is fresh each frame — effects are re-encoded
                // unconditionally, scoped by the dirty regions.
                let mut painter = VelloScenePainter::new(scene);
                apply_effects(&mut compositor, &mut painter, &effects, &dirty, w, h);
            });
        },
    );
}
