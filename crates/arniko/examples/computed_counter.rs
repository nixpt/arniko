//! Computed signals demo — shows Signal::derive, Computed::from2, and Computed::map.
//!
//! Three derived values update automatically whenever `count` or `step` change:
//!   • doubled  = count * 2            (derive from one signal)
//!   • sum      = count + step         (from2: two signals)
//!   • label    = "+" / "0" / "−"     (chained map on count)
//!
//! Run with:
//!   CARGO_TARGET_DIR=/build/target-arniko cargo run -p arniko --features reactive --example computed_counter

use arniko::reactive::{
    Computed, Div, ReactiveText, Signal, StaticHtml, Text, View, launch_reactive,
};

fn main() {
    launch_reactive(|mutator, reactor, router, root| {
        let count = Signal::new(0i32);
        let step = Signal::new(1i32);

        // ── Derived values ────────────────────────────────────────────────────
        let doubled: Computed<i32> = count.derive(|n| n * 2);
        let sum: Computed<i32> = Computed::from2(count.clone(), step.clone(), |c, s| c + s);
        let label: Computed<&'static str> = count.derive(|n| {
            if n > 0 {
                "positive"
            } else if n < 0 {
                "negative"
            } else {
                "zero"
            }
        });

        // ── Layout ────────────────────────────────────────────────────────────
        let ui = Div::styled(
            "padding:60px; background:#09090b; color:#f4f4f5; \
             font-family:ui-sans-serif,system-ui,sans-serif; \
             display:flex; flex-direction:column; align-items:center; \
             gap:24px; min-height:100vh; box-sizing:border-box;",
            vec![
                Box::new(StaticHtml(
                    r#"<h1 style="margin:0;color:#a855f7;font-size:26px;">
                           Computed Signals
                       </h1>"#
                        .into(),
                )),
                // Counter display + controls
                Box::new(Div::styled(
                    "display:flex; flex-direction:column; align-items:center; gap:16px; \
                     background:rgba(168,85,247,0.08); border:1px solid rgba(168,85,247,0.25); \
                     border-radius:16px; padding:28px 40px;",
                    vec![
                        Box::new(Div::styled(
                            "font-size:64px; font-weight:700; min-width:100px; text-align:center;",
                            vec![Box::new(ReactiveText::new(count.clone()))],
                        )),
                        Box::new(Div::styled(
                            "display:flex; gap:10px;",
                            vec![
                                Box::new(Div::styled(
                                    "cursor:pointer; padding:10px 24px; background:#ef4444; \
                                     border-radius:8px; font-size:18px; font-weight:700; \
                                     user-select:none;",
                                    vec![Box::new(Text("−".into()))],
                                )),
                                Box::new(Div::styled(
                                    "cursor:pointer; padding:10px 24px; background:#a855f7; \
                                     border-radius:8px; font-size:18px; font-weight:700; \
                                     user-select:none;",
                                    vec![Box::new(Text("+".into()))],
                                )),
                                Box::new(Div::styled(
                                    "cursor:pointer; padding:10px 24px; background:#3f3f46; \
                                     border-radius:8px; font-size:18px; font-weight:700; \
                                     user-select:none;",
                                    vec![Box::new(Text("0".into()))],
                                )),
                            ],
                        )),
                    ],
                )),
                // Derived values table
                Box::new(Div::styled(
                    "display:flex; flex-direction:column; gap:10px; \
                     background:#18181b; border:1px solid #27272a; \
                     border-radius:12px; padding:20px 28px; width:320px;",
                    vec![
                        row("doubled (count × 2)", Box::new(ReactiveText::new(doubled))),
                        row("sum (count + step)", Box::new(ReactiveText::new(sum))),
                        row("label", Box::new(ReactiveText::new(label))),
                    ],
                )),
                // Step control
                Box::new(Div::styled(
                    "display:flex; align-items:center; gap:12px; font-size:13px; color:#71717a;",
                    vec![
                        Box::new(Text("step:".into())),
                        Box::new(Div::styled(
                            "cursor:pointer; padding:4px 12px; background:#27272a; \
                             border-radius:6px; font-size:13px; user-select:none;",
                            vec![Box::new(Text("−".into()))],
                        )),
                        Box::new(ReactiveText::new(step.clone())),
                        Box::new(Div::styled(
                            "cursor:pointer; padding:4px 12px; background:#27272a; \
                             border-radius:6px; font-size:13px; user-select:none;",
                            vec![Box::new(Text("+".into()))],
                        )),
                    ],
                )),
            ],
        );

        let ui_id = ui.mount(mutator, reactor, root);

        // ── Wire click handlers ───────────────────────────────────────────────
        let counter_card = mutator.child_ids(ui_id)[1];
        let btns_row = mutator.child_ids(counter_card)[1];
        let btns = mutator.child_ids(btns_row);

        let step_row = mutator.child_ids(ui_id)[3];
        let step_btns = [
            mutator.child_ids(step_row)[1],
            mutator.child_ids(step_row)[3],
        ];

        let dec = count.clone();
        let s = step.clone();
        router.on_click(btns[0], move || {
            let s = s.get();
            dec.update(|n| n - s);
        });

        let inc = count.clone();
        let s = step.clone();
        router.on_click(btns[1], move || {
            let s = s.get();
            inc.update(|n| n + s);
        });

        let rst = count.clone();
        router.on_click(btns[2], move || rst.set(0));

        let sd = step.clone();
        router.on_click(step_btns[0], move || sd.update(|s| (s - 1).max(1)));

        let si = step.clone();
        router.on_click(step_btns[1], move || si.update(|s| s + 1));
    });
}

fn row(label: &str, value: Box<dyn View>) -> Box<dyn View> {
    Box::new(Div::styled(
        "display:flex; justify-content:space-between; align-items:center; \
         font-size:13px; border-bottom:1px solid #27272a; padding-bottom:8px;",
        vec![
            Box::new(Div::styled(
                "color:#71717a;",
                vec![Box::new(Text(label.to_string()))],
            )),
            Box::new(Div::styled(
                "font-weight:600; font-size:14px; color:#f4f4f5;",
                vec![value],
            )),
        ],
    ))
}
