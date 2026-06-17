//! Tests for document.rs - mutation round-trips

use crate::{BaseDocument, DocumentConfig, DocumentMutator, local_name, qual_name};

fn setup_doc() -> BaseDocument {
    let config = DocumentConfig::default();
    BaseDocument::new(config)
}

// ── Node Creation and Removal Round-Trips ──────────────────────────────────────

#[test]
fn test_create_and_remove_element() {
    let mut doc = setup_doc();
    let mut mutator = doc.mutate();

    let root = mutator.create_element(qual_name!("div"), vec![]);
    let child = mutator.create_element(qual_name!("span"), vec![]);
    mutator.append_children(root, &[child]);

    assert!(mutator.doc.nodes.get(root).is_some());
    assert!(mutator.doc.nodes.get(child).is_some());

    mutator.remove_and_drop_node(child);

    assert!(mutator.doc.nodes.get(root).is_some());
    assert!(mutator.doc.nodes.get(child).is_none());
    drop(mutator);
}

#[test]
fn test_create_text_node_and_get_content() {
    let mut doc = setup_doc();
    let mut mutator = doc.mutate();

    let text_node = mutator.create_text_node("Hello, world!");
    assert!(mutator.doc.nodes.get(text_node).is_some());
    
    if let Some(node) = mutator.doc.nodes.get(text_node) {
        if let Some(text_data) = node.text_data() {
            assert_eq!(text_data.content, "Hello, world!");
        } else {
            panic!("Expected text node");
        }
    }
    drop(mutator);
}

#[test]
fn test_set_node_text() {
    let mut doc = setup_doc();
    let mut mutator = doc.mutate();

    let text_node = mutator.create_text_node("initial");
    mutator.set_node_text(text_node, "updated");

    if let Some(node) = mutator.doc.nodes.get(text_node) {
        if let Some(text_data) = node.text_data() {
            assert_eq!(text_data.content, "updated");
        } else {
            panic!("Expected text node");
        }
    }
    drop(mutator);
}

// ── Attribute Round-Trips ────────────────────────────────────────────────────

#[test]
fn test_set_and_get_attribute() {
    let mut doc = setup_doc();
    let mut mutator = doc.mutate();

    let elem = mutator.create_element(qual_name!("div"), vec![]);
    mutator.set_attribute(elem, qual_name!("id"), "my-element");
    
    if let Some(node) = mutator.doc.nodes.get(elem) {
        assert_eq!(node.attr(local_name!("id")), Some("my-element"));
    } else {
        panic!("Element not found");
    }
    drop(mutator);
}

#[test]
fn test_set_and_clear_attribute() {
    let mut doc = setup_doc();
    let mut mutator = doc.mutate();

    let elem = mutator.create_element(qual_name!("div"), vec![]);
    mutator.set_attribute(elem, qual_name!("class"), "active");
    
    if let Some(node) = mutator.doc.nodes.get(elem) {
        assert_eq!(node.attr(local_name!("class")), Some("active"));
    }
    
    mutator.clear_attribute(elem, qual_name!("class"));
    if let Some(node) = mutator.doc.nodes.get(elem) {
        assert_eq!(node.attr(local_name!("class")), None);
    }
    drop(mutator);
}

#[test]
fn test_multiple_attributes() {
    let mut doc = setup_doc();
    let mut mutator = doc.mutate();

    let elem = mutator.create_element(qual_name!("div"), vec![]);
    mutator.set_attribute(elem, qual_name!("id"), "test");
    mutator.set_attribute(elem, qual_name!("class"), "foo bar");
    mutator.set_attribute(elem, qual_name!("title"), "My Title");
    
    if let Some(node) = mutator.doc.nodes.get(elem) {
        assert_eq!(node.attr(local_name!("id")), Some("test"));
        assert_eq!(node.attr(local_name!("class")), Some("foo bar"));
        assert_eq!(node.attr(local_name!("title")), Some("My Title"));
    }
    drop(mutator);
}

// ── Tree Structure Round-Trips ────────────────────────────────────────────────

