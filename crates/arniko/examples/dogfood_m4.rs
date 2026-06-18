//! Dogfood M4 — interactive markdown renderer exercising the full M3/M4 reactive
//! surface in one app:
//!
//! - `Signal<String>`            — reactive markdown source, mutated by click handlers
//! - `Computed<Vec<String>>`     — heading list extracted from source, via `.derive`
//! - `For<String, _>`            — outline sidebar over the computed heading list
//! - `Switch<ViewMode, _>`       — right pane picks between rendered / source / outline
//! - `ComponentView<Badge>`      — top-bar app badge via the `Component::to_view()`
//!                                 RAII adapter (consumes the badge, mounts it)
//!
//! Three buttons cycle the source between three sample documents; each click
//! bumps `content`, which fires the reactor, which patches the heading list,
//! which triggers `For`'s reconciliation, which re-renders the sidebar —
//! end-to-end through every reactive primitive.
//!
//! Run with:
//!   cargo run -p arniko --features reactive --example dogfood_m4

use arniko::components::Badge;
use arniko::reactive::{
    Div, For, ReactiveHtml, ReactiveText, Signal, Switch, Text, View, launch_reactive,
};
// Bring `Component` into scope so `Badge::new(...).to_view()` resolves.
use arniko::Component;

// ── Markdown source samples ──────────────────────────────────────────────────

const SAMPLE_WELCOME: &str = "# Welcome\n\n\
Arniko is a reactive UI framework. This sample demonstrates the dogfood demo.\n\n\
## Getting Started\n\n\
To launch the demo, click one of the source buttons below.\n\n\
### Prerequisites\n\n\
- Install Rust\n- Run with --features reactive\n\n\
### What you'll see\n\n\
Three panes: an outline (left), the raw source (here), and the preview (right).\n";

const SAMPLE_API: &str = "# API Reference\n\n\
Reactive primitives exposed in `arniko::reactive::*`.\n\n\
## Signals\n\n\
`Signal<T>` is share-by-clone reactive state. Mutate via `.set()` or `.update()`.\n\n\
## Computed\n\n\
`Computed<T>` derives a lazy value from one or more signals. Recomputes on read\nwhen any dep version has advanced.\n\n\
## Views\n\n\
`View::mount` returns a node ID and a `Scope`. The Scope tracks the bindings\nthe mount registered on the reactor.\n\n\
### Scope lifecycle\n\n\
`Scope::Drop` deregisters every binding it carries. The M4 `park_scope`\nmethod lets closures re-home a scope on the reactor so its bindings survive\npast the closure's return.\n";

const SAMPLE_CHANGELOG: &str = "# Changelog\n\n\
## v0.1.0 — M4\n\n\
- ComponentView::to_view() RAII adapter\n- Show/Switch conditional rendering\n- For positional diff reconciliation\n\n\
## v0.0.9 — M3\n\n\
- Scope-based binding lifecycle\n\n\
## v0.0.8 — M2\n\n\
- ReactiveText, ReactiveHtml\n\n\
## v0.0.1 — initial\n\n\
- HTML-only components\n";

/// Three-button list of (label, source) pairs cycled by the demo.
const SOURCES: &[(&str, &str)] = &[
    ("Welcome", SAMPLE_WELCOME),
    ("API Reference", SAMPLE_API),
    ("Changelog", SAMPLE_CHANGELOG),
];

// ── Markdown parsing (pure-Rust, no deps) ────────────────────────────────────

