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
use arniko::reactive::{For, Reactor, Signal, Text, View};
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
    reactor.flush(&mut mutator, None);
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
        let mutator = doc.mutate();
        mutator.child_ids(root_id)
    };
    for child_id in children {
        if let Some(id) = find_by_class(doc, child_id, class) {
            return Some(id);
        }
    }
    None
}

/// Mount a view and park the returned `Scope` on the reactor so its
/// reactive bindings survive past the call site.
///
/// This handles the very common test idiom of `view.mount(...)` invoked
/// as a statement, where the returned `Scope` would otherwise be silently
/// dropped at the semicolon and (via `Scope::drop`) deregister every
/// binding it carries. With the parking step the bindings stay live so
/// later `signal.set(...)` + `flush_reactive(...)` can fire patches.
///
/// Production code should generally capture the returned `Scope` and
/// drop it explicitly (or call `scope.unmount()`) — see
/// `test_binding_lifecycle_scope_cleanup` for the explicit-cleanup
/// pattern this helper side-steps.
fn mount_parked<V: View + ?Sized>(
    view: &V,
    mutator: &mut DocumentMutator,
    reactor: &mut Reactor,
    parent: usize,
) -> usize {
    let (id, scope) = view.mount(mutator, reactor, parent);
    reactor.park_scope(scope);
    id
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
        mount_parked(&view, &mut mutator, &mut reactor, root_id);
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
        mount_parked(&view, &mut mutator, &mut reactor, root_id);
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
        mount_parked(&view, &mut mutator, &mut reactor, root_id);
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
        mount_parked(&view, &mut mutator, &mut reactor, root_id);
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
        mount_parked(&view, &mut mutator, &mut reactor, root_id);
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
        mount_parked(&view, &mut mutator, &mut reactor, root_id);
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
        let (id, _scope) = view.mount(&mut mutator, &mut reactor, root_id);
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
        mount_parked(&view, &mut mutator, &mut reactor, root_id);
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
        mount_parked(&view, &mut mutator, &mut reactor, root_id);
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
        mount_parked(&view, &mut mutator, &mut reactor, root_id);
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
        mount_parked(&view, &mut mutator, &mut reactor, root_id);
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

// ── For (Positional Diffing) Tests ───────────────────────────────────────────

#[test]
fn test_for_initial_mount_renders_all_items() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let items: Signal<Vec<String>> = Signal::new(vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ]);

    let view: For<String, Signal<Vec<String>>> = For::new(items, |item| {
        Box::new(Text(item.clone()))
    });
    {
        let mut mutator = doc.mutate();
        mount_parked(&view, &mut mutator, &mut reactor, root_id);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(text.contains("apple"), "Should contain apple: {}", text);
    assert!(text.contains("banana"), "Should contain banana: {}", text);
    assert!(text.contains("cherry"), "Should contain cherry: {}", text);
}

#[test]
fn test_for_add_item_preserves_existing() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let items: Signal<Vec<String>> = Signal::new(vec![
        "alpha".to_string(),
        "beta".to_string(),
    ]);

    let view: For<String, Signal<Vec<String>>> = For::new(items.clone(), |item| {
        Box::new(Text(item.clone()))
    });
    {
        let mut mutator = doc.mutate();
        mount_parked(&view, &mut mutator, &mut reactor, root_id);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);

    // Add a third item
    items.set(vec![
        "alpha".to_string(),
        "beta".to_string(),
        "gamma".to_string(),
    ]);
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(text.contains("alpha"), "Should still contain alpha: {}", text);
    assert!(text.contains("beta"), "Should still contain beta: {}", text);
    assert!(text.contains("gamma"), "Should contain new item gamma: {}", text);
}

#[test]
fn test_for_remove_item_drops_trailing() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let items: Signal<Vec<String>> = Signal::new(vec![
        "one".to_string(),
        "two".to_string(),
        "three".to_string(),
    ]);

    let view: For<String, Signal<Vec<String>>> = For::new(items.clone(), |item| {
        Box::new(Text(item.clone()))
    });
    {
        let mut mutator = doc.mutate();
        mount_parked(&view, &mut mutator, &mut reactor, root_id);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);
    assert!(node_text(&mut doc, root_id).contains("three"));

    // Remove the last item
    items.set(vec!["one".to_string(), "two".to_string()]);
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(text.contains("one"), "Should still contain one: {}", text);
    assert!(text.contains("two"), "Should still contain two: {}", text);
    assert!(!text.contains("three"), "Should NOT contain removed three: {}", text);
}

