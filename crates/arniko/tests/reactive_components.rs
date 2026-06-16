//! Integration tests for reactive arniko components.
//!
//! Each test sets up a BaseDocument via DocumentMutator, mounts a reactive View,
//! flushes the Reactor, and verifies DOM state using only the public API.
//!
//! Run with: `cargo test -p arniko --features "reactive,launch,components,html" --test reactive_components`

use arniko::components::{
    BarEntry, ShortcutEntry, SplashConfig, ThemeState, ToastVariant, bar_chart_reactive,
    mount_toast, mount_toast_with_variant, progress_ring_reactive, shortcut_help_reactive,
    splash_screen_reactive, theme_toggle_reactive,
};
use arniko::reactive::{Reactor, Signal, View};
use bliss_dom::{BaseDocument, DocumentConfig, DocumentMutator, qual_name};
use bliss_html::HtmlProvider;
use std::sync::Arc;

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Create a fresh document with a root element, return (doc, root_id).
fn setup_doc() -> (BaseDocument, usize) {
    let config = DocumentConfig {
        html_parser_provider: Some(Arc::new(HtmlProvider)),
        ..DocumentConfig::default()
    };
    let mut doc = BaseDocument::new(config);
    let mut mutator = doc.mutate();
    let root_id = mutator.create_element(qual_name!("div"), vec![]);
    // Append to the document root (node 0)
    mutator.append_children(0, &[root_id]);
    drop(mutator);
    (doc, root_id)
}

/// Flush the reactor and drop the mutator, returning doc for assertions.
fn flush_reactive(doc: &mut BaseDocument, reactor: &mut Reactor) {
    let mut mutator = doc.mutate();
    reactor.flush(&mut mutator);
    drop(mutator);
}

/// Get the text content of a node and its subtree using only public APIs.
fn node_text(doc: &mut BaseDocument, root_id: usize) -> String {
    let mut text = String::new();
    let mut mutator = doc.mutate();
    collect_text_from_mutator(&mut mutator, root_id, &mut text);
    drop(mutator);
    text
}

fn collect_text_from_mutator(mutator: &mut DocumentMutator, node_id: usize, buf: &mut String) {
    // Check if this node is a text node
    if let Some(node) = mutator.doc.get_node(node_id) {
        if let Some(td) = node.text_data() {
            buf.push_str(&td.content);
        }
    }
    // Recurse into children
    let children = mutator.child_ids(node_id);
    for child_id in children {
        collect_text_from_mutator(mutator, child_id, buf);
    }
}

/// Find an element by CSS class in the subtree, using public APIs.
fn find_by_class(doc: &mut BaseDocument, root_id: usize, class: &str) -> Option<usize> {
    if let Some(node) = doc.get_node(root_id) {
        if let Some(el) = node.element_data() {
            if let Some(c) = el.attr(bliss_dom::local_name!("class")) {
                if c.split_whitespace().any(|part| part == class) {
                    return Some(root_id);
                }
            }
        }
    }
    let children = {
        let mut mutator = doc.mutate();
        mutator.child_ids(root_id)
    };
    for child_id in children {
        if let Some(id) = find_by_class(doc, child_id, class) {
            return Some(id);
        }
    }
    None
}

// ── Toast Tests ──────────────────────────────────────────────────────────────

#[test]
fn test_mount_toast_visible() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let msg: Signal<Option<String>> = Signal::new(Some("Hello Toast".to_string()));

    {
        let mut mutator = doc.mutate();
        mount_toast(&mut mutator, &mut reactor, root_id, &msg);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(
        text.contains("Hello Toast"),
        "Toast text should appear: {}",
        text
    );
}

#[test]
fn test_mount_toast_hidden_when_none() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let msg: Signal<Option<String>> = Signal::new(None);

    {
        let mut mutator = doc.mutate();
        mount_toast(&mut mutator, &mut reactor, root_id, &msg);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);

    // When None, the toast text should be empty (no message shown)
    let text = node_text(&mut doc, root_id);
    assert!(
        !text.contains("Hello") && !text.contains("Toast"),
        "Toast should have no text when None, got: {}",
        text
    );
}

