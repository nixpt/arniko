//! Direct DOM Mutation API — a thin, high-level wrapper over
//! `bliss_dom::DocumentMutator` that notifies a `SceneScheduler` on
//! every mutation.
//!
//! This is the "dom-capability native path" from the Phase 5 roadmap.
//! It complements the reactive `Reactor` (which patches the DOM in
//! response to signal changes) by giving capability authors a direct,
//! intent-revealing API for mutating the DOM without going through
//! signals.
//!
//! Every method on `DirectDomMutator` notifies the scheduler (region-tracked
//! where the node's layout rect is known, whole-scene otherwise)
//! after the underlying mutation succeeds, so the next frame's
//! `set_scene_effects` hook will re-apply GPU effects over the new
//! DOM state.
//!
//! # Example
//!
//! ```rust,ignore
//! use arniko::reactive::direct_mut::DirectDomMutator;
//! use arniko::reactive::scheduler::SceneScheduler;
//! use bliss_dom::qual_name;
//!
//! let scheduler = SceneScheduler::new();
//! let mut m = doc.inner_mut().mutate();
//! let m = DirectDomMutator::new(&mut m, &scheduler);
//!
//! m.set_text(text_node_id, "Hello, world!")?;
//! m.set_attr(button_id, qual_name!("disabled", html), "")?;
//! m.remove_node(old_child_id)?;
//! ```

use bliss_dom::{DocumentMutator, QualName, qual_name};

use crate::mustang::Region;
use crate::mustang::scheduler::SceneScheduler;

/// Direct DOM mutation API. Wraps a `DocumentMutator` and notifies a
/// `SceneScheduler` on every successful mutation.
///
/// # Region-aware notifications
///
/// Since mustang 0.3.0 the scheduler distinguishes whole-scene dirtiness
/// (`on_dom_changed`) from per-region dirtiness (`on_region_changed`).
/// Every mutation here tries to report WHERE it happened: the mutated
/// node's last-computed layout rect (from
/// `BaseDocument::get_client_bounding_rect`), expanded conservatively
/// because a mutation can grow content beyond the pre-mutation rect
/// (lengthened text, an attribute adding padding, ...). When no rect is
/// available (node not in the tree, layout never computed) the fallback
/// is the whole scene.
///
/// The frame side consumes this via `SceneScheduler::take_dirty()`: a
/// whole-scene flag re-checks every effect; otherwise only effects
/// intersecting the returned rects need re-application.
pub struct DirectDomMutator<'doc, 'data, 'sched> {
    inner: &'doc mut DocumentMutator<'data>,
    scheduler: &'sched SceneScheduler,
}

/// Padding applied around a mutated node's layout rect before reporting
/// it to the scheduler — mutations may grow the node (longer text,
/// size-affecting attributes), so the invalidated area extends past the
/// last-computed geometry.
const REGION_PAD: f32 = 64.0;

impl<'doc, 'data, 'sched> DirectDomMutator<'doc, 'data, 'sched> {
    pub fn new(inner: &'doc mut DocumentMutator<'data>, scheduler: &'sched SceneScheduler) -> Self {
        Self { inner, scheduler }
    }

