use std::marker::PhantomData;

use bliss_dom::DocumentMutator;

use super::signal::Reactive;

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
