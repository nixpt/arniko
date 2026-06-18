//! `batch()` — explicit entry-point combining many reactive writes with a
//! single flush.
//!
//! The arniko model is **poll-on-flush** with version-counters: all writes
//! between flushes are already batched at version-check granularity. This
//! helper names the pattern and guarantees a flush after the writes
//! complete, so callers can read the post-flush DOM state in one place.
//!
//! # Model-fitting (D3)
//!
//! Leptos/Solid `batch()` suppresses subscriber notifications between write
//! boundaries and fires them once at the end. arniko's model already does
//! this implicitly — every `signal.set()` is applied at the next flush, and
//! subscribers are notified exactly once per flush. `batch()` is therefore
//! a documentation entry-point, not a behavioral change.
//!
//! # Example
//!
//! ```ignore
//! use arniko::reactive::batch;
//!
//! batch(reactor, mutator, scheduler, || {
//!     sig_a.set(1);
//!     sig_b.set(2);
//!     sig_c.set(3);
//!     // sig_a/b/c all updated. Reactor flushes once at the end of this closure.
//! });
//! ```

use bliss_dom::DocumentMutator;

use crate::mustang::SceneScheduler;

use super::Reactor;

/// Run `f`, then call `reactor.flush(...)` exactly once. Returns whatever `f`
/// returns. Use this whenever a closure does multiple signal writes and
/// wants a single flush + DOM patch cycle.
pub fn batch<O, F: FnOnce() -> O>(
    reactor: &mut Reactor,
    mutator: &mut DocumentMutator,
    scheduler: Option<&SceneScheduler>,
    f: F,
) -> O {
    let result = f();
    reactor.flush(mutator, scheduler);
    result
}
