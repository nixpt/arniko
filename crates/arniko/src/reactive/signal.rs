use std::sync::{Arc, RwLock};

use super::computed::Computed;

struct SignalInner<T> {
    value: T,
    version: u64,
}

/// Shared interface for anything that can be read reactively — both `Signal<T>` and `Computed<T>`.
/// Implementors must be clone-to-share (`Clone`), thread-safe (`Send + Sync`), and `'static`.
pub trait Reactive<T: Clone + 'static>: Clone + Send + Sync + 'static {
    fn get_value(&self) -> T;
    /// Monotonic counter; increments whenever the value changes.
    fn reactive_version(&self) -> u64;
}

/// Reactive state container. Clone-to-share; `set` notifies the `Reactor` on next flush.
pub struct Signal<T: Clone + 'static> {
    inner: Arc<RwLock<SignalInner<T>>>,
}

impl<T: Clone + 'static> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T: Clone + Send + Sync + 'static> Reactive<T> for Signal<T> {
    fn get_value(&self) -> T {
        self.get()
    }
    fn reactive_version(&self) -> u64 {
        self.version()
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

impl<T: Clone + Send + Sync + 'static> Signal<T> {
    /// Derive a `Computed<U>` from this signal. The function runs lazily — only when the
    /// computed value is actually read and the signal version has changed since last compute.
    pub fn derive<U>(&self, f: impl Fn(T) -> U + Send + Sync + 'static) -> Computed<U>
    where
        U: Clone + Send + Sync + 'static,
    {
        Computed::from_signal(self.clone(), f)
    }
}
