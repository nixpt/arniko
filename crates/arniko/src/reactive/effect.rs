//! `create_effect` — explicit-dep side-effect binding on a reactive source.
//!
//! Like [`super::Computed`] but evaluates a side-effect closure on every
//! version change instead of producing a value. Use for logging, telemetry,
//! side effects that need to run on every change, etc.
//!
//! # Model-fitting limitation (D3)
//!
//! Deps are declared via the `source` parameter (like [`Computed::from_signal`]
//! / [`Computed::from2`] / [`Computed::from3`]). The push-subscriber-graph form
//! of auto-dep-tracking that Leptos and Solid provide is OUT OF SCOPE for this
//! iteration (per the codebase-wide `D3` decision in `.dejavue/decisions.md`).
//! To read multiple sources in one effect, compose them with
//! [`Computed::from2`] / [`Computed::from3`] and pass the resulting
//! `Computed` as the source — its dependencies are tracked via the existing
//! pre-declared mechanism.
//!
//! # Example
//!
//! ```ignore
//! use arniko::reactive::{Signal, create_effect};
//!
//! let counter = Signal::new(0u32);
//! let mut reactor = Reactor::new();
//! let _scope = create_effect(&mut reactor, counter.clone(), |n| {
//!     println!("counter changed to {}", n);
//! });
//! counter.set(5); // prints "counter changed to 5" on next flush
//! ```

use super::reactor::Reactor;
use super::signal::Reactive;

/// Create an effect that fires on every change of `source`. The closure receives
/// the new value `&T`. Returns a [`super::Scope`] — drop it to deregister the
/// effect.
///
/// Deps are pre-declared via the `source` parameter. See the module rustdoc
/// for the auto-dep-tracking limitation that is out of scope for this D3
/// iteration.
pub fn create_effect<T, R, F>(reactor: &mut Reactor, source: R, effect: F) -> super::reactor::Scope
where
    T: Clone + 'static,
    R: Reactive<T>,
    F: Fn(&T) + Send + Sync + 'static,
{
    reactor.bind_scoped(source, move |_mutator, new_value| {
        effect(new_value);
    })
}
