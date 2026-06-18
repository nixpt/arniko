//! Conditional view rendering for Arniko.
//!
//! Provides `Show` and `Switch` views for conditional rendering based on reactive signals.

use std::sync::Arc;

use bliss_dom::{DocumentMutator, QualName, local_name, ns};

use super::reactor::Scope;
use super::signal::Reactive;
use super::{Reactor, View};

/// A view that conditionally renders its child based on a boolean signal.
/// When the signal is true, the child is mounted and visible.
/// When the signal is false, the child is unmounted (not just hidden).
///
/// # Example
///
/// ```ignore
/// let visible = Signal::new(true);
/// let view = Show::new(visible, || Box::new(Text("Hello".to_string())));
/// ```
pub struct Show<R: Reactive<bool>> {
    condition: R,
    child: Arc<dyn Fn() -> Box<dyn View> + Send + Sync + 'static>,
}

impl<R: Reactive<bool>> Show<R> {
    /// Create a new Show view that conditionally renders a child.
    pub fn new(condition: R, child: impl Fn() -> Box<dyn View> + Send + Sync + 'static) -> Self {
        Show {
            condition,
            child: Arc::new(child),
        }
    }
}

impl<R: Reactive<bool>> View for Show<R> {
    fn mount(
        &self,
        mutator: &mut DocumentMutator,
        reactor: &mut Reactor,
        parent: usize,
    ) -> (usize, Scope) {
        // Create a container div for the conditional content
        let container_id =
            mutator.create_element(QualName::new(None, ns!(html), local_name!("div")), vec![]);
        mutator.append_children(parent, &[container_id]);

        // Check initial condition
        let initial_condition = self.condition.get_value();
        let mut combined_scope = Scope {
            reactor: reactor as *mut _,
            handles: vec![],
        };

        if initial_condition {
            let child_view = (self.child)();
            let (_, child_scope) = child_view.mount(mutator, reactor, container_id);
            combined_scope.merge(child_scope);
        }

        // Register a binding to update the child when the condition changes
        let condition = self.condition.clone();
        let child_fn = Arc::clone(&self.child);
        let container_id_copy = container_id;
        let reactor_ptr = reactor as *mut Reactor;
        let scope =
            reactor.bind_scoped(condition, move |mutator: &mut DocumentMutator, visible| {
                // Get the child state
                let children = mutator.child_ids(container_id_copy);

                if *visible && children.is_empty() {
                    // Show: mount the child
                    let child_view = child_fn();
                    let reactor_ref = unsafe { &mut *reactor_ptr };
                    let (_, child_scope) =
                        child_view.mount(mutator, reactor_ref, container_id_copy);
                    // Park the child's scope on the reactor so its bindings survive
                    // past this closure. Without parking, the scope would Drop here
                    // and remove the binding before the next flush — silently killing
                    // the child's reactivity after the first toggle.
                    reactor_ref.park_scope(child_scope);
                } else if !*visible && !children.is_empty() {
                    // Hide: remove all children
                    for child_id in children {
                        mutator.remove_and_drop_node(child_id);
                    }
                }
            });

        combined_scope.merge(scope);
        (container_id, combined_scope)
    }
}

/// A view that switches between multiple children based on a signal value.
/// Uses equality comparison to match the signal value against the match values.
///
/// # Example
///
/// ```ignore
/// enum Tab { Home, Settings, Profile }
/// let current_tab = Signal::new(Tab::Home);
/// let view = Switch::new(
///     current_tab,
///     |tab| match tab {
///         Tab::Home => Box::new(Text("Home".to_string())),
///         Tab::Settings => Box::new(Text("Settings".to_string())),
///         Tab::Profile => Box::new(Text("Profile".to_string())),
///     },
/// );
/// ```
pub struct Switch<T, R>
where
    T: Clone + PartialEq + 'static,
    R: Reactive<T>,
{
    value: R,
    branches: Arc<dyn Fn(&T) -> Box<dyn View> + Send + Sync + 'static>,
}

impl<T, R> Switch<T, R>
where
    T: Clone + PartialEq + Send + Sync + 'static,
    R: Reactive<T>,
{
    /// Create a new Switch view.
    pub fn new(value: R, branches: impl Fn(&T) -> Box<dyn View> + Send + Sync + 'static) -> Self {
        Switch {
            value,
            branches: Arc::new(branches),
        }
    }
}

impl<T, R> View for Switch<T, R>
where
    T: Clone + PartialEq + Send + Sync + 'static,
    R: Reactive<T>,
{
    fn mount(
        &self,
        mutator: &mut DocumentMutator,
        reactor: &mut Reactor,
        parent: usize,
    ) -> (usize, Scope) {
        // Create a container div for the switch content
        let container_id =
            mutator.create_element(QualName::new(None, ns!(html), local_name!("div")), vec![]);
        mutator.append_children(parent, &[container_id]);

        // Get initial value and mount the matching branch
        let initial_value = self.value.get_value();
        let child_view = (self.branches)(&initial_value);
        let (_, child_scope) = child_view.mount(mutator, reactor, container_id);

        let mut combined_scope = Scope {
            reactor: reactor as *mut _,
            handles: vec![],
        };
        combined_scope.merge(child_scope);

        // Register a binding to switch branches when the value changes
        let reactor_ptr = reactor as *mut Reactor;
        let value = self.value.clone();
        let branches = Arc::clone(&self.branches);
        let container_id_copy = container_id;
        let scope = reactor.bind_scoped(value, move |mutator: &mut DocumentMutator, new_value| {
            // Remove current children
            let children = mutator.child_ids(container_id_copy);
            for child_id in children {
                mutator.remove_and_drop_node(child_id);
            }

            // Mount the new branch
            let child_view = branches(new_value);
            let reactor_ref = unsafe { &mut *reactor_ptr };
            let (_, child_scope) = child_view.mount(mutator, reactor_ref, container_id_copy);
            // Park the branch's scope so its bindings survive past this closure
            // (same rationale as Show::mount's re-mount path above).
            reactor_ref.park_scope(child_scope);
        });

        combined_scope.merge(scope);
        (container_id, combined_scope)
    }
}
