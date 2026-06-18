use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;

use bliss_dom::DocumentMutator;

use super::signal::Reactive;
use super::view::View;

/// A handle to a binding that can be used to remove it from the reactor.
/// This enables binding lifecycle management — when a view is unmounted,
/// its bindings can be removed to prevent memory leaks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BindingHandle(usize);

/// A RAII scope that automatically removes its bindings when dropped.
/// Returned by `Reactor::bind_scoped` and `Reactor::bind_for_scoped` to
/// track binding lifecycle.
pub struct Scope {
    pub(crate) reactor: *mut Reactor,
    pub(crate) handles: Vec<BindingHandle>,
}

impl Scope {
    /// Manually unmount this scope's bindings without waiting for Drop.
    pub fn unmount(self) {
        // Safe: we only call this with handles that were created from this reactor
        unsafe {
            let reactor = &mut *self.reactor;
            // Use ManuallyDrop to take ownership of handles
            let mut this = std::mem::ManuallyDrop::new(self);
            let handles = std::mem::take(&mut this.handles);
            for handle in handles {
                reactor.remove_binding(handle);
            }
        }
    }

    /// Merge another scope's handles into this one.
    /// The other scope is consumed and should not be used after this call.
    pub fn merge(&mut self, other: Scope) {
        // Both scopes must point to the same reactor
        debug_assert_eq!(self.reactor, other.reactor);
        // Use ManuallyDrop to prevent other from being dropped
        let mut other = std::mem::ManuallyDrop::new(other);
        // Now we can safely take the handles
        self.handles.extend(std::mem::take(&mut other.handles));
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        // Default RAII cleanup: every binding registered via `bind_*` on
        // this scope is removed from the reactor. The caller is expected
        // to retain the `Scope` if they want reactivity to persist past
        // the binding site's lexical scope.
        //
        // For the common pattern where `view.mount(...)` is invoked as a
        // statement and the returned scope would be silently discarded at
        // the semicolon, callers should use [`Reactor::park_scope`] (e.g.
        // via a helper) — see `crates/arniko/tests/reactive_components.rs`
        // for the `mount_parked` helper used by the component integration
        // tests.
        //
        // SAFETY: `self.reactor` is a raw pointer captured at scope-creation
        // time as `reactor as *mut _` from inside one of the `bind_*`
        // methods. The contract is that the target reactor is **never moved
        // or reallocated** after the binding is registered — i.e. its
        // address is stable for the lifetime of this scope. Currently no
        // public API moves a Reactor, and `ItemState.reactor: Box<Reactor>`
        // (in this file) ensures child (parked-scope) reactors likewise
        // have stable heap-buffer addresses. If a future API breaks this
        // invariant, this drop will dereference freed memory and SIGSEGV.
        unsafe {
            let reactor = &mut *self.reactor;
            for handle in self.handles.drain(..) {
                reactor.remove_binding(handle);
            }
        }
    }
}

trait Binding {
    fn flush(&mut self, mutator: &mut DocumentMutator) -> bool;
}

struct ReactiveBinding<T: Clone + 'static, R: Reactive<T>> {
    source: R,
    last_version: u64,
    patch: Box<dyn Fn(&mut DocumentMutator, &T)>,
    _marker: PhantomData<T>,
}

impl<T: Clone + 'static, R: Reactive<T>> Binding for ReactiveBinding<T, R> {
    fn flush(&mut self, mutator: &mut DocumentMutator) -> bool {
        let version = self.source.reactive_version();
        if version != self.last_version {
            self.last_version = version;
            let value = self.source.get_value();
            (self.patch)(mutator, &value);
            true
        } else {
            false
        }
    }
}

