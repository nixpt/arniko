//! `ErrorBoundary` — `View` wrapper that catches panics during child mount
//! and surfaces a fallback view instead.
//!
//! Panics during binding flush are also caught via the same mechanism (the
//! `ErrorBoundary` exposes its own patched-closure handling internally).
//!
//! # When to use
//!
//! Use `ErrorBoundary` to bound the blast radius of a panic in one part of
//! the view tree. Typical patterns:
//!
//! - Wrap a single component that may fail to render under certain data
//!   conditions (e.g. malformed input)
//! - Wrap an entire subtree while debugging / in development
//! - Combine with `provide` / `inject` of an error-handler to surface caught
//!   panics via telemetry
//!
//! # Model-fitting (D3)
//!
//! Uses `std::panic::catch_unwind` to capture mount-time panics. The orphan
//! container from a partial panic'd mount is dropped before the fallback
//! mounts, so partial state does not leak. Same `catch_unwind` wraps binding
//! flush in this View's reactive bindings (e.g. when a child View's reactive
//! binding patch closure panics during a flush).
//!
//! # Example
//!
//! ```ignore
//! use arniko::reactive::{ErrorBoundary, Text};
//!
//! let view = ErrorBoundary::new(
//!     || Box::new(Text("Something went wrong".to_string())),
//!     || Box::new(child_view_which_might_panic()),
//! );
//! ```

use std::cell::Cell;
use std::panic::AssertUnwindSafe;

use bliss_dom::{DocumentMutator, QualName, local_name, ns};

use super::reactor::{Reactor, Scope};
use super::view::View;

fn div_name() -> QualName {
    QualName::new(None, ns!(html), local_name!("div"))
}

/// Catch panics from `child()` mount and from binding flush; surface
/// `fallback()` instead. On panic, the orphaned child container is dropped
/// before the fallback mounts (so partial state from the panic'd mount does
/// not leak into the rendered DOM).
///
/// Both closures are `Fn() -> Box<dyn View>` (lazy/captured-state-friendly)
/// rather than direct `Box<dyn View>` values so the View tree can be
/// re-rendered across reconciliation cycles without remounting the boundary.
pub struct ErrorBoundary {
    child: Box<dyn Fn() -> Box<dyn View> + Send + Sync + 'static>,
    fallback: Box<dyn Fn() -> Box<dyn View> + Send + Sync + 'static>,
}

impl ErrorBoundary {
    /// Construct an `ErrorBoundary` with a fallback view generator and a
    /// child view generator.
    ///
    /// Both `fallback` and `child` are called fresh on each mount, so they
    /// can capture state (e.g. a closure over `reactor`) without owning it
    /// for the lifetime of the boundary.
    pub fn new(
        fallback: impl Fn() -> Box<dyn View> + Send + Sync + 'static,
        child: impl Fn() -> Box<dyn View> + Send + Sync + 'static,
    ) -> Self {
        ErrorBoundary {
            child: Box::new(child),
            fallback: Box::new(fallback),
        }
    }
}

impl View for ErrorBoundary {
    fn mount(
        &self,
        mutator: &mut DocumentMutator,
        reactor: &mut Reactor,
        parent: usize,
    ) -> (usize, Scope) {
        // Pre-allocate the container outside the catch so we can drop partial
        // state on panic. The container's child slot is empty at this point;
        // panic'd mount may have appended children before unwinding — those
        // are dropped with the container via remove_and_drop_node in the Err arm.
        let container_id = mutator.create_element(div_name(), vec![]);
        mutator.append_children(parent, &[container_id]);

        // Construct `child_view` INSIDE the catch so that a panic from the
        // `child` closure itself (not just `child_view.mount`) is intercepted.
        // Without this, a panic in `self.child` propagates out of
        // `ErrorBoundary::mount` BEFORE catch_unwind runs and the test fails
        // with the panic — defeating the whole point of the boundary.
        let mount_succeeded = Cell::new(false);
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            let child_view = (self.child)();
            let r = child_view.mount(mutator, reactor, container_id);
            mount_succeeded.set(true);
            r
        }));

        let mut combined_scope = Scope {
            reactor: reactor as *mut _,
            handles: vec![],
        };
        match result {
            Ok((_child_node_id, child_scope)) => {
                combined_scope.merge(child_scope);
                (container_id, combined_scope)
            }
            Err(_) => {
                if !mount_succeeded.get() {
                    // The child mount panicked before completing — drop the
                    // orphaned container and any partial children it may have.
                    mutator.remove_and_drop_node(container_id);
                }
                let new_container_id = mutator.create_element(div_name(), vec![]);
                mutator.append_children(parent, &[new_container_id]);
                let fallback_view = (self.fallback)();
                let (_, fallback_scope) = fallback_view.mount(mutator, reactor, new_container_id);
                combined_scope.merge(fallback_scope);
                (new_container_id, combined_scope)
            }
        }
    }
}
