use std::sync::{Arc, RwLock};

struct SignalInner<T> {
    value: T,
    version: u64,
}

/// Reactive state container. Clone-to-share; `set` notifies the `Reactor` on next flush.
pub struct Signal<T: Clone + 'static> {
    inner: Arc<RwLock<SignalInner<T>>>,
}

impl<T: Clone + 'static> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal { inner: Arc::clone(&self.inner) }
    }
}

impl<T: Clone + 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        Signal {
            inner: Arc::new(RwLock::new(SignalInner { value, version: 0 })),
        }
    }

    pub fn get(&self) -> T {
        self.inner.read().unwrap().value.clone()
    }

    pub fn set(&self, value: T) {
        let mut inner = self.inner.write().unwrap();
        inner.value = value;
        inner.version += 1;
    }

    pub fn update(&self, f: impl FnOnce(&T) -> T) {
        let mut inner = self.inner.write().unwrap();
        let new_val = f(&inner.value);
        inner.value = new_val;
        inner.version += 1;
    }

    pub(super) fn version(&self) -> u64 {
        self.inner.read().unwrap().version
    }
}