/// Reconciles a `Reactive<Vec<T>>` against a container node using positional
/// diffing: items at the same index survive (their DOM nodes and child reactors
/// are preserved, so nested `ReactiveText` / `Computed` inside items keep
/// updating); extra trailing items are dropped via `remove_and_drop_node`;
/// new items are mounted with dedicated child reactors.
///
/// This replaces the v1 clear-and-remount strategy that leaked DOM nodes
/// (arena growth on every list change) and broke nested reactivity after
/// the first reconciliation.
struct ForBinding<T, R>
where
    T: Clone + Send + Sync + 'static,
    R: Reactive<Vec<T>>,
{
    source: R,
    last_version: u64,
    template: Arc<dyn Fn(&T) -> Box<dyn View> + Send + Sync>,
    container_id: usize,
    children: Vec<ItemState>,
    _marker: PhantomData<T>,
}

impl<T, R> Binding for ForBinding<T, R>
where
    T: Clone + Send + Sync + 'static,
    R: Reactive<Vec<T>>,
{
    fn flush(&mut self, mutator: &mut DocumentMutator) -> bool {
        // Always flush child reactors so nested reactive views update
        // regardless of whether the list itself changed.
        let mut any_dirty = false;
        for child in &mut self.children {
            if child.reactor.flush(mutator, None) {
                any_dirty = true;
            }
        }

        let version = self.source.reactive_version();
        if version == self.last_version {
            return any_dirty;
        }
        self.last_version = version;
        let list = self.source.get_value();

        let old_len = self.children.len();
        let new_len = list.len();

        // Drop trailing items that are no longer in the list.
        for i in (new_len..old_len).rev() {
            let child = self.children.remove(i);
            mutator.remove_and_drop_node(child.node_id);
            // child.reactor drops — its bindings are disposed.
            any_dirty = true;
        }

        // Mount new items at the end.
        for i in old_len..new_len {
            // Heap-allocate the child reactor so its address is stable
            // across the `self.children.push(...)` move below. Parked
            // scopes hold raw pointers to this reactor's memory.
            let mut child_reactor = Box::new(Reactor::new());
            let view = (self.template)(&list[i]);
            let (node_id, child_scope) =
                view.mount(mutator, &mut *child_reactor, self.container_id);
            // Park the child's scope on its own reactor so the bindings it
            // registered survive past this statement. The child_reactor (and
            // thus these bindings) is dropped when the item is removed during
            // a future reconciliation.
            child_reactor.park_scope(child_scope);
            self.children.push(ItemState {
                node_id,
                reactor: child_reactor,
            });
            any_dirty = true;
        }

        any_dirty
    }
}

/// Reconciles a `Reactive<Vec<T>>` against a container node using **keyed**
/// diffing: items with the same key (extracted via `key_fn`) survive across
/// reorders; new keys mount fresh DOM nodes + child reactors; missing keys
/// drop via `remove_and_drop_node`. DOM nodes are repositioned via
/// `mutator.insert_nodes_before(...)` so the painted sibling order matches
/// the new key order.
///
/// Versus [`ForBinding`]'s positional diff, keyed diff preserves identity
/// across reorders — surviving items keep their internal reactive bindings
/// + child reactor state. See `super::keyed::KeyedFor` for the View surface.
struct KeyedForBinding<T, K, R>
where
    T: Clone + Send + Sync + 'static,
    K: Clone + Hash + Eq + Send + Sync + 'static,
    R: Reactive<Vec<T>>,
{
    source: R,
    last_version: u64,
    key_fn: Arc<dyn Fn(&T) -> K + Send + Sync>,
    template: Arc<dyn Fn(&T) -> Box<dyn View> + Send + Sync>,
    container_id: usize,
    items: HashMap<K, ItemState>,
    /// New-key-order from the most recent flush. Used to detect which items
    /// have moved (reorders) and to drive reposition via DOM insert_before.
    order: Vec<K>,
    _marker: PhantomData<T>,
}

