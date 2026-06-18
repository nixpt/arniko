//! `Resource<T>` — synchronous state-machine facade for async-driven data.
//!
//! This is **NOT** an `async` primitive. arniko has no async runtime.
//! `Resource` is a state-machine facade over a [`Reactive<ResourceState<T>>`]
//! so external async drivers (a thread, a `tokio::spawn`, a timer) can call
//! `.resolve(t)` / `.reject(e)` from outside the reactive runtime and the
//! resulting state-prop transition propagates through the existing
//! `Reactor::flush` machinery naturally.
//!
//! # Model-fitting limitation (D3)
//!
//! `create_resource` does NOT auto-fetch a value. Callers wire their own
//! driver (a thread, a timer, a `tokio::spawn` task) that calls
//! `.resolve(t)` / `.reject(e)` from outside. See the example below for
//! the canonical "spawn thread → call `.resolve`" idiom.
//!
//! To drive the resource from the launch path, the calling code typically:
//!   1. Creates the resource in `Pending` (`create_resource()`).
//!   2. Spawns a thread (or timer) that calls `.resolve(t)` once data arrives.
//!   3. Mounts a view that reads the resource.
//!
//! # Example
//!
//! ```ignore
//! use arniko::reactive::{ResourceState, Resource, create_resource, Reactive, ReactiveText};
//!
//! let user: Resource<String> = create_resource();
//!
//! // External thread driver:
//! std::thread::spawn({
//!     let user = user.clone();
//!     move || {
//!         // ... fetch from network ...
//!         user.resolve("Alice".to_string());
//!     }
//! });
//!
//! // View in the reactive tree:
//! let view = ReactiveText::new(user.clone());
//! // Initial state is Pending; user.resolve fires on flush → text updates.
//! ```

use std::sync::Arc;

use parking_lot::RwLock;

use super::signal::Reactive;


/// State of a `Resource<T>`.
///
/// Pattern-matched by views that want to render different content per state.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceState<T> {
    /// Initial / waiting state — no value yet, no error yet.
    Pending,
    /// Resource has resolved with `T`.
    Resolved(T),
    /// Resource has rejected with the given error message.
    Error(String),
}

impl<T> ResourceState<T> {
    /// Returns true if state is `Pending`.
    pub fn is_pending(&self) -> bool {
        matches!(self, ResourceState::Pending)
    }

    /// Returns true if state is `Resolved`.
    pub fn is_resolved(&self) -> bool {
        matches!(self, ResourceState::Resolved(_))
    }

    /// Returns true if state is `Error`.
    pub fn is_error(&self) -> bool {
        matches!(self, ResourceState::Error(_))
    }

    /// Get the resolved value if `Resolved`, else `None`.
    pub fn value(&self) -> Option<&T> {
        match self {
            ResourceState::Resolved(t) => Some(t),
            _ => None,
        }
    }
}

/// Synchronous async-style resource. Clone-to-share (Arc-backed). State
/// transitions are `.resolve(t)` / `.reject(e)` from any thread / timer /
/// caller-supplied async driver.
///
/// `Resource<T>` implements [`Reactive<ResourceState<T>>`] so it slots into
/// every existing reactive surface: `ReactiveText::new(resource)`,
/// `Computed::from2`, `Switch::new(resource, …)`, `create_effect`, etc.
pub struct Resource<T: Clone + 'static> {
    inner: Arc<RwLock<ResourceInner<T>>>,
}

struct ResourceInner<T> {
    state: ResourceState<T>,
    version: u64,
}

impl<T: Clone + 'static> Clone for Resource<T> {
    fn clone(&self) -> Self {
        Resource { inner: Arc::clone(&self.inner) }
    }
}

impl<T: Clone + Send + Sync + 'static> Resource<T> {
    /// Construct a fresh resource in `Pending` state. Use this when you intend
    /// to drive it from an external runtime (thread, timer, async task).
    pub fn new() -> Self {
        Resource {
            inner: Arc::new(RwLock::new(ResourceInner {
                state: ResourceState::Pending,
                version: 0,
            })),
        }
    }

    /// Construct a resource pre-resolved to `value`. Mostly useful for tests,
    /// mocks, or stubs that don't need a fetch lifecycle.
    pub fn resolved(value: T) -> Self {
        Resource {
            inner: Arc::new(RwLock::new(ResourceInner {
                state: ResourceState::Resolved(value),
                version: 0,
            })),
        }
    }

    /// Transition to `Resolved(value)`. Bumps the version so subscribers see
    /// the change on next flush.
    pub fn resolve(&self, value: T) {
        let mut inner = self.inner.write();
        inner.state = ResourceState::Resolved(value);
        inner.version += 1;
    }

    /// Transition to `Error(message)`. Bumps the version.
    pub fn reject(&self, message: impl Into<String>) {
        let mut inner = self.inner.write();
        inner.state = ResourceState::Error(message.into());
        inner.version += 1;
    }

    /// Reset to `Pending` (e.g. for re-fetching). Bumps the version so any
    /// stale `Resolved`/`Error` views flush out.
    pub fn invalidate(&self) {
        let mut inner = self.inner.write();
        inner.state = ResourceState::Pending;
        inner.version += 1;
    }

    /// Read the current state.
    pub fn get(&self) -> ResourceState<T> {
        self.inner.read().state.clone()
    }
}

impl<T: Clone + Send + Sync + 'static> Default for Resource<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Send + Sync + 'static> Reactive<ResourceState<T>> for Resource<T> {
    fn get_value(&self) -> ResourceState<T> {
        self.get()
    }
    fn reactive_version(&self) -> u64 {
        self.inner.read().version
    }
}

/// Sugar for [`Resource::new`] — semantic equivalent of Leptos
/// `create_resource` without the auto-fetch. Callers wire their own driver
/// and call `.resolve(t)` / `.reject(e)` from outside the reactive runtime.
pub fn create_resource<T: Clone + Send + Sync + 'static>() -> Resource<T> {
    Resource::new()
}
