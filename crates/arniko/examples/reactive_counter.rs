//! Reactive counter — demonstrates Signal + View + EventRouter end-to-end.
//!
//! Run with:
//!   CARGO_TARGET_DIR=/build/target-arniko cargo run -p arniko --features reactive --example reactive_counter

use arniko::reactive::{Div, ReactiveText, Signal, StaticHtml, Text, View, launch_reactive};

fn main() {
    launch_reactive(|mutator, reactor, router, root, _rt| {
        let count = Signal::new(0i32);

        // ── Layout ─────────────────────────────────────────────────────────────
        let shell = Div::styled(
            "padding: 60px; background: #050508; color: #f4f4f5; \
             font-family: ui-sans-serif, system-ui, sans-serif; \
             display: flex; flex-direction: column; align-items: center; gap: 24px; \
             min-height: 100vh; box-sizing: border-box;",
            vec![
                // Title
                Box::new(StaticHtml(
                    r#"<h1 style="margin:0; color:#a855f7; font-size:28px;">Arniko Counter</h1>"#
                        .into(),
                )),
                // Counter value — reactive
                Box::new(Div::styled(
                    "font-size: 72px; font-weight: 700; min-width: 120px; text-align: center; \
                     background: rgba(168,85,247,0.1); border: 1px solid rgba(168,85,247,0.3); \
                     border-radius: 16px; padding: 16px 32px;",
                    vec![Box::new(ReactiveText::new(count.clone()))],
                )),
                // Buttons row
                Box::new(Div::styled(
                    "display: flex; gap: 12px;",
                    vec![
                        Box::new(Div::styled(
                            "cursor: pointer; padding: 12px 28px; background: #ef4444; \
                             border-radius: 10px; font-size: 20px; font-weight: 600; \
                             user-select: none;",
                            vec![Box::new(Text("−".into()))],
                        )),
                        Box::new(Div::styled(
                            "cursor: pointer; padding: 12px 28px; background: #a855f7; \
                             border-radius: 10px; font-size: 20px; font-weight: 600; \
                             user-select: none;",
                            vec![Box::new(Text("+".into()))],
                        )),
                        Box::new(Div::styled(
                            "cursor: pointer; padding: 12px 28px; background: #3f3f46; \
                             border-radius: 10px; font-size: 20px; font-weight: 600; \
                             user-select: none;",
                            vec![Box::new(Text("0".into()))],
                        )),
                    ],
                )),
                // Status line — reactive
                Box::new(Div::styled(
                    "font-size: 13px; color: #71717a;",
                    vec![
                        Box::new(Text("value is ".into())),
                        Box::new(ReactiveText::new(count.clone())),
                    ],
                )),
            ],
        );

        let (shell_id, _scope) = shell.mount(mutator, reactor, root);

        // ── Wire click handlers ────────────────────────────────────────────────
        // Node IDs for the three buttons come from walking the mounted subtree.
        // shell → children[2] (buttons row) → children[0,1,2]
        let buttons_row = mutator.child_ids(shell_id)[2];
        let btn_ids = mutator.child_ids(buttons_row);

        let dec = count.clone();
        router.on_click(btn_ids[0], move || dec.update(|n| n - 1));

        let inc = count.clone();
        router.on_click(btn_ids[1], move || inc.update(|n| n + 1));

        let rst = count.clone();
        router.on_click(btn_ids[2], move || rst.set(0));
    });
}