impl<T, K, R> Binding for KeyedForBinding<T, K, R>
where
    T: Clone + Send + Sync + 'static,
    K: Clone + Hash + Eq + Send + Sync + 'static,
    R: Reactive<Vec<T>>,
{
    fn flush(&mut self, mutator: &mut DocumentMutator) -> bool {
        // Always flush child reactors first so nested reactive views update
        // regardless of whether the list itself changed.
        let mut any_dirty = false;
        for ki in self.items.values_mut() {
            if ki.reactor.flush(mutator, None) {
                any_dirty = true;
            }
        }

        let version = self.source.reactive_version();
        if version == self.last_version {
            return any_dirty;
        }
        self.last_version = version;
        let new_list = self.source.get_value();
        let new_keys: Vec<K> = new_list.iter().map(|t| (self.key_fn)(t)).collect();

        // Phase 1: Mount any new keys, append at end of container.
        for (i, item) in new_list.iter().enumerate() {
            let key = new_keys[i].clone();
            if !self.items.contains_key(&key) {
                // Heap-allocate so the address stays stable: parked scopes
                // hold raw pointers to the reactor's memory and outlive the
                // local scope below.
                let mut child_reactor = Box::new(Reactor::new());
                let view = (self.template)(item);
                let (node_id, child_scope) =
                    view.mount(mutator, &mut *child_reactor, self.container_id);
                child_reactor.park_scope(child_scope);
                self.items.insert(
                    key.clone(),
                    ItemState {
                        node_id,
                        reactor: child_reactor,
                    },
                );
                // Note: view.mount already appends `node_id` to `self.container_id`.
                // Do NOT call `mutator.append_children` here — that would append
                // a second time and produce a duplicate DOM child, breaking the
                // `assert_eq!(ids_after.len(), expected)` invariants on add.
                any_dirty = true;
            }
        }

        // Phase 2: Drop keys no longer in the new list, remove their DOM nodes.
        let new_key_set: HashSet<K> = new_keys.iter().cloned().collect();
        let removed: Vec<K> = self
            .items
            .keys()
            .filter(|k| !new_key_set.contains(*k))
            .cloned()
            .collect();
        for key in removed {
            if let Some(state) = self.items.remove(&key) {
                mutator.remove_and_drop_node(state.node_id);
                any_dirty = true;
            }
        }

        // Phase 3: Reposition — after Phase 1+2, DOM children of the container
        // are exactly the keys in new_keys but possibly in old `self.order`.
        // Walk new_keys and reposition any item whose current DOM position
        // differs from its new index.
        for (i, key) in new_keys.iter().enumerate() {
            let target = self.items[key].node_id;
            let dom_children = mutator.child_ids(self.container_id);
            if i >= dom_children.len() {
                // Past the end — append.
                mutator.append_children(self.container_id, &[target]);
                any_dirty = true;
            } else if dom_children[i] != target {
                // Defensive two-step: remove_node + re-insert. Universally safe
                // regardless of whether insert_nodes_before reparents existing nodes.
                mutator.remove_node(target);
                let fresh = mutator.child_ids(self.container_id);
                if i < fresh.len() {
                    let new_anchor = fresh[i];
                    mutator.insert_nodes_before(new_anchor, &[target]);
                } else {
                    mutator.append_children(self.container_id, &[target]);
                }
                any_dirty = true;
            }
        }

        // Update order to new_keys for next reconciliation.
        self.order = new_keys;
        any_dirty
    }
}

/// A mounted list item tracked by `ForBinding` and `KeyedForBinding`.
///
/// Each item owns a dedicated child `Reactor` so nested `ReactiveText` /
/// `Computed` views inside the item keep updating across list reconciliations.
///
/// `reactor` is held inside a `Box<Reactor>` so the **heap buffer address** of
/// the `Reactor` value stays stable across moves of the enclosing `ItemState`,
/// the `ForBinding::children` `Vec`, or the `KeyedForBinding::items` `HashMap`.
/// (Moving the `Box` itself only moves the heap **pointer**; the buffer it
/// points to stays put at one address.) Children views park scopes whose
/// `*mut Reactor` field references that buffer — inline storage would let
/// the `Vec` reallocations and `HashMap` rehashing relocate the buffer and
/// silently invalidate every parked-scope raw pointer, producing
/// use-after-free SIGSEGVs during process cleanup.
pub(crate) struct ItemState {
    pub node_id: usize,
    pub reactor: Box<Reactor>,
}