#[test]
fn test_append_children() {
    let mut doc = setup_doc();
    let mut mutator = doc.mutate();

    let parent = mutator.create_element(qual_name!("div"), vec![]);
    let child1 = mutator.create_element(qual_name!("span"), vec![]);
    let child2 = mutator.create_element(qual_name!("p"), vec![]);
    
    mutator.append_children(parent, &[child1, child2]);
    
    let children = mutator.child_ids(parent);
    assert_eq!(children.len(), 2);
    assert!(children.contains(&child1));
    assert!(children.contains(&child2));
    drop(mutator);
}

#[test]
fn test_insert_before() {
    let mut doc = setup_doc();
    let mut mutator = doc.mutate();

    let parent = mutator.create_element(qual_name!("div"), vec![]);
    let anchor = mutator.create_element(qual_name!("span"), vec![]);
    let new_node = mutator.create_element(qual_name!("p"), vec![]);
    
    mutator.append_children(parent, &[anchor]);
    mutator.insert_nodes_before(anchor, &[new_node]);
    
    let children = mutator.child_ids(parent);
    assert_eq!(children.len(), 2);
    let pos_new = children.iter().position(|&id| id == new_node).unwrap();
    let pos_anchor = children.iter().position(|&id| id == anchor).unwrap();
    assert!(pos_new < pos_anchor);
    drop(mutator);
}

#[test]
fn test_insert_after() {
    let mut doc = setup_doc();
    let mut mutator = doc.mutate();

    let parent = mutator.create_element(qual_name!("div"), vec![]);
    let anchor = mutator.create_element(qual_name!("span"), vec![]);
    let new_node = mutator.create_element(qual_name!("p"), vec![]);
    
    mutator.append_children(parent, &[anchor]);
    mutator.insert_nodes_after(anchor, &[new_node]);
    
    let children = mutator.child_ids(parent);
    assert_eq!(children.len(), 2);
    let pos_anchor = children.iter().position(|&id| id == anchor).unwrap();
    let pos_new = children.iter().position(|&id| id == new_node).unwrap();
    assert!(pos_anchor < pos_new);
    drop(mutator);
}

#[test]
fn test_replace_node_with() {
    let mut doc = setup_doc();
    let mut mutator = doc.mutate();

    let parent = mutator.create_element(qual_name!("div"), vec![]);
    let old_node = mutator.create_element(qual_name!("span"), vec![]);
    let new_node = mutator.create_element(qual_name!("p"), vec![]);
    
    mutator.append_children(parent, &[old_node]);
    mutator.replace_node_with(old_node, &[new_node]);
    
    let children = mutator.child_ids(parent);
    assert_eq!(children.len(), 1);
    assert!(children.contains(&new_node));
    assert!(!children.contains(&old_node));
    drop(mutator);
}

#[test]
fn test_remove_all_children() {
    let mut doc = setup_doc();
    let mut mutator = doc.mutate();

    let parent = mutator.create_element(qual_name!("div"), vec![]);
    let child1 = mutator.create_element(qual_name!("span"), vec![]);
    let child2 = mutator.create_element(qual_name!("p"), vec![]);
    
    mutator.append_children(parent, &[child1, child2]);
    assert_eq!(mutator.child_ids(parent).len(), 2);
    
    mutator.remove_and_drop_all_children(parent);
    assert_eq!(mutator.child_ids(parent).len(), 0);
    drop(mutator);
}

#[test]
fn test_reparent_children() {
    let mut doc = setup_doc();
    let mut mutator = doc.mutate();

    let old_parent = mutator.create_element(qual_name!("div"), vec![]);
    let new_parent = mutator.create_element(qual_name!("section"), vec![]);
    let child1 = mutator.create_element(qual_name!("span"), vec![]);
    let child2 = mutator.create_element(qual_name!("p"), vec![]);
    
    mutator.append_children(old_parent, &[child1, child2]);
    assert_eq!(mutator.child_ids(old_parent).len(), 2);
    assert_eq!(mutator.child_ids(new_parent).len(), 0);
    
    mutator.reparent_children(old_parent, new_parent);
    
    assert_eq!(mutator.child_ids(old_parent).len(), 0);
    assert_eq!(mutator.child_ids(new_parent).len(), 2);
    drop(mutator);
}

// ── Inner HTML Round-Trips ────────────────────────────────────────────────────
// Note: These tests are skipped because they require HTML parser configuration
// which is not available in the default test setup.