#[test]
fn test_mount_toast_variants_render_text() {
    // Verify that each variant mounts without panic and renders text
    let variants = [
        ToastVariant::Info,
        ToastVariant::Success,
        ToastVariant::Warning,
        ToastVariant::Error,
    ];

    for variant in variants {
        let (mut doc, root_id) = setup_doc();
        let mut reactor = Reactor::new();
        let msg: Signal<Option<String>> = Signal::new(Some("test".to_string()));

        {
            let mut mutator = doc.mutate();
            mount_toast_with_variant(&mut mutator, &mut reactor, root_id, &msg, variant);
            drop(mutator);
        }
        flush_reactive(&mut doc, &mut reactor);

        // Verify the toast text renders for each variant
        let text = node_text(&mut doc, root_id);
        assert!(
            text.contains("test"),
            "Variant {:?}: toast should show text",
            variant
        );
    }
}

#[test]
fn test_mount_toast_transition_on_update() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let msg: Signal<Option<String>> = Signal::new(None);

    {
        let mut mutator = doc.mutate();
        mount_toast(&mut mutator, &mut reactor, root_id, &msg);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);

    // Set message to Some → text should update
    msg.set(Some("Updated!".to_string()));
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(
        text.contains("Updated!"),
        "Toast text should update after set: {}",
        text
    );
}

// ── ProgressRing Tests ───────────────────────────────────────────────────────

#[test]
fn test_progress_ring_reactive_mounts() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let score = Signal::new(75.0_f64);

    let view = progress_ring_reactive(
        score.clone(),
        arniko::components::DEFAULT_RING_THRESHOLDS.to_vec(),
    );
    {
        let mut mutator = doc.mutate();
        view.mount(&mut mutator, &mut reactor, root_id);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(text.contains("75%"), "Ring should show 75%, got: {}", text);
    assert!(
        text.contains("WARNING"),
        "Ring should show WARNING label, got: {}",
        text
    );
}

#[test]
fn test_progress_ring_reactive_updates() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let score = Signal::new(30.0_f64);

    let view = progress_ring_reactive(
        score.clone(),
        arniko::components::DEFAULT_RING_THRESHOLDS.to_vec(),
    );
    {
        let mut mutator = doc.mutate();
        view.mount(&mut mutator, &mut reactor, root_id);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);
    assert!(node_text(&mut doc, root_id).contains("CRITICAL"));

    // Update to 90
    score.set(90.0);
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(
        text.contains("90%"),
        "Updated ring should show 90%, got: {}",
        text
    );
    assert!(
        text.contains("SECURE"),
        "Updated ring should show SECURE, got: {}",
        text
    );
}

// ── BarChart Tests ───────────────────────────────────────────────────────────

#[test]
fn test_bar_chart_reactive_mounts() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let entries = Signal::new(vec![
        BarEntry::new("CRITICAL", 5.0, "#ef4444"),
        BarEntry::new("HIGH", 3.0, "#f59e0b"),
    ]);

    let view = bar_chart_reactive(entries);
    {
        let mut mutator = doc.mutate();
        view.mount(&mut mutator, &mut reactor, root_id);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(
        text.contains("CRITICAL"),
        "Chart should show CRITICAL label: {}",
        text
    );
    assert!(
        text.contains("HIGH"),
        "Chart should show HIGH label: {}",
        text
    );
    assert!(
        text.contains("5.0"),
        "Chart should show value 5.0: {}",
        text
    );
    assert!(
        text.contains("3.0"),
        "Chart should show value 3.0: {}",
        text
    );
}

#[test]
fn test_bar_chart_reactive_empty() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let entries: Signal<Vec<BarEntry>> = Signal::new(vec![]);

    let view = bar_chart_reactive(entries);
    {
        let mut mutator = doc.mutate();
        view.mount(&mut mutator, &mut reactor, root_id);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(
        text.contains("No data"),
        "Empty chart should show 'No data': {}",
        text
    );
}

#[test]
fn test_bar_chart_reactive_updates() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let entries: Signal<Vec<BarEntry>> = Signal::new(vec![BarEntry::new("A", 1.0, "#ef4444")]);

    let view = bar_chart_reactive(entries.clone());
    {
        let mut mutator = doc.mutate();
        view.mount(&mut mutator, &mut reactor, root_id);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);
    assert!(node_text(&mut doc, root_id).contains("A"));

    // Add a new entry
    entries.set(vec![
        BarEntry::new("A", 1.0, "#ef4444"),
        BarEntry::new("B", 2.0, "#10b981"),
    ]);
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(
        text.contains("B"),
        "Updated chart should contain B: {}",
        text
    );
    assert!(
        text.contains("2.0"),
        "Updated chart should show value 2.0: {}",
        text
    );
}

// ── KeyboardShortcuts Tests ──────────────────────────────────────────────────