    /// Borrow the underlying `DocumentMutator` for operations not
    /// covered by this trait (escape hatch).
    pub fn raw(&mut self) -> &mut DocumentMutator<'data> {
        self.inner
    }

    /// Notify the scheduler about a mutation touching `node_id`.
    ///
    /// Region-tracked when the node has a computed layout rect
    /// (expanded by [`REGION_PAD`]); whole-scene fallback otherwise.
    /// Must be called BEFORE the mutation for removals (the node leaves
    /// the tree) and AFTER for in-place edits — either order yields the
    /// same last-layout rect for in-place edits, but removals need the
    /// pre-removal lookup.
    fn notify(&mut self, node_id: usize) {
        if let Some(rect) = self.inner.doc.get_client_bounding_rect(node_id) {
            let region = Region::new(
                ((rect.x as f32) - REGION_PAD).max(0.0),
                ((rect.y as f32) - REGION_PAD).max(0.0),
                (rect.width as f32) + REGION_PAD * 2.0,
                (rect.height as f32) + REGION_PAD * 2.0,
            );
            self.scheduler.on_region_changed(region);
        } else {
            self.scheduler.on_dom_changed();
        }
    }

    /// Set the text content of a text node. No-op if `node_id` is not
    /// a text node (matches `DocumentMutator::set_node_text` semantics).
    pub fn set_text(&mut self, node_id: usize, text: &str) {
        self.notify(node_id);
        self.inner.set_node_text(node_id, text);
    }

    /// Set an attribute on an element. Notifies the scheduler.
    pub fn set_attr(&mut self, node_id: usize, name: QualName, value: &str) {
        self.notify(node_id);
        self.inner.set_attribute(node_id, name, value);
    }

    /// Clear an attribute. Notifies the scheduler.
    pub fn remove_attr(&mut self, node_id: usize, name: QualName) {
        self.notify(node_id);
        self.inner.clear_attribute(node_id, name);
    }

    /// Remove a node from its parent (kept alive, not dropped). Notifies the scheduler.
    pub fn remove_node(&mut self, node_id: usize) {
        self.notify(node_id);
        self.inner.remove_node(node_id);
    }

    /// Remove a node and drop it. Notifies the scheduler.
    pub fn remove_and_drop_node(&mut self, node_id: usize) {
        self.notify(node_id);
        self.inner.remove_and_drop_node(node_id);
    }

    /// Remove all children of a node, dropping them. Notifies the scheduler.
    pub fn remove_all_children(&mut self, parent_id: usize) {
        // Pre-removal: the parent's rect covers the doomed children;
        // post-removal they are gone from the tree (fallback would fire).
        self.notify(parent_id);
        self.inner.remove_and_drop_all_children(parent_id);
    }

    /// Append pre-created children to a parent. Notifies the scheduler.
    pub fn append_children(&mut self, parent_id: usize, child_ids: &[usize]) {
        self.notify(parent_id);
        self.inner.append_children(parent_id, child_ids);
    }

    /// Insert nodes before an anchor. Notifies the scheduler.
    pub fn insert_before(&mut self, anchor_id: usize, new_node_ids: &[usize]) {
        self.notify(anchor_id);
        self.inner.insert_nodes_before(anchor_id, new_node_ids);
    }

    /// Insert nodes after an anchor. Notifies the scheduler.
    pub fn insert_after(&mut self, anchor_id: usize, new_node_ids: &[usize]) {
        self.notify(anchor_id);
        self.inner.insert_nodes_after(anchor_id, new_node_ids);
    }

    /// Replace an anchor node with one or more new nodes. Notifies the scheduler.
    pub fn replace_with(&mut self, anchor_id: usize, new_node_ids: &[usize]) {
        self.notify(anchor_id);
        self.inner.replace_node_with(anchor_id, new_node_ids);
    }

    /// Move all children from `old_parent_id` to `new_parent_id`. Notifies the scheduler.
    pub fn reparent_children(&mut self, old_parent_id: usize, new_parent_id: usize) {
        self.notify(old_parent_id);
        self.notify(new_parent_id);
        self.inner.reparent_children(old_parent_id, new_parent_id);
    }

    /// Parse and set `innerHTML` on a node. Notifies the scheduler.
    pub fn set_inner_html(&mut self, node_id: usize, html: &str) {
        self.notify(node_id);
        self.inner.set_inner_html(node_id, html);
    }

    /// Convenience: set the `id` attribute. Notifies the scheduler.
    pub fn set_id(&mut self, node_id: usize, id: &str) {
        self.set_attr(node_id, qual_name!("id"), id);
    }

    /// Convenience: set the `class` attribute. Notifies the scheduler.
    pub fn set_class(&mut self, node_id: usize, class: &str) {
        self.set_attr(node_id, qual_name!("class"), class);
    }
}

