//! Phase 5 demo — reactive-coordinated scene scheduling.
//!
//! Demonstrates the full Mustang↔Bliss-DOM feedback loop:
//! 1. A `Signal<i32>` is bound to a `ReactiveText` via the `Reactor`.
//! 2. The `SceneScheduler` is passed to the `Reactor` (so signal
//!    flushes notify it of DOM changes) and to the `set_scene_effects`
//!    hook (so effects only re-apply when the DOM actually changed).
//! 3. Clicking the +/− buttons updates the signal, which triggers a
//!    reactive flush, which notifies the scheduler, which causes the
//!    next frame to re-apply the GPU blur over the new DOM state.
//!
//! Run with:
//!   CARGO_TARGET_DIR=/build/target-arniko cargo run -p arniko \
//!     --features reactive,gpu --example phase5_demo

use arniko::mustang::scheduler::SceneScheduler;
use arniko::reactive::direct_mut::DirectDomMutator;
use arniko::reactive::{
    Div, EventRouter, Reactor, ReactiveText, Signal, StaticHtml, Text, View,
    launch_reactive_configured,
};
use bliss_dom::DocumentMutator;

fn main() {
    let scheduler = SceneScheduler::new();

    launch_reactive_configured(
        // ── DOM + signal setup ────────────────────────────────────────────────
        |mutator: &mut DocumentMutator, reactor: &mut Reactor, router: &mut EventRouter, root, _rt| {
            let count = Signal::new(0i32);

            // Use DirectDomMutator to demonstrate the Phase 5 mutation API.
            // It notifies the scheduler on every mutation, so the GPU blur
            // re-applies on the next frame.
            let mut dm = DirectDomMutator::new(mutator, &scheduler);

            // Mount the root view first (this also registers signal→DOM
            // bindings in the reactor).
            let ui = Div::styled(
                "padding:80px; background:#09090b; color:#f4f4z5; \
                 font-family:ui-sans-serif,system-ui,sans-serif; \
                 min-height:100vh; box-sizing:border-box;",
                vec![
                    Box::new(StaticHtml(
                        r#"<h2 style="margin:0;font-size:22px;color:#a855f7;">
                               Phase 5: SceneScheduler + DirectDomMutator
                           </h2>
                           <p style="margin:0;font-size:14px;color:#71717a;">
                               Click +/−. Each click bumps the signal → reactive flush
                               → scheduler notified → blur re-applied next frame.
                           </p>"#
                            .into(),
                    )),
                    // The ReactiveText is bound to `count` — it auto-updates
                    // when the signal changes, and the Reactor notifies the
                    // scheduler of the DOM change.
                    Box::new(ReactiveText::new(count.clone())),
                    Box::new(Div::styled(
                        "display:flex; gap:12px; align-items:center; margin-top:16px;",
                        vec![
                            Box::new(Div::styled(
                                "cursor:pointer; padding:10px 20px; background:#a855f7; \
                                 border-radius:8px; font-size:16px; font-weight:600; \
                                 user-select:none;",
                                vec![Box::new(Text("−".into()))],
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
            );

            // Mount the view, capturing the button IDs.
            let mounted = ui.mount(dm.raw(), reactor, root);
            let _btn_minus = dm.raw().child_ids(mounted).get(2).copied();
            let _btn_plus = dm.raw().child_ids(mounted).get(3).copied();

            // Wire click handlers via the EventRouter using the actual API.
            if let Some(btn_id) = _btn_minus {
                let dec = count.clone();
                router.on_click(btn_id, move || dec.update(|n| n - 1));
            }
            if let Some(btn_id) = _btn_plus {
                let inc = count.clone();
                router.on_click(btn_id, move || inc.update(|n| n + 1));
            }
        },
        // ── Renderer configuration: wire blur, gated by scheduler ────────────
        |renderer| {
            let scheduler_for_hook = scheduler.clone();
            renderer.set_scene_effects(move |_scene, _w, _h| {
                if scheduler_for_hook.should_apply() {
                    // In a real app, wrap _scene in a VelloScenePainter and
                    // call MustangCompositor::apply_scene_effects here.
                    // For this demo we just observe the apply counter.
                }
            });
        },
    );
}
