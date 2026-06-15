use std::marker::PhantomData;
use std::sync::Arc;

use bliss_dom::DocumentMutator;

use super::signal::Reactive;
use super::view::View;

trait Binding {
    fn flush(&mut self, mutator: &mut DocumentMutator);
}

struct ReactiveBinding<T: Clone + 'static, R: Reactive<T>> {
    source: R,
    last_version: u64,
    patch: Box<dyn Fn(&mut DocumentMutator, &T)>,
    _marker: PhantomData<T>,
}

impl<T: Clone + 'static, R: Reactive<T>> Binding for ReactiveBinding<T, R> {
    fn flush(&mut self, mutator: &mut DocumentMutator) {
        let version = self.source.reactive_version();
        if version != self.last_version {
            self.last_version = version;
            let value = self.source.get_value();
            (self.patch)(mutator, &value);
        }
    }
}

/// Reconciles a `Reactive<Vec<T>>` against a container node by clearing all mounted children
/// and re-mounting fresh ones whenever the list version advances.
///
/// v1 limitation: item views remounted during reconciliation are given a stub `Reactor`, so
/// nested `ReactiveText` / `Computed` inside items only work on the initial render. Items
/// containing reactive views should use the initial mount; static item templates work fully.
struct ForBinding<T, R>
where
    T: Clone + Send + Sync + 'static,
    R: Reactive<Vec<T>>,
{
    source: R,
    last_version: u64,
    template: Arc<dyn Fn(&T) -> Box<dyn View> + Send + Sync>,
    container_id: usize,
    mounted_ids: Vec<usize>,
    _marker: PhantomData<T>,
}

impl<T, R> Binding for ForBinding<T, R>
where
    T: Clone + Send + Sync + 'static,
    R: Reactive<Vec<T>>,
{
    fn flush(&mut self, mutator: &mut DocumentMutator) {
        let version = self.source.reactive_version();
        if version == self.last_version {
            return;
        }
        self.last_version = version;
        let list = self.source.get_value();

        // Clear existing children.
        for id in self.mounted_ids.drain(..) {
            mutator.remove_node(id);
        }

        // Re-mount fresh children. A stub Reactor is used so nested reactive views
        // don't register in the main reactor (v1 limitation).
        let mut stub = Reactor::new();
        for item in &list {
            let view = (self.template)(item);
            let id = view.mount(mutator, &mut stub, self.container_id);
            self.mounted_ids.push(id);
        }
    }
}

/// Tracks reactive→DOM patch bindings. Call `flush` after mutating signals to apply patches.
pub struct Reactor {
    bindings: Vec<Box<dyn Binding>>,
}

impl Reactor {
    pub fn new() -> Self {
        Reactor { bindings: Vec::new() }
    }

    /// Bind any `Reactive<T>` (a `Signal` or `Computed`) to a DOM patch function.
    /// The patch fires on `flush` whenever the source's version has advanced.
    pub fn bind<T: Clone + 'static, R: Reactive<T>>(
        &mut self,
        source: R,
        patch: impl Fn(&mut DocumentMutator, &T) + 'static,
    ) {
        let last_version = source.reactive_version();
        self.bindings.push(Box::new(ReactiveBinding {
            source,
            last_version,
            patch: Box::new(patch),
            _marker: PhantomData,
        }));
    }

    /// Register a list reconciliation binding for a `For` view.
    /// Called from `For::mount` after the initial items are mounted.
    pub(super) fn bind_for<T, R>(
        &mut self,
        source: R,
        template: Arc<dyn Fn(&T) -> Box<dyn View> + Send + Sync>,
        container_id: usize,
        initial_ids: Vec<usize>,
    )
    where
        T: Clone + Send + Sync + 'static,
        R: Reactive<Vec<T>>,
    {
        let last_version = source.reactive_version();
        self.bindings.push(Box::new(ForBinding {
            source,
            last_version,
            template,
            container_id,
            mounted_ids: initial_ids,
            _marker: PhantomData,
        }));
    }

    /// Apply all dirty patches to the document.
    pub fn flush(&mut self, mutator: &mut DocumentMutator) {
        for binding in &mut self.bindings {
            binding.flush(mutator);
        }
    }
}

impl Default for Reactor {
    fn default() -> Self {
        Self::new()
    }
}
