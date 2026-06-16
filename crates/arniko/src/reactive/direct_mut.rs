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
//! Every method on `DirectDomMutator` calls `scheduler.on_dom_changed()`
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

use crate::mustang::scheduler::SceneScheduler;

/// Direct DOM mutation API. Wraps a `DocumentMutator` and notifies a
/// `SceneScheduler` on every successful mutation.
pub struct DirectDomMutator<'doc, 'data, 'sched> {
    inner: &'doc mut DocumentMutator<'data>,
    scheduler: &'sched SceneScheduler,
}

impl<'doc, 'data, 'sched> DirectDomMutator<'doc, 'data, 'sched> {
    pub fn new(
        inner: &'doc mut DocumentMutator<'data>,
        scheduler: &'sched SceneScheduler,
    ) -> Self {
        Self { inner, scheduler }
    }

    /// Borrow the underlying `DocumentMutator` for operations not
    /// covered by this trait (escape hatch).
    pub fn raw(&mut self) -> &mut DocumentMutator<'data> {
        self.inner
    }

    /// Set the text content of a text node. No-op if `node_id` is not
    /// a text node (matches `DocumentMutator::set_node_text` semantics).
    pub fn set_text(&mut self, node_id: usize, text: &str) {
        self.inner.set_node_text(node_id, text);
        self.scheduler.on_dom_changed();
    }

    /// Set an attribute on an element. Notifies the scheduler.
    pub fn set_attr(&mut self, node_id: usize, name: QualName, value: &str) {
        self.inner.set_attribute(node_id, name, value);
        self.scheduler.on_dom_changed();
    }

    /// Clear an attribute. Notifies the scheduler.
    pub fn remove_attr(&mut self, node_id: usize, name: QualName) {
        self.inner.clear_attribute(node_id, name);
        self.scheduler.on_dom_changed();
    }

    /// Remove a node from its parent (kept alive, not dropped). Notifies the scheduler.
    pub fn remove_node(&mut self, node_id: usize) {
        self.inner.remove_node(node_id);
        self.scheduler.on_dom_changed();
    }

    /// Remove a node and drop it. Notifies the scheduler.
    pub fn remove_and_drop_node(&mut self, node_id: usize) {
        self.inner.remove_and_drop_node(node_id);
        self.scheduler.on_dom_changed();
    }

    /// Remove all children of a node, dropping them. Notifies the scheduler.
    pub fn remove_all_children(&mut self, parent_id: usize) {
        self.inner.remove_and_drop_all_children(parent_id);
        self.scheduler.on_dom_changed();
    }

    /// Append pre-created children to a parent. Notifies the scheduler.
    pub fn append_children(&mut self, parent_id: usize, child_ids: &[usize]) {
        self.inner.append_children(parent_id, child_ids);
        self.scheduler.on_dom_changed();
    }

    /// Insert nodes before an anchor. Notifies the scheduler.
    pub fn insert_before(&mut self, anchor_id: usize, new_node_ids: &[usize]) {
        self.inner.insert_nodes_before(anchor_id, new_node_ids);
        self.scheduler.on_dom_changed();
    }

    /// Insert nodes after an anchor. Notifies the scheduler.
    pub fn insert_after(&mut self, anchor_id: usize, new_node_ids: &[usize]) {
        self.inner.insert_nodes_after(anchor_id, new_node_ids);
        self.scheduler.on_dom_changed();
    }

    /// Replace an anchor node with one or more new nodes. Notifies the scheduler.
    pub fn replace_with(&mut self, anchor_id: usize, new_node_ids: &[usize]) {
        self.inner.replace_node_with(anchor_id, new_node_ids);
        self.scheduler.on_dom_changed();
    }

    /// Move all children from `old_parent_id` to `new_parent_id`. Notifies the scheduler.
    pub fn reparent_children(&mut self, old_parent_id: usize, new_parent_id: usize) {
        self.inner.reparent_children(old_parent_id, new_parent_id);
        self.scheduler.on_dom_changed();
    }

    /// Parse and set `innerHTML` on a node. Notifies the scheduler.
    pub fn set_inner_html(&mut self, node_id: usize, html: &str) {
        self.inner.set_inner_html(node_id, html);
        self.scheduler.on_dom_changed();
    }

    /// Convenience: set the `id` attribute. Notifies the scheduler.
    pub fn set_id(&mut self, node_id: usize, id: &str) {
        self.inner
            .set_attribute(node_id, qual_name!("id"), id);
        self.scheduler.on_dom_changed();
    }

    /// Convenience: set the `class` attribute. Notifies the scheduler.
    pub fn set_class(&mut self, node_id: usize, class: &str) {
        self.inner
            .set_attribute(node_id, qual_name!("class"), class);
        self.scheduler.on_dom_changed();
    }
}

// NOTE: A `DirectMutError` enum was originally planned here for
// future strict-mode validation, but every underlying `DocumentMutator`
// method is currently infallible (it silently no-ops on type mismatches).
// The enum was removed to avoid dead-code warnings; re-introduce it
// here when strict-mode validation lands.
