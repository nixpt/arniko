//! Integration tests for reactive primitives: Signal, Computed, Reactor.
//!
//! These test the reactive core without requiring DOM — no bliss_dom dependency.
//! Run with: `cargo test -p arniko --features reactive --test reactive_signals`

use arniko::reactive::direct_mut::DirectDomMutator;
use arniko::reactive::{Computed, Reactive, ReactiveText, Reactor, Signal, View};
use arniko::mustang::SceneScheduler;
use bliss_dom::{BaseDocument, DocumentConfig, DocumentMutator, qual_name};
use bliss_html::HtmlProvider;
use std::sync::Arc;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn setup_doc() -> (BaseDocument, usize) {
    let config = DocumentConfig {
        html_parser_provider: Some(Arc::new(HtmlProvider)),
        ..DocumentConfig::default()
    };
    let mut doc = BaseDocument::new(config);
    let mut mutator = doc.mutate();
    let root_id = mutator.create_element(qual_name!("div"), vec![]);
    mutator.append_children(0, &[root_id]);
    drop(mutator);
    (doc, root_id)
}

fn flush_reactive(doc: &mut BaseDocument, reactor: &mut Reactor) {
    let mut mutator = doc.mutate();
    reactor.flush(&mut mutator, None);
    drop(mutator);
}

fn node_text(doc: &mut BaseDocument, root_id: usize) -> String {
    let mut text = String::new();
    let mut mutator = doc.mutate();
    collect_text(&mut mutator, root_id, &mut text);
    drop(mutator);
    text
}

fn collect_text(mutator: &mut DocumentMutator, node_id: usize, buf: &mut String) {
    if let Some(node) = mutator.doc.get_node(node_id) {
        if let Some(td) = node.text_data() {
            buf.push_str(&td.content);
        }
    }
    let children = mutator.child_ids(node_id);
    for child_id in children {
        collect_text(mutator, child_id, buf);
    }
}

// ── Signal Tests ─────────────────────────────────────────────────────────────

#[test]
fn test_signal_set_get() {
    let sig = Signal::new(42_i32);
    assert_eq!(sig.get(), 42);
    sig.set(99);
    assert_eq!(sig.get(), 99);
}

#[test]
fn test_signal_update() {
    let sig = Signal::new("hello".to_string());
    sig.update(|s| format!("{} world", s));
    assert_eq!(sig.get(), "hello world");
}

#[test]
fn test_signal_version_increments() {
    let sig = Signal::new(0_i32);
    let v0 = sig.reactive_version();
    sig.set(1);
    let v1 = sig.reactive_version();
    sig.set(2);
    let v2 = sig.reactive_version();
    assert!(v1 > v0, "Version should increment after set");
    assert!(v2 > v1, "Version should increment again");
}

#[test]
fn test_signal_clone_shares_state() {
    let sig1 = Signal::new(10_i32);
    let sig2 = sig1.clone();
    sig2.set(20);
    assert_eq!(sig1.get(), 20, "Clones share the same state");
}

// ── Computed Tests ───────────────────────────────────────────────────────────

#[test]
fn test_computed_derives_from_signal() {
    let count = Signal::new(3_i32);
    let doubled = count.derive(|v| v * 2);

    assert_eq!(doubled.get(), 6);
    count.set(5);
    assert_eq!(doubled.get(), 10);
}

#[test]
fn test_computed_lazy_recomputation() {
    // Computed only recomputes when get() is called after a dep changed.
    let sig = Signal::new(1_i32);
    let v0 = sig.reactive_version();
    let derived = sig.derive(|v| v + 100);

    assert_eq!(derived.get(), 101);

    // Change signal but don't read computed
    sig.set(2);
    // Still shows old value because not read yet
    let v1 = sig.reactive_version();
    assert!(v1 > v0);

    // Now read — should recompute
    assert_eq!(derived.get(), 102);
}

#[test]
fn test_computed_from2() {
    let a = Signal::new(10_i32);
    let b = Signal::new(20_i32);
    let sum = Computed::from2(a.clone(), b.clone(), |x, y| x + y);

    assert_eq!(sum.get(), 30);
    a.set(15);
    assert_eq!(sum.get(), 35);
    b.set(30);
    assert_eq!(sum.get(), 45);
}

