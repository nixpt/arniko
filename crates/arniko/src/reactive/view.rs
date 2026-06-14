use std::fmt::Display;

use bliss_dom::{Attribute, DocumentMutator, QualName, local_name, ns};

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

/// A text node bound to a signal — updates in-place when the signal changes.
pub struct ReactiveText<T: Clone + Display + 'static> {
    pub signal: Signal<T>,
}

impl<T: Clone + Display + 'static> ReactiveText<T> {
    pub fn new(signal: Signal<T>) -> Self {
        ReactiveText { signal }
    }
}

impl<T: Clone + Display + 'static> View for ReactiveText<T> {
    fn mount(&self, mutator: &mut DocumentMutator, reactor: &mut Reactor, parent: usize) -> usize {
        let initial = self.signal.get().to_string();
        let node_id = mutator.create_text_node(&initial);
        mutator.append_children(parent, &[node_id]);
        reactor.bind(self.signal.clone(), move |m, v| {
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
        Div { attrs: vec![], children }
    }

    pub fn styled(style: impl Into<String>, children: Vec<Box<dyn View>>) -> Self {
        let style_attr = Attribute {
            name: QualName::new(None, ns!(), local_name!("style")),
            value: style.into(),
        };
        Div { attrs: vec![style_attr], children }
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
        Span { attrs: vec![], children }
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
