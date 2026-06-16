use std::sync::{Arc, Mutex};

use super::signal::{Reactive, Signal};

struct ComputedInner<T> {
    value: T,
    version: u64,
    /// Last-seen dep versions. Compared against fresh versions to decide whether to recompute.
    dep_versions: Vec<u64>,
    get_dep_versions: Box<dyn Fn() -> Vec<u64> + Send + Sync>,
    compute: Box<dyn Fn() -> T + Send + Sync>,
}

impl<T: Clone> ComputedInner<T> {
    /// Recomputes the value if any dep version changed. Returns whether the value was updated.
    fn maybe_recompute(&mut self) -> bool {
        let current = (self.get_dep_versions)();
        if current != self.dep_versions {
            self.dep_versions = current;
            self.value = (self.compute)();
            self.version += 1;
            true
        } else {
            false
        }
    }
}

/// A read-only derived value. Recomputes lazily — only when read and a dependency has changed.
/// Clone-to-share (Arc-backed). Use [`Signal::derive`] for the common single-dep case.
pub struct Computed<T: Clone + 'static> {
    inner: Arc<Mutex<ComputedInner<T>>>,
}

impl<T: Clone + 'static> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Computed {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T: Clone + Send + Sync + 'static> Reactive<T> for Computed<T> {
    fn get_value(&self) -> T {
        self.get()
    }
    fn reactive_version(&self) -> u64 {
        self.version()
    }
}

impl<T: Clone + Send + Sync + 'static> Computed<T> {
    pub fn get(&self) -> T {
        let mut inner = self.inner.lock().unwrap();
        inner.maybe_recompute();
        inner.value.clone()
    }

    pub(super) fn version(&self) -> u64 {
        let mut inner = self.inner.lock().unwrap();
        inner.maybe_recompute();
        inner.version
    }

    /// Derive from a single source signal.
    pub fn from_signal<S>(signal: Signal<S>, f: impl Fn(S) -> T + Send + Sync + 'static) -> Self
    where
        S: Clone + Send + Sync + 'static,
    {
        let f = Arc::new(f);
        let sig_compute = signal.clone();
        let sig_deps = signal.clone();
        let f_compute = Arc::clone(&f);
        let initial = f(signal.get());
        let initial_deps = vec![signal.version()];
        Computed {
            inner: Arc::new(Mutex::new(ComputedInner {
                value: initial,
                version: 0,
                dep_versions: initial_deps,
                get_dep_versions: Box::new(move || vec![sig_deps.version()]),
                compute: Box::new(move || f_compute(sig_compute.get())),
            })),
        }
    }

    /// Derive from two source signals.
    pub fn from2<A, B>(
        a: Signal<A>,
        b: Signal<B>,
        f: impl Fn(A, B) -> T + Send + Sync + 'static,
    ) -> Self
    where
        A: Clone + Send + Sync + 'static,
        B: Clone + Send + Sync + 'static,
    {
        let f = Arc::new(f);
        let a_compute = a.clone();
        let b_compute = b.clone();
        let a_deps = a.clone();
        let b_deps = b.clone();
        let f_compute = Arc::clone(&f);
        let initial = f(a.get(), b.get());
        let initial_deps = vec![a.version(), b.version()];
        Computed {
            inner: Arc::new(Mutex::new(ComputedInner {
                value: initial,
                version: 0,
                dep_versions: initial_deps,
                get_dep_versions: Box::new(move || vec![a_deps.version(), b_deps.version()]),
                compute: Box::new(move || f_compute(a_compute.get(), b_compute.get())),
            })),
        }
    }

    /// Derive from three source signals.
    pub fn from3<A, B, C>(
        a: Signal<A>,
        b: Signal<B>,
        c: Signal<C>,
        f: impl Fn(A, B, C) -> T + Send + Sync + 'static,
    ) -> Self
    where
        A: Clone + Send + Sync + 'static,
        B: Clone + Send + Sync + 'static,
        C: Clone + Send + Sync + 'static,
    {
        let f = Arc::new(f);
        let (ac, bc, cc) = (a.clone(), b.clone(), c.clone());
        let (ad, bd, cd) = (a.clone(), b.clone(), c.clone());
        let f_compute = Arc::clone(&f);
        let initial = f(a.get(), b.get(), c.get());
        let initial_deps = vec![a.version(), b.version(), c.version()];
        Computed {
            inner: Arc::new(Mutex::new(ComputedInner {
                value: initial,
                version: 0,
                dep_versions: initial_deps,
                get_dep_versions: Box::new(move || vec![ad.version(), bd.version(), cd.version()]),
                compute: Box::new(move || f_compute(ac.get(), bc.get(), cc.get())),
            })),
        }
    }

    /// Chain: derive a new `Computed<U>` from this computed value.
    pub fn map<U>(&self, f: impl Fn(T) -> U + Send + Sync + 'static) -> Computed<U>
    where
        U: Clone + Send + Sync + 'static,
    {
        let f = Arc::new(f);
        let source = self.clone();
        let source_deps = self.clone();
        let f_compute = Arc::clone(&f);
        let initial = f(self.get());
        let initial_deps = vec![self.version()];
        Computed {
            inner: Arc::new(Mutex::new(ComputedInner {
                value: initial,
                version: 0,
                dep_versions: initial_deps,
                get_dep_versions: Box::new(move || vec![source_deps.version()]),
                compute: Box::new(move || f_compute(source.get())),
            })),
        }
    }
}