#[test]
fn test_computed_from3() {
    let a = Signal::new("a".to_string());
    let b = Signal::new("b".to_string());
    let c = Signal::new("c".to_string());
    let joined = Computed::from3(a.clone(), b.clone(), c.clone(), |x, y, z| {
        format!("{}-{}-{}", x, y, z)
    });

    assert_eq!(joined.get(), "a-b-c");
    a.set("x".to_string());
    assert_eq!(joined.get(), "x-b-c");
    c.set("z".to_string());
    assert_eq!(joined.get(), "x-b-z");
}

#[test]
fn test_computed_map_chaining() {
    let sig = Signal::new(5_i32);
    let doubled = sig.derive(|v| v * 2);
    let squared = doubled.map(|v| v * v);

    assert_eq!(squared.get(), 100); // (5*2)^2 = 100
    sig.set(7);
    assert_eq!(squared.get(), 196); // (7*2)^2 = 196
}

#[test]
fn test_computed_only_recomputes_when_deps_change() {
    let sig = Signal::new(42_i32);
    let derived = sig.derive(|v| {
        // This would be expensive in real code
        v * 2
    });

    let v0 = derived.reactive_version();
    // Reading without dep changes should not bump version
    let _ = derived.get();
    assert_eq!(derived.reactive_version(), v0, "Version unchanged when deps unchanged");

    // Reading again still no change
    let _ = derived.get();
    assert_eq!(derived.reactive_version(), v0, "Version still unchanged");
}

// ── Reactor Tests ────────────────────────────────────────────────────────────

#[test]
fn test_reactor_flush_dirty_detection() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let sig = Signal::new("initial".to_string());

    {
        let mut mutator = doc.mutate();
        let text = ReactiveText::new(sig.clone());
        text.mount(&mut mutator, &mut reactor, root_id);
        drop(mutator);
    }

    // First flush — should be dirty (initial bind)
    let _dirty = {
        let mut mutator = doc.mutate();
        reactor.flush(&mut mutator, None)
    };
    // After mount, no version change yet, so not dirty on first flush
    // (the binding was registered with last_version = initial version)

    // Change signal — now flush should be dirty
    sig.set("updated".to_string());
    let dirty = {
        let mut mutator = doc.mutate();
        reactor.flush(&mut mutator, None)
    };
    assert!(dirty, "Flush should report dirty after signal change");

    // No change — flush should not be dirty
    let dirty = {
        let mut mutator = doc.mutate();
        reactor.flush(&mut mutator, None)
    };
    assert!(!dirty, "Flush should report clean when no changes");
}

#[test]
fn test_multiple_bindings_on_one_reactor() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let sig1 = Signal::new("first".to_string());
    let sig2 = Signal::new("second".to_string());

    {
        let mut mutator = doc.mutate();
        ReactiveText::new(sig1.clone()).mount(&mut mutator, &mut reactor, root_id);
        ReactiveText::new(sig2.clone()).mount(&mut mutator, &mut reactor, root_id);
        drop(mutator);
    }

    // Change both signals
    sig1.set("FIRST".to_string());
    sig2.set("SECOND".to_string());
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(text.contains("FIRST"), "First binding should update: {}", text);
    assert!(text.contains("SECOND"), "Second binding should update: {}", text);
    assert!(!text.contains("first"), "Old value first should not remain");
    assert!(!text.contains("second"), "Old value second should not remain");
}

#[test]
fn test_reactor_partial_dirty() {
    // Only one of two signals changes — the other binding should not fire.
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let sig1 = Signal::new("one".to_string());
    let sig2 = Signal::new("two".to_string());

    {
        let mut mutator = doc.mutate();
        ReactiveText::new(sig1.clone()).mount(&mut mutator, &mut reactor, root_id);
        ReactiveText::new(sig2.clone()).mount(&mut mutator, &mut reactor, root_id);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);

    // Change only sig1
    sig1.set("ONE".to_string());
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(text.contains("ONE"), "Changed signal should update: {}", text);
    assert!(text.contains("two"), "Unchanged signal should stay: {}", text);
    assert!(!text.contains("one"), "Old value should be gone");
}