// NOTE: A `DirectMutError` enum was originally planned here for
// future strict-mode validation, but every underlying `DocumentMutator`
// method is currently infallible (it silently no-ops on type mismatches).
// The enum was removed to avoid dead-code warnings; re-introduce it
// here when strict-mode validation lands.

#[cfg(test)]
mod tests {
    use super::*;
    use bliss::dom::Document;
    use bliss::html::HtmlDocument;
    use bliss_dom::DocumentConfig;

    /// A laid-out document with a positioned, sized element.
    fn laid_out_doc() -> HtmlDocument {
        let mut doc = HtmlDocument::from_html(
            r#"<!DOCTYPE html><html><head></head><body>
               <div id="panel" style="position:absolute;left:100px;top:50px;width:200px;height:80px;">
                   <span id="label">hello</span>
               </div>
               </body></html>"#,
            DocumentConfig::default(),
        );
        // Compute layout so `get_client_bounding_rect` has geometry to read.
        doc.inner_mut().resolve_layout();
        doc
    }

    #[test]
    fn in_place_edit_reports_node_region_not_full() {
        let mut doc = laid_out_doc();
        let scheduler = SceneScheduler::new();

        let label_rect = {
            let inner = doc.inner();
            let label_id = inner.get_element_by_id("label").unwrap();
            inner
                .get_client_bounding_rect(label_id)
                .expect("label node has a layout rect after resolve_layout")
        };

        let label_id = doc.inner().get_element_by_id("label").unwrap();
        let mut inner = doc.inner_mut();
        let mut mutator = inner.mutate();
        let mut dm = DirectDomMutator::new(&mut mutator, &scheduler);
        dm.set_text(label_id, "changed");
        drop(dm);
        drop(mutator);
        drop(inner);

        let state = scheduler.take_dirty();
        assert!(
            !state.full,
            "in-place edit on a laid-out node must be region-tracked, not whole-scene"
        );
        assert_eq!(state.regions.len(), 1);
        // The reported region must cover the node's rect (expanded by REGION_PAD).
        let r = &state.regions[0];
        assert!(r.x <= label_rect.x as f32 && r.y <= label_rect.y as f32);
        assert!(
            r.x + r.width >= (label_rect.x + label_rect.width) as f32
                && r.y + r.height >= (label_rect.y + label_rect.height) as f32
        );
    }

    #[test]
    fn unknown_node_falls_back_to_whole_scene() {
        let mut doc = laid_out_doc();
        let scheduler = SceneScheduler::new();

        let mut inner = doc.inner_mut();
        let mut mutator = inner.mutate();
        let mut dm = DirectDomMutator::new(&mut mutator, &scheduler);
        // Node id far out of range — no node, no rect → whole-scene fallback.
        dm.set_text(usize::MAX - 1, "boo");
        drop(dm);
        drop(mutator);
        drop(inner);

        let state = scheduler.take_dirty();
        assert!(
            state.full,
            "mutation on a node without a layout rect must mark the whole scene dirty"
        );
        assert!(state.regions.is_empty());
    }

    #[test]
    fn two_node_edit_accumulates_two_regions() {
        let mut doc = laid_out_doc();
        let scheduler = SceneScheduler::new();

        let (panel_id, label_id) = {
            let inner = doc.inner();
            (
                inner.get_element_by_id("panel").unwrap(),
                inner.get_element_by_id("label").unwrap(),
            )
        };
        let mut inner = doc.inner_mut();
        let mut mutator = inner.mutate();
        let mut dm = DirectDomMutator::new(&mut mutator, &scheduler);
        dm.set_attr(panel_id, qual_name!("title"), "1");
        dm.set_text(label_id, "again");
        drop(dm);
        drop(mutator);
        drop(inner);

        let state = scheduler.take_dirty();
        assert!(!state.full);
        assert_eq!(state.regions.len(), 2, "two tracked mutations → two rects");
    }
}