/// Tracks reactive→DOM patch bindings. Call `flush` after mutating signals to apply patches.
pub struct Reactor {
    bindings: Vec<Option<Box<dyn Binding>>>,
    next_handle: usize,
    /// Scopes re-homed here so their bindings outlive the statement that created them.
    ///
    /// This is needed for scopes created **inside patch closures** (Show/Switch
    /// re-mounts, `For` reconciliation) that return `()` — there is no caller to
    /// hold the scope, so without parking it would `Drop` immediately and remove
    /// the just-registered binding before the next flush. Parked scopes (and their
    /// bindings) live until the reactor is dropped or a parent scope disposes them.
    parked_scopes: Vec<Scope>,
    /// Typed provider/inject context. See `super::context` — single flat
    /// `HashMap<TypeId, Arc<T>>` keyed by `TypeId::of::<T>()`, new-value-wins
    /// on `provide`, `Arc::clone` on `inject`. Out of scope per D3: nested
    /// scope semantics, intrusive dep tracking.
    pub(super) context: super::context::Context,
}

impl Reactor {
    pub fn new() -> Self {
        Reactor {
            bindings: Vec::new(),
            next_handle: 0,
            parked_scopes: Vec::new(),
            context: super::context::Context::new(),
        }
    }

    /// Park a `Scope` on this reactor so it (and the bindings it tracks) is kept
    /// alive until the reactor is dropped. The scope's `reactor` pointer must
    /// point at this reactor.
    ///
    /// Use this for scopes created inside patch closures (`Fn(...)`) where there
    /// is nowhere else to hold them — e.g. when `Show`/`Switch` re-mount a child
    /// view, or when `For` appends items during reconciliation.
    pub fn park_scope(&mut self, scope: Scope) {
        debug_assert_eq!(
            scope.reactor, self as *mut _,
            "park_scope: scope must belong to this reactor"
        );
        self.parked_scopes.push(scope);
    }