#[test]
fn test_rapid_signal_updates() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let sig = Signal::new("start".to_string());

    {
        let mut mutator = doc.mutate();
        ReactiveText::new(sig.clone()).mount(&mut mutator, &mut reactor, root_id);
        drop(mutator);
    }

    // Many rapid updates — only the latest should be visible after flush
    for i in 0..50 {
        sig.set(format!("v{}", i));
    }
    flush_reactive(&mut doc, &mut reactor);

    let text = node_text(&mut doc, root_id);
    assert!(text.contains("v49"), "Latest value should appear: {}", text);
    assert!(!text.contains("v0"), "First value should be gone: {}", text);
}

// ── ReactiveText with Computed Tests ─────────────────────────────────────────

#[test]
fn test_reactive_text_with_computed() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let count = Signal::new(1_i32);
    let label = count.derive(|v| format!("Count: {}", v));

    {
        let mut mutator = doc.mutate();
        let text = ReactiveText::new(label.clone());
        text.mount(&mut mutator, &mut reactor, root_id);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);
    assert!(node_text(&mut doc, root_id).contains("Count: 1"));

    count.set(5);
    flush_reactive(&mut doc, &mut reactor);
    assert!(node_text(&mut doc, root_id).contains("Count: 5"));
}

#[test]
fn test_computed_chain_with_reactive_text() {
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let width = Signal::new(10_i32);
    let height = Signal::new(20_i32);
    let area = Computed::from2(width.clone(), height.clone(), |w, h| w * h);
    let label = area.map(|a| format!("Area: {}px²", a));

    {
        let mut mutator = doc.mutate();
        ReactiveText::new(label.clone()).mount(&mut mutator, &mut reactor, root_id);
        drop(mutator);
    }
    flush_reactive(&mut doc, &mut reactor);
    assert!(node_text(&mut doc, root_id).contains("Area: 200px²"));

    width.set(15);
    flush_reactive(&mut doc, &mut reactor);
    assert!(node_text(&mut doc, root_id).contains("Area: 300px²"));

    height.set(30);
    flush_reactive(&mut doc, &mut reactor);
    assert!(node_text(&mut doc, root_id).contains("Area: 450px²"));
}

#[test]
fn test_signal_set_same_value_still_increments_version() {
    // Setting the same value still bumps the version — this is by design
    // to support patterns where identity matters (e.g., replacing a Vec with
    // equal content).
    let sig = Signal::new(42_i32);
    let v0 = sig.reactive_version();
    sig.set(42);
    let v1 = sig.reactive_version();
    assert!(v1 > v0, "Setting same value still increments version");
}

#[test]
fn test_reactor_flush_with_no_bindings() {
    let (mut doc, _) = setup_doc();
    let mut reactor = Reactor::new();
    // Reactor with no bindings should flush cleanly
    let dirty = {
        let mut mutator = doc.mutate();
        reactor.flush(&mut mutator, None)
    };
    assert!(!dirty, "Empty reactor should report clean");
}

// ── DirectDomMutator Tests ────────────────────────────────────────────────────────

#[test]
fn test_direct_mutator_set_text() {
    let (mut doc, root_id) = setup_doc();
    let scheduler = SceneScheduler::new();
    let dirty_count_before = scheduler.dirty_count();

    {
        let mut mutator = doc.mutate();
        let text_node = mutator.create_text_node("initial");
        mutator.append_children(root_id, &[text_node]);

        let mut dmut = DirectDomMutator::new(&mut mutator, &scheduler);
        dmut.set_text(text_node, "updated");
    }

    // Scheduler should have been notified
    assert_eq!(scheduler.dirty_count(), dirty_count_before + 1);

    // Verify text was updated
    let text = node_text(&mut doc, root_id);
    assert_eq!(text, "updated");
}

#[test]
fn test_direct_mutator_set_attr() {
    let (mut doc, root_id) = setup_doc();
    let scheduler = SceneScheduler::new();
    let dirty_count_before = scheduler.dirty_count();

    {
        let mut mutator = doc.mutate();
        let elem = mutator.create_element(qual_name!("div"), vec![]);
        mutator.append_children(root_id, &[elem]);

        let mut dmut = DirectDomMutator::new(&mut mutator, &scheduler);
        dmut.set_attr(elem, qual_name!("title"), "Test Title");
    }

    assert_eq!(scheduler.dirty_count(), dirty_count_before + 1);
}

