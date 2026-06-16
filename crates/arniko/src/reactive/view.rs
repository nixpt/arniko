use std::fmt::Display;
use std::marker::PhantomData;
use std::sync::Arc;

use bliss_dom::{Attribute, DocumentMutator, QualName, local_name, ns};

use super::signal::Reactive;
use super::{Reactor, Signal};

/// A component that mounts itself into the bliss-dom tree and registers reactive bindings.
/// Returns the root node ID created under `parent`.
pub trait View {
    fn mount(&self, mutator: &mut DocumentMutator, reactor: &mut Reactor, parent: usize) -> usize;
}

// Box<dyn View> is itself a View
impl View for Box<dyn View> {
    fn mount(&self, mutator: &mut DocumentMutator, reactor: &mut Reactor, parent: usize) -> usize {
        (**self).mount(mutator, reactor, parent)
    }
}

fn div_name() -> QualName {
    QualName::new(None, ns!(html), local_name!("div"))
}

fn span_name() -> QualName {
    QualName::new(None, ns!(html), local_name!("span"))
}

/// Injects a raw HTML string into a wrapper `<div>` via `set_inner_html`.
/// Bridges the existing string-based `Component` API into the View tree.
pub struct StaticHtml(pub String);

impl View for StaticHtml {
    fn mount(&self, mutator: &mut DocumentMutator, _reactor: &mut Reactor, parent: usize) -> usize {
        let node_id = mutator.create_element(div_name(), vec![]);
        mutator.append_children(parent, &[node_id]);
        mutator.set_inner_html(node_id, &self.0);
        node_id
    }
}

/// A static text node.
pub struct Text(pub String);

impl View for Text {
    fn mount(&self, mutator: &mut DocumentMutator, _reactor: &mut Reactor, parent: usize) -> usize {
        let node_id = mutator.create_text_node(&self.0);
        mutator.append_children(parent, &[node_id]);
        node_id
    }
}

/// A text node bound to any `Reactive<T>` — a `Signal` or `Computed` — updates in-place when
/// the source changes. Use `ReactiveText::new(signal)` or `ReactiveText::new(computed)`.
pub struct ReactiveText<T: Clone + Display + 'static, R: Reactive<T> = Signal<T>> {
    source: R,
    _marker: PhantomData<T>,
}

impl<T: Clone + Display + 'static, R: Reactive<T>> ReactiveText<T, R> {
    pub fn new(source: R) -> Self {
        ReactiveText {
            source,
            _marker: PhantomData,
        }
    }
}

impl<T: Clone + Display + 'static, R: Reactive<T>> View for ReactiveText<T, R> {
    fn mount(&self, mutator: &mut DocumentMutator, reactor: &mut Reactor, parent: usize) -> usize {
        let initial = self.source.get_value().to_string();
        let node_id = mutator.create_text_node(&initial);
        mutator.append_children(parent, &[node_id]);
        reactor.bind(self.source.clone(), move |m, v| {
            m.set_node_text(node_id, &v.to_string());
        });
        node_id
    }
}

/// A `<div>` wrapping child views. Pass style via `attrs`.
pub struct Div {
    pub attrs: Vec<Attribute>,
    pub children: Vec<Box<dyn View>>,
}

impl Div {
    pub fn new(children: Vec<Box<dyn View>>) -> Self {
        Div {
            attrs: vec![],
            children,
        }
    }

    pub fn styled(style: impl Into<String>, children: Vec<Box<dyn View>>) -> Self {
        let style_attr = Attribute {
            name: QualName::new(None, ns!(), local_name!("style")),
            value: style.into(),
        };
        Div {
            attrs: vec![style_attr],
            children,
        }
    }
}

impl View for Div {
    fn mount(&self, mutator: &mut DocumentMutator, reactor: &mut Reactor, parent: usize) -> usize {
        let node_id = mutator.create_element(div_name(), self.attrs.clone());
        mutator.append_children(parent, &[node_id]);
        for child in &self.children {
            child.mount(mutator, reactor, node_id);
        }
        node_id
    }
}