/// Extract headings (`#`, `##`, `###` lines) from a markdown source.
/// Used both for the outline sidebar and for the heading-count badge.
fn extract_headings(src: &str) -> Vec<String> {
    src.lines()
        .filter_map(|line| {
            if let Some(t) = line.strip_prefix("### ") {
                Some(t.to_string())
            } else if let Some(t) = line.strip_prefix("## ") {
                Some(t.to_string())
            } else if let Some(t) = line.strip_prefix("# ") {
                Some(t.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// HTML-escape a string for safe interpolation into markdown-rendered HTML.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Render the markdown source to a small HTML subset (headings + paragraphs).
/// Real markdown parsing is out of scope for this demo — it just exercises the
/// reactive plumbing, not a parser.
fn render_markdown(src: &str) -> String {
    let mut out = String::new();
    let mut in_para = false;
    for line in src.lines() {
        if line.is_empty() {
            if in_para {
                out.push_str("</p>");
                in_para = false;
            }
            continue;
        }
        if let Some(text) = line.strip_prefix("### ") {
            if in_para {
                out.push_str("</p>");
                in_para = false;
            }
            out.push_str(&format!("<h3>{}</h3>", escape_html(text)));
        } else if let Some(text) = line.strip_prefix("## ") {
            if in_para {
                out.push_str("</p>");
                in_para = false;
            }
            out.push_str(&format!("<h2>{}</h2>", escape_html(text)));
        } else if let Some(text) = line.strip_prefix("# ") {
            if in_para {
                out.push_str("</p>");
                in_para = false;
            }
            out.push_str(&format!("<h1>{}</h1>", escape_html(text)));
        } else if let Some(text) = line.strip_prefix("- ") {
            if in_para {
                out.push_str("</p>");
                in_para = false;
            }
            out.push_str(&format!("<li>{}</li>", escape_html(text)));
        } else {
            if !in_para {
                out.push_str("<p>");
                in_para = true;
            }
            out.push_str(&escape_html(line));
            out.push(' ');
        }
    }
    if in_para {
        out.push_str("</p>");
    }
    out
}

// ── ViewMode — Switch payload ────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
enum ViewMode {
    /// Rendered HTML preview
    Rendered,
    /// Raw markdown source
    Source,
    /// Just the heading outline (re-renders without the prose)
    Outline,
}

fn mode_label(m: &ViewMode) -> &'static str {
    match m {
        ViewMode::Rendered => "rendered",
        ViewMode::Source => "source",
        ViewMode::Outline => "outline",
    }
}

// ── Components ───────────────────────────────────────────────────────────────

/// Clickable button view. Encapsulates the "label + click handler" pattern
/// that the demo repeats six times (three source buttons, three mode buttons).
fn labeled_button(label: &str, accent: &str) -> Div {
    Div::styled(
        format!(
            "cursor:pointer; padding:8px 14px; margin-right:8px; \
             background:{accent}; color:var(--arniko-white); \
             border-radius:6px; font-size:13px; font-weight:600; \
             user-select:none; transition:opacity .15s ease;"
        ),
        vec![Box::new(Text(label.to_string()))],
    )
}

/// Body of the right pane's preview area. The body uses the cloned signal
/// so the Switch branch closure owns its handle to the reactive source.
fn preview_for_mode(
    mode: ViewMode,
    content: Signal<String>,
    headings: arniko::reactive::Computed<Vec<String>>,
) -> Box<dyn View> {
    match mode {
        ViewMode::Rendered => {
            // ReactiveText via .derive — preview updates as content changes.
            // `render_markdown` takes `&str`; close over `&s` to bridge Fn(String).
            let html: arniko::reactive::Computed<String> = content.derive(|s| render_markdown(&s));
            Box::new(Div::styled(
                "padding:16px 24px; background:var(--arniko-bg-elevated); \
                     border-radius:6px; border:1px solid var(--arniko-border); \
                     color:var(--arniko-text-body); font-size:14px; line-height:1.6;\
                     min-height:200px;",
                // ReactiveHtml wraps raw HTML; consumes a Reactive<String> source.
                vec![Box::new(ReactiveHtml::new(html))],
            ))
        }
        ViewMode::Source => Box::new(Div::styled(
            "padding:16px 24px; background:var(--arniko-bg-elevated); \
             border-radius:6px; border:1px solid var(--arniko-border); \
             font-family:ui-monospace,monospace; font-size:13px; \
             color:var(--arniko-text-body); white-space:pre-wrap; \
             min-height:200px; overflow-y:auto;",
            vec![Box::new(ReactiveText::new(content))],
        )),
        ViewMode::Outline => Box::new(Div::styled(
            "padding:16px 24px; background:var(--arniko-bg-elevated); \
             border-radius:6px; border:1px solid var(--arniko-border); \
             min-height:200px;",
            vec![Box::new(For::styled(
                "display:flex; flex-direction:column; gap:10px;",
                headings,
                |h| {
                    Box::new(Div::styled(
                        "padding:8px 12px; background:var(--arniko-bg-subtle); \
                         border-radius:4px; font-size:13px; \
                         color:var(--arniko-text-body);",
                        vec![Box::new(Text(h.clone()))],
                    ))
                },
            ))],
        )),
    }
}

// ── main ─────────────────────────────────────────────────────────────────────

fn main() {
    launch_reactive(|mutator, reactor, router, root, rt| {
        // ── Reactive state ─────────────────────────────────────────────────
        let content = Signal::new(SOURCES[0].1.to_string());
        // Wire to the event loop so click-driven .set() wakes the loop and triggers a flush.
        rt.wire(&content); // Computed<Vec<String>> derived from content. Lazy recompute on read.
        // `extract_headings` takes `&str`; close over `&s` to bridge Fn(String).
        let headings: arniko::reactive::Computed<Vec<String>> =
            content.derive(|s| extract_headings(&s));

        let content_for_previews = content.clone();
        let headings_for_previews = headings.clone();
        let mode: Signal<ViewMode> = Signal::new(ViewMode::Rendered);
        rt.wire(&mode);

        // ── Layout: top bar + 3-pane body + button bar ─────────────────────
        let shell = Div::styled(
            "padding:32px; background:var(--arniko-bg-surface); \
             color:var(--arniko-text-primary); \
             font-family:ui-sans-serif,system-ui,sans-serif; \
             display:flex; flex-direction:column; gap:20px; \
             min-height:100vh; box-sizing:border-box;",
            vec![
                // ── Top bar ────────────────────────────────────────────────
                Box::new(Div::styled(
                    "display:flex; align-items:center; gap:14px;",
                    vec![
                        // ComponentView<Badge> via .to_view() — consumes a Badge value
                        Box::new(
                            Badge::new("ARNIKO M4")
                                .variant(arniko::components::BadgeVariant::Purple)
                                .to_view(),
                        ),
                        Box::new(Text("Dogfood Demo".to_string())),
                        Box::new(Div::styled(
                            "margin-left:auto; font-size:12px; \
                             color:var(--arniko-text-muted);",
                            vec![Box::new(ReactiveText::new(headings.map(|v| {
                                format!(
                                    "{} heading{}",
                                    v.len(),
                                    if v.len() == 1 { "" } else { "s" }
                                )
                            })))],
                        )),
                    ],
                )),
                // ── 3-pane body ────────────────────────────────────────────
                Box::new(Div::styled(
                    "display:grid; grid-template-columns:240px 1fr; gap:16px;",
                    vec![
                        // Outline (For over Computed<Vec<String>>)
                        Box::new(Div::styled(
                            "padding:16px; background:var(--arniko-bg-elevated); \
                             border:1px solid var(--arniko-border); \
                             border-radius:8px; display:flex; \
                             flex-direction:column; gap:8px;",
                            vec![
                                Box::new(Div::styled(
                                    "font-size:11px; font-weight:600; \
                                     text-transform:uppercase; \
                                     letter-spacing:.08em; \
                                     color:var(--arniko-text-muted); \
                                     padding-bottom:8px; \
                                     border-bottom:1px solid \
                                     var(--arniko-border);",
                                    vec![Box::new(Text("Outline".to_string()))],
                                )),
                                Box::new(For::styled(
                                    "display:flex; flex-direction:column; \
                                     gap:6px;",
                                    headings.clone(),
                                    |h| {
                                        Box::new(Div::styled(
                                            "padding:6px 10px; \
                                             background:var(--arniko-bg-subtle); \
                                             border-radius:4px; \
                                             font-size:13px; \
                                             color:var(--arniko-text-body);",
                                            vec![Box::new(Text(h.clone()))],
                                        ))
                                    },
                                )),
                            ],
                        )),
                        // Right column wraps a Switch over ViewMode.
                        Box::new(Div::styled(
                            "padding:0; display:flex; flex-direction:column; \
                             gap:8px;",
                            vec![Box::new(Switch::new(mode.clone(), move |m: &ViewMode| {
                                preview_for_mode(
                                    m.clone(),
                                    content_for_previews.clone(),
                                    headings_for_previews.clone(),
                                )
                            }))],
                        )),
                    ],
                )),
                // ── Source-load button bar ──────────────────────────────────
                Box::new(Div::styled(
                    "padding:12px 16px; background:var(--arniko-bg-elevated); \
                     border:1px solid var(--arniko-border); \
                     border-radius:8px; display:flex; flex-wrap:wrap; \
                     align-items:center; gap:8px;",
                    vec![
                        Box::new(Div::styled(
                            "font-size:11px; font-weight:600; \
                             text-transform:uppercase; letter-spacing:.08em; \
                             color:var(--arniko-text-muted); margin-right:12px;",
                            vec![Box::new(Text("Load sample:".to_string()))],
                        )),
                        Box::new(labeled_button("Welcome", "#10b981")),
                        Box::new(labeled_button("API Reference", "#6366f1")),
                        Box::new(labeled_button("Changelog", "#f59e0b")),
                    ],
                )),
                // ── Mode-toggle button bar ──────────────────────────────────
                Box::new(Div::styled(
                    "padding:12px 16px; background:var(--arniko-bg-elevated); \
                     border:1px solid var(--arniko-border); \
                     border-radius:8px; display:flex; align-items:center; \
                     gap:8px;",
                    vec![
                        Box::new(Div::styled(
                            "font-size:11px; font-weight:600; \
                             text-transform:uppercase; letter-spacing:.08em; \
                             color:var(--arniko-text-muted); margin-right:12px;",
                            vec![Box::new(Text("Preview mode:".to_string()))],
                        )),
                        Box::new(labeled_button("Rendered", "var(--arniko-accent)")),
                        Box::new(labeled_button("Source", "var(--arniko-info)")),
                        Box::new(labeled_button("Outline", "var(--arniko-warning)")),
                        Box::new(Div::styled(
                            "margin-left:auto; font-size:12px; \
                             color:var(--arniko-text-muted);",
                            vec![Box::new(Div::styled(
                                "display:flex; gap:6px; align-items:center;",
                                vec![
                                    Box::new(Text("current: ".to_string())),
                                    Box::new(ReactiveText::new(
                                        // `mode: Signal<ViewMode>`; `Signal::derive` is `Fn(T) -> U`
                                        // (by value), and `mode_label` takes `&ViewMode` — pass `&m`.
                                        mode.derive(|m| mode_label(&m).to_string()),
                                    )),
                                ],
                            ))],
                        )),
                    ],
                )),
            ],
        );

        let (shell_id, _scope) = shell.mount(mutator, reactor, root);

        // ── Wire click handlers ────────────────────────────────────────────
        // Walk the mounted tree to discover the actual node IDs:
        //   shell
        //   ├─ [0] top_bar
        //   ├─ [1] body (3-pane)
        //   │   └─ [0] outline_pane
        //   │       └─ [0] "Outline" header, [1] For container
        //   │   └─ [1] right_column
        //   ├─ [2] source_button_bar
        //   │   ├─ [0] "Load sample:" header
        //   │   ├─ [1] Welcome btn
        //   │   ├─ [2] API Reference btn
        //   │   ├─ [3] Changelog btn
        //   ├─ [3] mode_button_bar
        //       ├─ [0] "Preview mode:" header
        //       ├─ [1] Rendered btn
        //       ├─ [2] Source btn
        //       ├─ [3] Outline btn
        //       ├─ [4] current-mode label
        //
        // Skip the walkers we don't need and just inspect child IDs at each level.

        let shell_children = mutator.child_ids(shell_id);
        let source_bar = shell_children[2];
        let mode_bar = shell_children[3];

        let source_btns = mutator.child_ids(source_bar);
        // [0] = "Load sample:" label, [1..] = the three sample buttons
        for (i, (_, src)) in SOURCES.iter().enumerate() {
            let btn_id = source_btns[1 + i];
            let new_src = src.to_string();
            let c = content.clone();
            router.on_click(btn_id, move || {
                c.set(new_src.clone());
            });
        }

        let mode_btns = mutator.child_ids(mode_bar);
        let mode_targets = [
            (mode_btns[1], ViewMode::Rendered),
            (mode_btns[2], ViewMode::Source),
            (mode_btns[3], ViewMode::Outline),
        ];
        for (btn_id, target) in mode_targets {
            let m = mode.clone();
            router.on_click(btn_id, move || {
                m.set(target.clone());
            });
        }
    });
}