#[ignore = "Requires HTML parser provider"]
#[test]
fn test_set_inner_html() {
    let mut doc = setup_doc();
    let mut mutator = doc.mutate();

    let elem = mutator.create_element(qual_name!("div"), vec![]);
    mutator.set_inner_html(elem, "<span>Hello</span>");
    
    let children = mutator.child_ids(elem);
    assert_eq!(children.len(), 1);
    drop(mutator);
}

#[ignore = "Requires HTML parser provider"]
#[test]
fn test_set_inner_html_with_multiple_elements() {
    let mut doc = setup_doc();
    let mut mutator = doc.mutate();

    let elem = mutator.create_element(qual_name!("div"), vec![]);
    mutator.set_inner_html(elem, "<span>First</span><span>Second</span>");
    
    let children = mutator.child_ids(elem);
    assert_eq!(children.len(), 2);
    drop(mutator);
}

// ── Complex Round-Trip: Build and Verify Tree ────────────────────────────────

#[test]
fn test_complex_document_round_trip() {
    let mut doc = setup_doc();
    let mut mutator = doc.mutate();

    let root = mutator.create_element(qual_name!("html"), vec![]);
    let head = mutator.create_element(qual_name!("head"), vec![]);
    let body = mutator.create_element(qual_name!("body"), vec![]);
    let title = mutator.create_element(qual_name!("title"), vec![]);
    let h1 = mutator.create_element(qual_name!("h1"), vec![]);
    let p = mutator.create_element(qual_name!("p"), vec![]);
    let text1 = mutator.create_text_node("Hello");
    let text2 = mutator.create_text_node("World");
    
    mutator.set_attribute(root, qual_name!("lang"), "en");
    
    mutator.append_children(root, &[head, body]);
    mutator.append_children(head, &[title]);
    mutator.append_children(body, &[h1, p]);
    mutator.append_children(p, &[text1, text2]);
    
    assert_eq!(mutator.child_ids(root).len(), 2);
    assert_eq!(mutator.child_ids(head).len(), 1);
    assert_eq!(mutator.child_ids(body).len(), 2);
    assert_eq!(mutator.child_ids(p).len(), 2);
    
    if let Some(node) = mutator.doc.nodes.get(root) {
        assert_eq!(node.attr(local_name!("lang")), Some("en"));
    }
    
    assert!(mutator.doc.nodes.get(text1).is_some());
    assert!(mutator.doc.nodes.get(text2).is_some());
    drop(mutator);
}

// ── D-2c-followup safety backstop: empty-doc root_element() NoPanic regression ─

#[test]
fn test_root_element_none_safety() {
    // D-2c-followup safety backstop: an empty BaseDocument (no HTML parsed,
    // no element child of the document root) must not panic in any migrated
    // root_element() call site. The cascade previously chained unwrap-panics
    // from `first_element_child().unwrap().as_element().unwrap()`; this
    // verifies the new `Option<&Node>` shape propagates None to each migrated
    // caller with sensible empty / null result behavior.
    let mut doc = setup_doc();

    // try_root_element() / root_element() both return None on empty doc.
    assert!(doc.try_root_element().is_none());
    assert!(doc.root_element().is_none());

    // hit() returns None on no root (?-propagation in migrated site).
    assert!(doc.hit(10.0, 10.0).is_none());

    // scroll_viewport_by_has_changed() returns false (content_size degrades
    // to taffy::Size::default() — zero dimensions).
    assert!(!doc.scroll_viewport_by_has_changed(0.0, 0.0));

    // resolve() short-circuits via the upstream `is_none` guard + early return;
    // the debug_assert! inside resolve() must hold — this is also a load-bearing
    // test confirming the upstream guard still functions.
    doc.resolve(0.0);

    // scroll_node_by_has_changed() with a stale / non-Element node id is a
    // no-op return false (root_element: Some(root) guard no-ops scroll_node).
    let mut scroll_event_seen = false;
    assert!(!doc.scroll_node_by_has_changed(0, 1.0, 1.0, |_| {
        scroll_event_seen = true;
    }));
    assert!(!scroll_event_seen);

    // set_layout / scroll_event are no-ops on the (non-Element) document root.
    // clear_focus / clear_hover are no-ops on empty doc.
    assert!(!doc.clear_focus());
    assert!(!doc.clear_hover());
}
