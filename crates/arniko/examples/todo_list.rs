//! Todo list — demonstrates `For<T, R>` reactive list rendering.
//!
//! A `Signal<Vec<String>>` drives the list. Add/remove items; the DOM reconciles automatically.
//!
//! Run with:
//!   CARGO_TARGET_DIR=/build/target-arniko cargo run -p arniko --features reactive --example todo_list

use arniko::reactive::{Div, For, Signal, StaticHtml, Text, View, launch_reactive};
use std::sync::{Arc, Mutex};

fn main() {
    launch_reactive(|mutator, reactor, router, root, _rt| {
        let items: Signal<Vec<String>> = Signal::new(vec![
            "Learn arniko".into(),
            "Build a UI".into(),
            "Ship it".into(),
        ]);

        let shell = Div::styled(
            "padding:60px; background:#09090b; color:#f4f4f5; \
             font-family:ui-sans-serif,system-ui,sans-serif; \
             display:flex; flex-direction:column; align-items:center; \
             gap:24px; min-height:100vh; box-sizing:border-box;",
            vec![
                Box::new(StaticHtml(
                    r#"<h1 style="margin:0;color:#a855f7;font-size:26px;">Todo List</h1>"#.into(),
                )),
                // Add / clear buttons
                Box::new(Div::styled(
                    "display:flex; gap:10px;",
                    vec![
                        Box::new(Div::styled(
                            "cursor:pointer; padding:10px 22px; background:#a855f7; \
                             border-radius:8px; font-size:14px; font-weight:600; \
                             user-select:none;",
                            vec![Box::new(Text("+ Add item".into()))],
                        )),
                        Box::new(Div::styled(
                            "cursor:pointer; padding:10px 22px; background:#ef4444; \
                             border-radius:8px; font-size:14px; font-weight:600; \
                             user-select:none;",
                            vec![Box::new(Text("✕ Remove last".into()))],
                        )),
                        Box::new(Div::styled(
                            "cursor:pointer; padding:10px 22px; background:#3f3f46; \
                             border-radius:8px; font-size:14px; font-weight:600; \
                             user-select:none;",
                            vec![Box::new(Text("Clear all".into()))],
                        )),
                    ],
                )),
                // The reactive list
                Box::new(For::styled(
                    "display:flex; flex-direction:column; gap:8px; \
                     width:400px; min-height:40px;",
                    items.clone(),
                    |item| todo_row(item),
                )),
                // Item count (computed inline via a separate signal binding)
                Box::new(Div::styled(
                    "font-size:12px; color:#52525b;",
                    vec![Box::new(arniko::reactive::ReactiveText::new(items.derive(
                        |v| format!("{} item{}", v.len(), if v.len() == 1 { "" } else { "s" }),
                    )))],
                )),
            ],
        );

        let (shell_id, _scope) = shell.mount(mutator, reactor, root);

        // Wire buttons: shell→[title, buttons_row, list, count]
        let btns_row = mutator.child_ids(shell_id)[1];
        let btns = mutator.child_ids(btns_row);

        let counter = Arc::new(Mutex::new(0usize));

        let add = items.clone();
        let ctr = Arc::clone(&counter);
        router.on_click(btns[0], move || {
            let mut n = ctr.lock().unwrap();
            *n += 1;
            let label = format!("New item {}", *n);
            add.update(|v| {
                let mut v = v.clone();
                v.push(label.clone());
                v
            });
        });

        let rem = items.clone();
        router.on_click(btns[1], move || {
            rem.update(|v| {
                let mut v = v.clone();
                v.pop();
                v
            });
        });

        let clr = items.clone();
        router.on_click(btns[2], move || clr.set(vec![]));
    });
}

fn todo_row(item: &String) -> Box<dyn View> {
    Box::new(Div::styled(
        "display:flex; align-items:center; gap:12px; \
         background:#18181b; border:1px solid #27272a; border-radius:10px; \
         padding:12px 16px; font-size:14px;",
        vec![
            Box::new(Div::styled(
                "width:8px; height:8px; border-radius:50%; background:#a855f7; flex-shrink:0;",
                vec![],
            )),
            Box::new(Text(item.clone())),
        ],
    ))
}