#[test]
fn test_for_item_content_updates_via_child_reactors() {
    // Positional diffing preserves DOM nodes at the same position. When item
    // content changes in-place (signal value update, not list replacement),
    // child reactors flush and update the DOM text.
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let sig1 = Signal::new("old1".to_string());
    let sig2 = Signal::new("old2".to_string());
    let items: Signal<Vec<Signal<String>>> = Signal::new(vec![sig1.clone(), sig2.clone()]);

    let view: For<Signal<String>, Signal<Vec<Signal<String>>>> = For::new(items, |sig| {
        Box::new(arniko::reactive::ReactiveText::new(sig.clone()))
    });
    {
        let mut mutator = doc.mutate();
        mount_parked(&view, &mut mutator, &mut reactor, root_id);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);
    assert!(node_text(&mut doc, root_id).contains("old1"));
    assert!(node_text(&mut doc, root_id).contains("old2"));

    // Update the existing signals in-place — child reactors flush and update text
    sig1.set("new1".to_string());
    sig2.set("new2".to_string());
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(!text.contains("old1"), "Should NOT contain old1: {}", text);
    assert!(!text.contains("old2"), "Should NOT contain old2: {}", text);
    assert!(text.contains("new1"), "Should contain new1: {}", text);
    assert!(text.contains("new2"), "Should contain new2: {}", text);
}

#[test]
fn test_for_empty_to_populated() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let items: Signal<Vec<String>> = Signal::new(vec![]);

    let view: For<String, Signal<Vec<String>>> = For::new(items.clone(), |item| {
        Box::new(Text(item.clone()))
    });
    {
        let mut mutator = doc.mutate();
        mount_parked(&view, &mut mutator, &mut reactor, root_id);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(text.is_empty(), "Empty list should have no text: {}", text);

    // Populate
    items.set(vec!["first".to_string(), "second".to_string()]);
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(text.contains("first"), "Should contain first: {}", text);
    assert!(text.contains("second"), "Should contain second: {}", text);
}

#[test]
fn test_for_populated_to_empty() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let items: Signal<Vec<String>> = Signal::new(vec![
        "x".to_string(),
        "y".to_string(),
    ]);

    let view: For<String, Signal<Vec<String>>> = For::new(items.clone(), |item| {
        Box::new(Text(item.clone()))
    });
    {
        let mut mutator = doc.mutate();
        mount_parked(&view, &mut mutator, &mut reactor, root_id);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);
    assert!(node_text(&mut doc, root_id).contains("x"));

    // Clear the list
    items.set(vec![]);
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(!text.contains("x"), "Should NOT contain removed x: {}", text);
    assert!(!text.contains("y"), "Should NOT contain removed y: {}", text);
}

#[test]
fn test_for_surviving_items_preserve_dom_nodes() {
    // B-2 spec: positional diffing preserves DOM nodes for items at the same position.
    // We verify this by mounting, capturing child node IDs, mutating the list,
    // and checking that surviving items keep the same DOM node IDs.
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let items: Signal<Vec<String>> = Signal::new(vec![
        "keep1".to_string(),
        "keep2".to_string(),
        "drop".to_string(),
    ]);

    let view: For<String, Signal<Vec<String>>> = For::new(items.clone(), |item| {
        Box::new(Text(item.clone()))
    });
    let container_id = {
        let mut mutator = doc.mutate();
        let id = mount_parked(&view, &mut mutator, &mut reactor, root_id);
        drop(mutator);
        id
    };
    flush_reactive(&mut doc, &mut reactor);

    // Capture child node IDs before mutation
    let child_ids_before = {
        let mutator = doc.mutate();
        let ids = mutator.child_ids(container_id);
        drop(mutator);
        ids
    };
    assert_eq!(child_ids_before.len(), 3, "Should have 3 children");

    // Drop the last item; keep1 and keep2 should survive at positions 0 and 1
    items.set(vec!["keep1".to_string(), "keep2".to_string()]);
    flush_reactive(&mut doc, &mut reactor);

    // Check that survivors kept their DOM node IDs
    let child_ids_after = {
        let mutator = doc.mutate();
        let ids = mutator.child_ids(container_id);
        drop(mutator);
        ids
    };
    assert_eq!(child_ids_after.len(), 2, "Should have 2 children after removal");
    assert_eq!(
        child_ids_after[0], child_ids_before[0],
        "First surviving item should keep its DOM node ID"
    );
    assert_eq!(
        child_ids_after[1], child_ids_before[1],
        "Second surviving item should keep its DOM node ID"
    );
}

