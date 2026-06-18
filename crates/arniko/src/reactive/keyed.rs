//! `KeyedFor<T, K, R>` — keyed list diff (complement to `For`'s positional diff).
//!
//! For lists where item identity matters across reorders (e.g. todo items
//! keyed by stable id, table rows keyed by primary key, log entries keyed by
//! timestamp), use `KeyedFor` instead of [`For`]. The keyed diff:
//!
//! - **New keys** → mount a fresh DOM node + child reactor under this key
//! - **Missing keys** → drop the DOM node + drop the child reactor (cleanup
//!   cascade via `remove_and_drop_node`)
//! - **Existing keys** → preserve the DOM node + child reactor; reposition
//!   via `mutator.insert_nodes_before(...)` to reflect the new paint order
//!
//! Versus [`For`]'s positional diff (which preserves nodes by *position*),
//! `KeyedFor` preserves nodes by *identity* — surviving items keep their
//! internal state across reorders, prepends, and appends.
//!
//! # Model-fitting (D3)
//!
//! Requires `K: Clone + Hash + Eq + Send + Sync + 'static` for the key type.
//! The key extractor and template are both `Arc<dyn Fn(...) + Send + Sync>`.
//!
//! # Example
//!
//! ```ignore
//! use arniko::reactive::{KeyedFor, ReactiveText, Signal};
//!
//! #[derive(Clone)]
//! struct Todo { id: u32, title: String, done: bool }
//!
//! let todos: Signal<Vec<Todo>> = Signal::new(vec![
//!     Todo { id: 1, title: "First".into(),  done: false },
//!     Todo { id: 2, title: "Second".into(), done: false },
//! ]);
//!
//! let view = KeyedFor::new(
//!     todos.clone(),
//!     |t: &Todo| t.id,
//!     |t: &Todo| Box::new(ReactiveText::new(t.title.clone())),
//! );
//!
//! todos.set(vec![
//!     Todo { id: 2, title: "Second".into(), done: true }, // moved to position 0
//!     Todo { id: 1, title: "First".into(),  done: false },
//!     Todo { id: 3, title: "Third".into(),  done: false },  // new
//! ]);
//! // After flush:
//! // - id=1 + id=2 keep their DOM nodes (just reordered)
//! // - id=3 mounts a new DOM node at the end
//! ```

use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;

use bliss_dom::{Attribute, DocumentMutator, QualName, local_name, ns};

use super::reactor::{ItemState, Reactor, Scope};
use super::signal::Reactive;
use super::view::View;

fn div_name() -> QualName {
    QualName::new(None, ns!(html), local_name!("div"))
}

/// Keyed list view. Each child is identified by `K` (extracted via `key_fn`).
/// Reconciliation preserves DOM nodes + child reactors across reorders.
pub struct KeyedFor<T, K, R>
where
    T: Clone + Send + Sync + 'static,
    K: Clone + Hash + Eq + Send + Sync + 'static,
    R: Reactive<Vec<T>>,
{
    source: R,
    key_fn: Arc<dyn Fn(&T) -> K + Send + Sync>,
    template: Arc<dyn Fn(&T) -> Box<dyn View> + Send + Sync>,
    attrs: Vec<Attribute>,
    _marker: PhantomData<T>,
}

impl<T, K, R> KeyedFor<T, K, R>
where
    T: Clone + Send + Sync + 'static,
    K: Clone + Hash + Eq + Send + Sync + 'static,
    R: Reactive<Vec<T>>,
{
    /// Create a keyed list view with no extra styling.
    pub fn new(
        source: R,
        key_fn: impl Fn(&T) -> K + Send + Sync + 'static,
        template: impl Fn(&T) -> Box<dyn View> + Send + Sync + 'static,
    ) -> Self {
        KeyedFor {
            source,
            key_fn: Arc::new(key_fn),
            template: Arc::new(template),
            attrs: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Create a keyed list view with inline CSS applied to the wrapping
    /// `<div>`.
    pub fn styled(
        style: impl Into<String>,
        source: R,
        key_fn: impl Fn(&T) -> K + Send + Sync + 'static,
        template: impl Fn(&T) -> Box<dyn View> + Send + Sync + 'static,
    ) -> Self {
        let style_attr = Attribute {
            name: QualName::new(None, ns!(), local_name!("style")),
            value: style.into(),
        };
        KeyedFor {
            source,
            key_fn: Arc::new(key_fn),
            template: Arc::new(template),
            attrs: vec![style_attr],
            _marker: PhantomData,
        }
    }
}

impl<T, K, R> View for KeyedFor<T, K, R>
where
    T: Clone + Send + Sync + 'static,
    K: Clone + Hash + Eq + Send + Sync + 'static,
    R: Reactive<Vec<T>>,
{
    fn mount(
        &self,
        mutator: &mut DocumentMutator,
        reactor: &mut Reactor,
        parent: usize,
    ) -> (usize, Scope) {
        let container_id = mutator.create_element(div_name(), self.attrs.clone());
        mutator.append_children(parent, &[container_id]);

        // Mount initial items keyed by key. Order is recorded in `order: Vec<K>`.
        // Each child reactor is heap-allocated (Box) so its address stays stable
        // across the `items.insert(...)` move below — parked scopes hold raw
        // pointers to the reactor's memory and outlive this binding site.
        let initial_list = self.source.get_value();
        let mut items: HashMap<K, ItemState> = HashMap::with_capacity(initial_list.len());
        let mut order: Vec<K> = Vec::with_capacity(initial_list.len());
        for item in &initial_list {
            let key = (self.key_fn)(item);
            let mut child_reactor = Box::new(Reactor::new());
            let view = (self.template)(item);
            let (node_id, child_scope) = view.mount(mutator, &mut *child_reactor, container_id);
            child_reactor.park_scope(child_scope);
            items.insert(
                key.clone(),
                ItemState {
                    node_id,
                    reactor: child_reactor,
                },
            );
            order.push(key);
        }

        let scope = reactor.bind_keyed_for(
            self.source.clone(),
            Arc::clone(&self.key_fn),
            Arc::clone(&self.template),
            container_id,
            items,
            order,
        );
        (container_id, scope)
    }
}
