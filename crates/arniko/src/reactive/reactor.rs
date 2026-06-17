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
        // Safe: we only call this with handles that were created from this reactor
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
            let mut child_reactor = Reactor::new();
            let view = (self.template)(&list[i]);
            let (node_id, _scope) = view.mount(mutator, &mut child_reactor, self.container_id);
            // The scope is for the child reactor, which we store in children.
            // When the child is removed, the child_reactor is dropped, which drops the scope.
            self.children.push(ItemState {
                node_id,
                reactor: child_reactor,
            });
            any_dirty = true;
        }

        any_dirty
    }
}

/// A mounted list item tracked by `ForBinding`.
///
/// Each item owns a dedicated child `Reactor` so nested `ReactiveText` /
/// `Computed` views inside the item keep updating across list reconciliations.
pub(crate) struct ItemState {
    pub node_id: usize,
    pub reactor: Reactor,
}

/// Tracks reactive→DOM patch bindings. Call `flush` after mutating signals to apply patches.
pub struct Reactor {
    bindings: Vec<Option<Box<dyn Binding>>>,
    next_handle: usize,
}

impl Reactor {
    pub fn new() -> Self {
        Reactor {
            bindings: Vec::new(),
            next_handle: 0,
        }
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