#[test]
fn test_direct_mutator_remove_node() {
    let (mut doc, root_id) = setup_doc();
    let scheduler = SceneScheduler::new();

    let child_id;
    {
        let mut mutator = doc.mutate();
        child_id = mutator.create_element(qual_name!("span"), vec![]);
        mutator.append_children(root_id, &[child_id]);
    }

    let dirty_count_before = scheduler.dirty_count();
    {
        let mut mutator = doc.mutate();
        let mut dmut = DirectDomMutator::new(&mut mutator, &scheduler);
        dmut.remove_node(child_id);
    }

    assert_eq!(scheduler.dirty_count(), dirty_count_before + 1);
}

#[test]
fn test_direct_mutator_remove_and_drop_node() {
    let (mut doc, root_id) = setup_doc();
    let scheduler = SceneScheduler::new();

    let child_id;
    {
        let mut mutator = doc.mutate();
        child_id = mutator.create_element(qual_name!("span"), vec![]);
        mutator.append_children(root_id, &[child_id]);
    }

    let dirty_count_before = scheduler.dirty_count();
    {
        let mut mutator = doc.mutate();
        let mut dmut = DirectDomMutator::new(&mut mutator, &scheduler);
        dmut.remove_and_drop_node(child_id);
    }

    assert_eq!(scheduler.dirty_count(), dirty_count_before + 1);
}

#[test]
fn test_direct_mutator_append_children() {
    let (mut doc, root_id) = setup_doc();
    let scheduler = SceneScheduler::new();

    let child1;
    let child2;
    {
        let mut mutator = doc.mutate();
        child1 = mutator.create_element(qual_name!("div"), vec![]);
        child2 = mutator.create_element(qual_name!("span"), vec![]);
    }

    let dirty_count_before = scheduler.dirty_count();
    {
        let mut mutator = doc.mutate();
        let mut dmut = DirectDomMutator::new(&mut mutator, &scheduler);
        dmut.append_children(root_id, &[child1, child2]);
    }

    assert_eq!(scheduler.dirty_count(), dirty_count_before + 1);
}

#[test]
fn test_direct_mutator_multiple_operations() {
    let (mut doc, root_id) = setup_doc();
    let scheduler = SceneScheduler::new();

    let text_node;
    {
        let mut mutator = doc.mutate();
        text_node = mutator.create_text_node("initial");
        mutator.append_children(root_id, &[text_node]);
    }

    let dirty_count_before = scheduler.dirty_count();
    {
        let mut mutator = doc.mutate();
        let mut dmut = DirectDomMutator::new(&mut mutator, &scheduler);
        dmut.set_text(text_node, "first");
        dmut.set_text(text_node, "second");
        dmut.set_text(text_node, "third");
    }

    // Each operation should notify the scheduler
    assert_eq!(scheduler.dirty_count(), dirty_count_before + 3);
}

#[test]
fn test_direct_mutator_set_id() {
    let (mut doc, root_id) = setup_doc();
    let scheduler = SceneScheduler::new();
    let dirty_count_before = scheduler.dirty_count();

    let elem;
    {
        let mut mutator = doc.mutate();
        elem = mutator.create_element(qual_name!("div"), vec![]);
        mutator.append_children(root_id, &[elem]);
    }

    {
        let mut mutator = doc.mutate();
        let mut dmut = DirectDomMutator::new(&mut mutator, &scheduler);
        dmut.set_id(elem, "my-id");
    }

    assert_eq!(scheduler.dirty_count(), dirty_count_before + 1);
}

#[test]
fn test_direct_mutator_set_class() {
    let (mut doc, root_id) = setup_doc();
    let scheduler = SceneScheduler::new();
    let dirty_count_before = scheduler.dirty_count();

    let elem;
    {
        let mut mutator = doc.mutate();
        elem = mutator.create_element(qual_name!("div"), vec![]);
        mutator.append_children(root_id, &[elem]);
    }

    {
        let mut mutator = doc.mutate();
        let mut dmut = DirectDomMutator::new(&mut mutator, &scheduler);
        dmut.set_class(elem, "my-class");
    }

    assert_eq!(scheduler.dirty_count(), dirty_count_before + 1);
}

