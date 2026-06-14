use bliss_dom::DocumentMutator;
use super::Signal;

trait Binding {
    fn flush(&mut self, mutator: &mut DocumentMutator);
}

struct SignalBinding<T: Clone + 'static> {
    signal: Signal<T>,
    last_version: u64,
    patch: Box<dyn Fn(&mut DocumentMutator, &T)>,
}

impl<T: Clone + 'static> Binding for SignalBinding<T> {
    fn flush(&mut self, mutator: &mut DocumentMutator) {
        let version = self.signal.version();
        if version != self.last_version {
            self.last_version = version;
            let value = self.signal.get();
            (self.patch)(mutator, &value);
        }
    }
}

/// Tracks signal→DOM patch bindings. Call `flush` after mutating signals to apply patches.
pub struct Reactor {
    bindings: Vec<Box<dyn Binding>>,
}

impl Reactor {
    pub fn new() -> Self {
        Reactor { bindings: Vec::new() }
    }

    /// Bind a signal to a DOM patch. The patch fires on `flush` whenever the signal version changes.
    pub fn bind<T: Clone + 'static>(
        &mut self,
        signal: Signal<T>,
        patch: impl Fn(&mut DocumentMutator, &T) + 'static,
    ) {
        let last_version = signal.version();
        self.bindings.push(Box::new(SignalBinding {
            signal,
            last_version,
            patch: Box::new(patch),
        }));
    }

    /// Apply all dirty signal patches to the document.
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