#[test]
fn test_shortcut_help_reactive_visible() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let visible = Signal::new(true);

    let shortcuts = vec![
        ShortcutEntry::new("F5", "Refresh"),
        ShortcutEntry::new("Esc", "Cancel"),
    ];

    let view = shortcut_help_reactive(&visible, shortcuts);
    {
        let mut mutator = doc.mutate();
        view.mount(&mut mutator, &mut reactor, root_id);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(text.contains("F5"), "Modal should show F5: {}", text);
    assert!(
        text.contains("Refresh"),
        "Modal should show Refresh: {}",
        text
    );
    assert!(text.contains("Esc"), "Modal should show Esc: {}", text);
    assert!(
        text.contains("Keyboard Shortcuts"),
        "Modal should show title: {}",
        text
    );
}

#[test]
fn test_shortcut_help_reactive_hidden() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let visible = Signal::new(false);

    let shortcuts = vec![ShortcutEntry::new("F5", "Refresh")];
    let view = shortcut_help_reactive(&visible, shortcuts);
    let view_id = {
        let mut mutator = doc.mutate();
        let id = view.mount(&mut mutator, &mut reactor, root_id);
        drop(mutator);
        id
    };
    flush_reactive(&mut doc, &mut reactor);

    // When hidden, the ReactiveHtml wrapper should have no text content
    let text = node_text(&mut doc, view_id);
    assert!(
        text.is_empty(),
        "Hidden modal should have no text content, got: {}",
        text
    );
}

#[test]
fn test_shortcut_help_reactive_toggle() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let visible = Signal::new(false);

    let shortcuts = vec![ShortcutEntry::new("F5", "Refresh")];
    let view = shortcut_help_reactive(&visible, shortcuts);
    {
        let mut mutator = doc.mutate();
        view.mount(&mut mutator, &mut reactor, root_id);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);

    // Now show it
    visible.set(true);
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(
        text.contains("F5"),
        "Toggled-on modal should show shortcuts: {}",
        text
    );
}

// ── ThemeToggle Tests ────────────────────────────────────────────────────────

#[test]
fn test_theme_toggle_reactive_mounts() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let theme = Signal::new(ThemeState {
        current: "🌙 Dark".to_string(),
        next: "☀️ Light".to_string(),
    });

    let view = theme_toggle_reactive(&theme);
    {
        let mut mutator = doc.mutate();
        view.mount(&mut mutator, &mut reactor, root_id);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(text.contains("Dark"), "Toggle should show Dark: {}", text);
    assert!(text.contains("Light"), "Toggle should show Light: {}", text);

    // Verify the button with correct class exists in the rendered DOM
    let button_id = find_by_class(&mut doc, root_id, "arniko-theme-toggle")
        .expect("Should find arniko-theme-toggle button");
    assert!(
        doc.get_node(button_id).is_some(),
        "Button node should exist"
    );
}

// ── SplashScreen Tests ───────────────────────────────────────────────────────

#[test]
fn test_splash_screen_reactive_mounts() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let config = Signal::new(SplashConfig {
        title: "My App".to_string(),
        subtitle: "v1.0".to_string(),
        progress: 50,
        status: "Loading modules...".to_string(),
        logo_svg: None,
    });

    let view = splash_screen_reactive(&config);
    {
        let mut mutator = doc.mutate();
        view.mount(&mut mutator, &mut reactor, root_id);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(
        text.contains("My App"),
        "Splash should show title: {}",
        text
    );
    assert!(
        text.contains("v1.0"),
        "Splash should show subtitle: {}",
        text
    );
    assert!(
        text.contains("50%"),
        "Splash should show progress: {}",
        text
    );
    assert!(
        text.contains("Loading modules..."),
        "Splash should show status: {}",
        text
    );
}

#[test]
fn test_splash_screen_reactive_updates() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let config = Signal::new(SplashConfig {
        title: "Loading".to_string(),
        subtitle: String::new(),
        progress: 0,
        status: "Starting...".to_string(),
        logo_svg: None,
    });

    let view = splash_screen_reactive(&config);
    {
        let mut mutator = doc.mutate();
        view.mount(&mut mutator, &mut reactor, root_id);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);
    assert!(node_text(&mut doc, root_id).contains("0%"));

    // Update progress
    config.set(SplashConfig {
        title: "Loading".to_string(),
        subtitle: String::new(),
        progress: 100,
        status: "Ready!".to_string(),
        logo_svg: None,
    });
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(
        text.contains("100%"),
        "Updated splash should show 100%: {}",
        text
    );
    assert!(
        text.contains("Ready!"),
        "Updated splash should show Ready!: {}",
        text
    );
}