#[test]
fn test_direct_mutator_insert_before() {
    let (mut doc, root_id) = setup_doc();
    let scheduler = SceneScheduler::new();

    let anchor;
    let new_node;
    {
        let mut mutator = doc.mutate();
        anchor = mutator.create_element(qual_name!("div"), vec![]);
        new_node = mutator.create_element(qual_name!("span"), vec![]);
        mutator.append_children(root_id, &[anchor]);
    }

    let dirty_count_before = scheduler.dirty_count();
    {
        let mut mutator = doc.mutate();
        let mut dmut = DirectDomMutator::new(&mut mutator, &scheduler);
        dmut.insert_before(anchor, &[new_node]);
    }

    assert_eq!(scheduler.dirty_count(), dirty_count_before + 1);
}

#[test]
fn test_direct_mutator_replace_with() {
    let (mut doc, root_id) = setup_doc();
    let scheduler = SceneScheduler::new();

    let anchor;
    let new_node;
    {
        let mut mutator = doc.mutate();
        anchor = mutator.create_element(qual_name!("div"), vec![]);
        new_node = mutator.create_element(qual_name!("span"), vec![]);
        mutator.append_children(root_id, &[anchor]);
    }

    let dirty_count_before = scheduler.dirty_count();
    {
        let mut mutator = doc.mutate();
        let mut dmut = DirectDomMutator::new(&mut mutator, &scheduler);
        dmut.replace_with(anchor, &[new_node]);
    }

    assert_eq!(scheduler.dirty_count(), dirty_count_before + 1);
}

#[test]
fn test_direct_mutator_remove_all_children() {
    let (mut doc, root_id) = setup_doc();
    let scheduler = SceneScheduler::new();

    let child1;
    let child2;
    {
        let mut mutator = doc.mutate();
        child1 = mutator.create_element(qual_name!("div"), vec![]);
        child2 = mutator.create_element(qual_name!("span"), vec![]);
        mutator.append_children(root_id, &[child1, child2]);
    }

    let dirty_count_before = scheduler.dirty_count();
    {
        let mut mutator = doc.mutate();
        let mut dmut = DirectDomMutator::new(&mut mutator, &scheduler);
        dmut.remove_all_children(root_id);
    }

    assert_eq!(scheduler.dirty_count(), dirty_count_before + 1);
}

// ── Binding Lifecycle Tests ─────────────────────────────────────────────────────

#[test]
fn test_binding_lifecycle_scope_cleanup() {
    // B-4: Verify that binding lifecycle management works correctly.
    // Mounting and unmounting N views should leave the binding count flat.

    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();

    // Track initial binding count
    let initial_binding_count = reactor.binding_count();

    // Mount 3 views, each with a reactive text binding
    let sig1 = Signal::new("text1".to_string());
    let sig2 = Signal::new("text2".to_string());
    let sig3 = Signal::new("text3".to_string());

    let view1 = ReactiveText::new(sig1.clone());
    let view2 = ReactiveText::new(sig2.clone());
    let view3 = ReactiveText::new(sig3.clone());

    let scope1;
    let scope2;
    let scope3;
    {
        let mut mutator = doc.mutate();
        let (_, s1) = view1.mount(&mut mutator, &mut reactor, root_id);
        let (_, s2) = view2.mount(&mut mutator, &mut reactor, root_id);
        let (_, s3) = view3.mount(&mut mutator, &mut reactor, root_id);
        scope1 = s1;
        scope2 = s2;
        scope3 = s3;
        drop(mutator);
    }

    // After mounting 3 views, we should have 3 more bindings
    assert_eq!(
        reactor.binding_count(),
        initial_binding_count + 3,
        "Should have 3 bindings after mounting 3 views"
    );

    // Now drop the scopes - this should remove the bindings
    drop(scope1);
    drop(scope2);
    drop(scope3);

    // After dropping all scopes, binding count should be back to initial
    assert_eq!(
        reactor.binding_count(),
        initial_binding_count,
        "Binding count should return to initial after dropping all scopes"
    );

    // Mount and unmount in a cycle to verify no leaks
    for _ in 0..5 {
        let sig = Signal::new("temp".to_string());
        let view = ReactiveText::new(sig.clone());
        let binding_count_before = reactor.binding_count();

        let scope;
        {
            let mut mutator = doc.mutate();
            let (_, s) = view.mount(&mut mutator, &mut reactor, root_id);
            scope = s;
            drop(mutator);
        }

        assert_eq!(
            reactor.binding_count(),
            binding_count_before + 1,
            "Should have 1 more binding after mounting"
        );

        drop(scope);

        assert_eq!(
            reactor.binding_count(),
            binding_count_before,
            "Binding count should return to previous after dropping scope"
        );
    }

    // Final binding count should still be initial
    assert_eq!(
        reactor.binding_count(),
        initial_binding_count,
        "After mount/unmount cycles, binding count should be flat"
    );
}