/// A `<span>` wrapping child views.
pub struct Span {
    pub attrs: Vec<Attribute>,
    pub children: Vec<Box<dyn View>>,
}

impl Span {
    pub fn new(children: Vec<Box<dyn View>>) -> Self {
        Span {
            attrs: vec![],
            children,
        }
    }
}

impl View for Span {
    fn mount(&self, mutator: &mut DocumentMutator, reactor: &mut Reactor, parent: usize) -> usize {
        let node_id = mutator.create_element(span_name(), self.attrs.clone());
        mutator.append_children(parent, &[node_id]);
        for child in &self.children {
            child.mount(mutator, reactor, node_id);
        }
        node_id
    }
}

/// Bridges an existing arniko `Component` into the View tree via `render()` + `set_inner_html`.
#[cfg(feature = "components")]
pub struct ComponentView<C: crate::Component>(pub C);

#[cfg(feature = "components")]
impl<C: crate::Component> View for ComponentView<C> {
    fn mount(&self, mutator: &mut DocumentMutator, reactor: &mut Reactor, parent: usize) -> usize {
        StaticHtml(self.0.render()).mount(mutator, reactor, parent)
    }
}

/// Renders a `Reactive<Vec<T>>` as a list of child views, reconciling on every flush.
///
/// On list change the previous children are removed and fresh ones are mounted from `template`.
/// The template receives a `&T` and returns any `Box<dyn View>`.
///
/// **v1 note**: The initial render mounts items with the live `Reactor`, so items can contain
/// reactive views (`ReactiveText`, etc.). Items remounted during reconciliation use a stub
/// reactor — nested reactivity is silently dropped on updates. For fully reactive items,
/// structure data as `Signal<Vec<Signal<ItemData>>>` and rely on the initial mount.
pub struct For<T, R>
where
    T: Clone + Send + Sync + 'static,
    R: Reactive<Vec<T>>,
{
    source: R,
    template: Arc<dyn Fn(&T) -> Box<dyn View> + Send + Sync>,
    attrs: Vec<Attribute>,
    _marker: PhantomData<T>,
}

impl<T, R> For<T, R>
where
    T: Clone + Send + Sync + 'static,
    R: Reactive<Vec<T>>,
{
    /// Create an unstyled list container.
    pub fn new(source: R, template: impl Fn(&T) -> Box<dyn View> + Send + Sync + 'static) -> Self {
        For {
            source,
            template: Arc::new(template),
            attrs: vec![],
            _marker: PhantomData,
        }
    }

    /// Create a list container with inline CSS applied to the wrapping `<div>`.
    pub fn styled(
        style: impl Into<String>,
        source: R,
        template: impl Fn(&T) -> Box<dyn View> + Send + Sync + 'static,
    ) -> Self {
        let style_attr = Attribute {
            name: QualName::new(None, ns!(), local_name!("style")),
            value: style.into(),
        };
        For {
            source,
            template: Arc::new(template),
            attrs: vec![style_attr],
            _marker: PhantomData,
        }
    }
}

impl<T, R> View for For<T, R>
where
    T: Clone + Send + Sync + 'static,
    R: Reactive<Vec<T>>,
{
    fn mount(&self, mutator: &mut DocumentMutator, reactor: &mut Reactor, parent: usize) -> usize {
        let container_id = mutator.create_element(div_name(), self.attrs.clone());
        mutator.append_children(parent, &[container_id]);

        // Mount initial items into the live reactor so nested reactive views work.
        let initial_list = self.source.get_value();
        let mut initial_ids = Vec::with_capacity(initial_list.len());
        for item in &initial_list {
            let view = (self.template)(item);
            let id = view.mount(mutator, reactor, container_id);
            initial_ids.push(id);
        }

        // Register the reconciliation binding for future list changes.
        reactor.bind_for(
            self.source.clone(),
            Arc::clone(&self.template),
            container_id,
            initial_ids,
        );

        container_id
    }
}
