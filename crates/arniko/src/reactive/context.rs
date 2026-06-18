//! Reactive context — typed provide / inject keyed by `TypeId`, scoped to a
//! [`Reactor`].
//!
//! Provides the canonical Vue / Leptos-style dependency-injection pattern for
//! arniko. Single flat `HashMap<TypeId, Arc<T>>` keyed by `TypeId::of::<T>()`
//! inside the Reactor. Values are not nested-scope-aware — out of scope per
//! the `D3` decision in `.dejavue/decisions.md`.
//!
//! # When to use
//!
//! Use `provide` / `inject` for:
//! - Configuration passed down through the view tree (theme, user locale, debug mode)
//! - Service objects that any descendant view might need (logger, telemetry client)
//! - State that should bypass the dependency-tracking runtime
//!
//! Use a `Signal` instead when the value changes over time and views need to
//! re-render in response. Provide/inject is for relatively stable values.
//!
//! # Example
//!
//! ```ignore
//! use std::sync::Arc;
//! use arniko::reactive::{Reactor, config_reactive};
//!
//! #[derive(Clone, Debug)]
//! struct AppConfig { pub debug: bool, pub max_items: usize }
//!
//! // At the top of the launch setup:
//! reactor.provide(AppConfig { debug: true, max_items: 100 });
//!
//! // In a descendant View's mount closure:
//! if let Some(cfg) = reactor.inject::<AppConfig>() {
//!     // cfg.debug is true here
//! }
//! ```

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use super::reactor::Reactor;

pub(super) type ContextValue = Box<dyn Any + Send + Sync>;

pub(super) struct Context {
    map: HashMap<TypeId, ContextValue>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            map: HashMap::new(),
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Reactor {
    /// Provide a value of type `T` keyed by `TypeId::of::<T>()`. Subsequent
    /// calls with the same type REPLACE the prior value (new-value-wins).
    /// The value is wrapped in an `Arc<T>` and shared among all
    /// `inject::<T>()` readers on this Reactor.
    ///
    /// # Example
    ///
    /// ```ignore
    /// reactor.provide(AppConfig { debug: true });
    /// ```
    pub fn provide<T: Send + Sync + 'static>(&mut self, value: T) {
        self.context
            .map
            .insert(TypeId::of::<T>(), Box::new(Arc::new(value)));
    }

    /// Inject the value of type `T` previously provided (if any). Returns
    /// `None` when no value of that type was provided on this Reactor.
    ///
    /// Returns `Arc<T>` so multiple readers can share the value without
    /// re-providing — typical for shared services / config.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(cfg) = reactor.inject::<AppConfig>() {
    ///     // cfg is Arc<AppConfig> — dereference or use methods directly
    ///     println!("debug mode: {}", cfg.debug);
    /// }
    /// ```
    pub fn inject<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.context
            .map
            .get(&TypeId::of::<T>())
            .and_then(|b| b.downcast_ref::<Arc<T>>())
            .map(Arc::clone)
    }

    /// Drop any stored value of type `T`. Returns `true` if a value was found
    /// and removed, `false` if no value of that type was stored.
    pub fn revoke<T: Send + Sync + 'static>(&mut self) -> bool {
        self.context.map.remove(&TypeId::of::<T>()).is_some()
    }

    /// Number of distinct types stored in the context. Mostly used by tests.
    pub fn context_len(&self) -> usize {
        self.context.map.len()
    }
}