    /// Bind any `Reactive<T>` (a `Signal` or `Computed`) to a DOM patch function.
    /// The patch fires on `flush` whenever the source's version has advanced.
    ///
    /// Note: This method does not return a handle for cleanup. For views that need
    /// lifecycle management, use `bind_scoped` instead.
    pub fn bind<T: Clone + 'static, R: Reactive<T>>(
        &mut self,
        source: R,
        patch: impl Fn(&mut DocumentMutator, &T) + 'static,
    ) {
        let last_version = source.reactive_version();
        let _handle = self.next_handle;
        self.next_handle += 1;
        self.bindings.push(Some(Box::new(ReactiveBinding {
            source,
            last_version,
            patch: Box::new(patch),
            _marker: PhantomData,
        })));
    }

    /// Bind with lifecycle tracking. Returns a `Scope` that, when dropped,
    /// automatically removes the binding from the reactor.
    ///
    /// Use this for views that may be unmounted before the reactor is dropped.
    pub fn bind_scoped<T: Clone + 'static, R: Reactive<T>>(
        &mut self,
        source: R,
        patch: impl Fn(&mut DocumentMutator, &T) + 'static,
    ) -> Scope {
        let last_version = source.reactive_version();
        let handle = BindingHandle(self.next_handle);
        self.next_handle += 1;
        self.bindings.push(Some(Box::new(ReactiveBinding {
            source,
            last_version,
            patch: Box::new(patch),
            _marker: PhantomData,
        })));
        Scope {
            reactor: self as *mut _,
            handles: vec![handle],
        }
    }

    /// Remove a binding by its handle. Called automatically by `Scope::drop`.
    pub fn remove_binding(&mut self, handle: BindingHandle) {
        let idx = handle.0;
        if idx < self.bindings.len() {
            self.bindings[idx] = None;
        }
    }

    /// Clean up all None slots in the bindings vector to prevent unbounded growth.
    /// Called periodically to compact the storage.
    pub fn cleanup_bindings(&mut self) {
        self.bindings.retain(|b| b.is_some());
    }

    /// Get the current number of active bindings (for testing).
    pub fn binding_count(&self) -> usize {
        self.bindings.iter().filter(|b| b.is_some()).count()
    }

    /// Register a list reconciliation binding for a `For` view.
    /// Called from `For::mount` after the initial items are mounted.
    /// Returns a Scope that will remove this binding when dropped.
    pub(super) fn bind_for<T, R>(
        &mut self,
        source: R,
        template: Arc<dyn Fn(&T) -> Box<dyn View> + Send + Sync>,
        container_id: usize,
        children: Vec<ItemState>,
    ) -> Scope
    where
        T: Clone + Send + Sync + 'static,
        R: Reactive<Vec<T>>,
    {
        let last_version = source.reactive_version();
        let handle = BindingHandle(self.next_handle);
        self.next_handle += 1;
        self.bindings.push(Some(Box::new(ForBinding {
            source,
            last_version,
            template,
            container_id,
            children,
            _marker: PhantomData,
        })));
        Scope {
            reactor: self as *mut _,
            handles: vec![handle],
        }
    }

    /// Register a keyed list reconciliation binding for a `KeyedFor` view.
    /// Called from `KeyedFor::mount` after the initial items are mounted.
    /// Returns a Scope that will remove this binding when dropped.
    pub(super) fn bind_keyed_for<T, K, R>(
        &mut self,
        source: R,
        key_fn: Arc<dyn Fn(&T) -> K + Send + Sync>,
        template: Arc<dyn Fn(&T) -> Box<dyn View> + Send + Sync>,
        container_id: usize,
        initial_items: HashMap<K, ItemState>,
        initial_order: Vec<K>,
    ) -> Scope
    where
        T: Clone + Send + Sync + 'static,
        K: Clone + Hash + Eq + Send + Sync + 'static,
        R: Reactive<Vec<T>>,
    {
        let last_version = source.reactive_version();
        let handle = BindingHandle(self.next_handle);
        self.next_handle += 1;
        self.bindings.push(Some(Box::new(KeyedForBinding {
            source,
            last_version,
            key_fn,
            template,
            container_id,
            items: initial_items,
            order: initial_order,
            _marker: PhantomData,
        })));
        Scope {
            reactor: self as *mut _,
            handles: vec![handle],
        }
    }

    /// Apply all dirty patches to the document.
    ///
    /// Returns `true` if any binding produced a dirty patch.
    ///
    /// If a `SceneScheduler` is provided, it is notified once if any
    /// binding produced a dirty patch — this is the "reactive-coordinated
    /// scheduling" hook from Phase 5: signal changes drive both the DOM
    /// patch and the GPU effect re-application.
    pub fn flush(
        &mut self,
        mutator: &mut DocumentMutator,
        scheduler: Option<&crate::mustang::SceneScheduler>,
    ) -> bool {
        let mut any_dirty = false;
        for binding in &mut self.bindings {
            if let Some(binding) = binding {
                if binding.flush(mutator) {
                    any_dirty = true;
                }
            }
        }
        if any_dirty {
            if let Some(s) = scheduler {
                s.on_dom_changed();
            }
        }
        any_dirty
    }
}

impl Default for Reactor {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Reactor {
    fn drop(&mut self) {
        // **Critical drop-order fix.** Rust drops struct fields in *declaration*
        // order; `bindings` is declared *before* `parked_scopes`, so under
        // the default field drop order `bindings` is freed first and then each
        // parked [`Scope`]'s drop callback writes `bindings[idx] = None` —
        // *use-after-free* → SIGSEGV during reactor cleanup in tests that
        // park scopes (e.g. via the `mount_parked` helper).
        //
        // We take ownership of `parked_scopes` FIRST so its `Scope::drop`
        // callbacks land on a still-valid `bindings` vector. The remaining
        // fields (`bindings`, `next_handle`, `context`) then drop normally
        // via auto-derived drop, with `bindings` already cleared.
        let _ = std::mem::take(&mut self.parked_scopes);
    }
}