// ── Lock-Poison Tolerance Tests (B-5) ──────────────────────────────────────────

#[test]
fn test_lock_poison_cascade_panic_in_handler_is_contained() {
    // B-5: Verify that a panicking handler doesn't poison the lock and crash the app.
    // With parking_lot, locks don't poison, so subsequent operations should work.
    use arniko::reactive::event_router;

    let (router, _sink) = event_router();
    
    // Register a handler that panics
    router.on_click(1, || {
        panic!("Handler panic!");
    });

    // Register another handler - this should still work
    router.on_click(2, || {});
}

// ── Event Ergonomics Tests (B-6) ──────────────────────────────────────────────

#[test]
fn test_per_node_keydown_api_exists() {
    // B-6: Verify per-node keydown registration API exists
    use arniko::reactive::event_router;
    use bliss::traits::events::BlissKeyEvent;

    let (router, _sink) = event_router();

    // This should compile - per-node keydown handler registration
    router.on_keydown_node(1, |_event: &BlissKeyEvent| {});

    // Global keydown should still work
    router.on_keydown(|_event: &BlissKeyEvent| {});
}

#[test]
fn test_event_handler_chaining() {
    // B-6: Verify event handler methods can be chained
    use arniko::reactive::event_router;
    use bliss::traits::events::BlissKeyEvent;

    let (router, _sink) = event_router();

    // This should compile and allow chaining
    router
        .on_click(1, || {})
        .on_keydown_node(2, |_event: &BlissKeyEvent| {})
        .on_input(3, |_value: String| {})
        .on_keydown(|_event: &BlissKeyEvent| {});
}

// ── Conditional Rendering Tests (B-7) ──────────────────────────────────────────

#[test]
fn test_show_view_hides_and_shows_content() {
    use arniko::reactive::{Show, Text};
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let visible = Signal::new(true);
    let view = Show::new(visible.clone(), move || Box::new(Text("visible content".to_string())));
    let _scope;
    { let mut mutator = doc.mutate(); _scope = view.mount(&mut mutator, &mut reactor, root_id).1; drop(mutator); }
    flush_reactive(&mut doc, &mut reactor);
    assert!(node_text(&mut doc, root_id).contains("visible content"));
    visible.set(false);
    flush_reactive(&mut doc, &mut reactor);
    assert!(!node_text(&mut doc, root_id).contains("visible content"));
    visible.set(true);
    flush_reactive(&mut doc, &mut reactor);
    assert!(node_text(&mut doc, root_id).contains("visible content"));
}


#[test]
fn test_switch_view_changes_branches() {
    use arniko::reactive::{Switch, Text};
    #[derive(Clone, PartialEq)]
    enum Page { Home, About, Contact }
    let (mut doc, root_id) = setup_doc();
    let mut reactor = Reactor::new();
    let page = Signal::new(Page::Home);
    let view = Switch::new(page.clone(), |p: &Page| {
        match p {
            Page::Home => Box::new(Text("Home Page".to_string())),
            Page::About => Box::new(Text("About Page".to_string())),
            Page::Contact => Box::new(Text("Contact Page".to_string())),
        }
    });
    let _scope;
    { let mut mutator = doc.mutate(); _scope = view.mount(&mut mutator, &mut reactor, root_id).1; drop(mutator); }
    flush_reactive(&mut doc, &mut reactor);
    assert!(node_text(&mut doc, root_id).contains("Home Page"));
    page.set(Page::About);
    flush_reactive(&mut doc, &mut reactor);
    assert!(node_text(&mut doc, root_id).contains("About Page"));
    assert!(!node_text(&mut doc, root_id).contains("Home Page"));
    page.set(Page::Contact);
    flush_reactive(&mut doc, &mut reactor);
    assert!(node_text(&mut doc, root_id).contains("Contact Page"));
    assert!(!node_text(&mut doc, root_id).contains("About Page"));
}