#[test]
fn test_for_new_items_get_fresh_dom_nodes() {
    // When items are added, they get new DOM nodes (not reused from dropped items).
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let items: Signal<Vec<String>> = Signal::new(vec!["a".to_string()]);

    let view: For<String, Signal<Vec<String>>> = For::new(items.clone(), |item| {
        Box::new(Text(item.clone()))
    });
    let container_id = {
        let mut mutator = doc.mutate();
        let id = mount_parked(&view, &mut mutator, &mut reactor, root_id);
        drop(mutator);
        id
    };
    flush_reactive(&mut doc, &mut reactor);

    let child_ids_before = {
        let mutator = doc.mutate();
        let ids = mutator.child_ids(container_id);
        drop(mutator);
        ids
    };
    let original_id = child_ids_before[0];

    // Add new items (expand from 1 to 3)
    items.set(vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
    ]);
    flush_reactive(&mut doc, &mut reactor);

    let child_ids_after = {
        let mutator = doc.mutate();
        let ids = mutator.child_ids(container_id);
        drop(mutator);
        ids
    };
    assert_eq!(child_ids_after.len(), 3, "Should have 3 children");
    // Original item at position 0 keeps its node
    assert_eq!(child_ids_after[0], original_id, "First item keeps its DOM node");
    // New items at positions 1 and 2 have different (fresh) IDs
    assert_ne!(child_ids_after[1], original_id, "New item should have fresh DOM node");
    assert_ne!(child_ids_after[2], original_id, "New item should have fresh DOM node");
}

#[test]
fn test_for_nested_reactivity_after_reconciliation() {
    // B-2 spec: nested ReactiveText inside For items keeps updating after
    // list reconciliation (the key fix — previously items got a throwaway Reactor).
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();

    // Each list item is a Signal<String>, wrapped so the template creates ReactiveText
    let item_signal = Signal::new("initial".to_string());
    let items: Signal<Vec<Signal<String>>> = Signal::new(vec![item_signal.clone()]);

    let view: For<Signal<String>, Signal<Vec<Signal<String>>>> = For::new(items.clone(), |sig| {
        Box::new(arniko::reactive::ReactiveText::new(sig.clone()))
    });
    {
        let mut mutator = doc.mutate();
        mount_parked(&view, &mut mutator, &mut reactor, root_id);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);
    assert!(node_text(&mut doc, root_id).contains("initial"));

    // Update the nested signal — nested reactivity should still work
    item_signal.set("updated".to_string());
    flush_reactive(&mut doc, &mut reactor);
    let text = node_text(&mut doc, root_id);
    assert!(
        text.contains("updated"),
        "Nested ReactiveText should update after initial mount: {}",
        text
    );

    // Now trigger a list reconciliation (add an item) and verify nested reactivity still works
    let new_signal = Signal::new("new_item".to_string());
    items.set(vec![item_signal.clone(), new_signal.clone()]);
    flush_reactive(&mut doc, &mut reactor);

    // The first item's signal should still be reactive
    item_signal.set("reconciled".to_string());
    flush_reactive(&mut doc, &mut reactor);
    let text = node_text(&mut doc, root_id);
    assert!(
        text.contains("reconciled"),
        "Nested ReactiveText should keep updating after list reconciliation: {}",
        text
    );

    // The new item should also be reactive
    new_signal.set("new_updated".to_string());
    flush_reactive(&mut doc, &mut reactor);
    let text = node_text(&mut doc, root_id);
    assert!(
        text.contains("new_updated"),
        "New item's ReactiveText should be reactive: {}",
        text
    );
}

#[test]
fn test_for_multiple_reconciliations_no_arena_leak() {
    // B-2 spec: repeated mutations should not leak DOM nodes (arena growth).
    // While we can't directly observe the arena from the test, we can verify
    // that many add-then-remove cycles produce the correct final state.
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let items: Signal<Vec<String>> = Signal::new(vec!["base".to_string()]);

    let view: For<String, Signal<Vec<String>>> = For::new(items.clone(), |item| {
        Box::new(Text(item.clone()))
    });
    let container_id = {
        let mut mutator = doc.mutate();
        let id = mount_parked(&view, &mut mutator, &mut reactor, root_id);
        drop(mutator);
        id
    };
    flush_reactive(&mut doc, &mut reactor);

    // Perform many mutations
    for i in 0..10 {
        // Add 3 items
        items.set(vec![
            "base".to_string(),
            format!("extra_{}_a", i),
            format!("extra_{}_b", i),
            format!("extra_{}_c", i),
        ]);
        flush_reactive(&mut doc, &mut reactor);

        // Remove them, back to 1
        items.set(vec!["base".to_string()]);
        flush_reactive(&mut doc, &mut reactor);
    }

    // After many cycles, should have exactly 1 child
    let child_count = {
        let mutator = doc.mutate();
        let ids = mutator.child_ids(container_id);
        drop(mutator);
        ids.len()
    };
    assert_eq!(child_count, 1, "Should have exactly 1 child after many reconciliations");

    let text = node_text(&mut doc, root_id);
    assert!(text.contains("base"), "Should still contain base: {}", text);
    assert!(!text.contains("extra_"), "Should NOT contain any extra items: {}", text);
}
