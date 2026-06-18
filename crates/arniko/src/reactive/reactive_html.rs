//! ReactiveHtml — a View that updates innerHTML when a Reactive source changes.
//!
//! This is the bridge between string-based HTML generation and the live DOM tree.
//! Use it to render component HTML strings that need to update reactively.

use super::reactor::Scope;
use super::signal::Reactive;
use super::{Reactor, View};
use bliss_dom::{Attribute, DocumentMutator, QualName, local_name, ns};

/// A `View` that mounts a `<div>` and keeps its `innerHTML` in sync with a
/// `Reactive<String>` source. The initial HTML is rendered on mount, and
/// subsequent changes from the reactor flush update the innerHTML in-place.
///
/// This bridges the existing string-based `Component` API into the View tree.
///
/// # Example
///
/// ```ignore
/// let html_signal = my_count.derive(|n| format!("<span>Count: {}</span>", n));
/// let view = ReactiveHtml::new(html_signal);
/// ```
pub struct ReactiveHtml<R: Reactive<String>> {
    pub source: R,
    pub class_name: Option<String>,
    pub style: Option<String>,
}

impl<R: Reactive<String>> ReactiveHtml<R> {
    /// Create a new `ReactiveHtml` from a reactive string source.
    pub fn new(source: R) -> Self {
        Self {
            source,
            class_name: None,
            style: None,
        }
    }

    /// Add a CSS class to the wrapper `<div>`.
    pub fn with_class(mut self, class: &str) -> Self {
        self.class_name = Some(class.to_string());
        self
    }

    /// Add inline CSS to the wrapper `<div>`.
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = Some(style.to_string());
        self
    }
}

impl<R: Reactive<String>> View for ReactiveHtml<R> {
    fn mount(
        &self,
        mutator: &mut DocumentMutator,
        reactor: &mut Reactor,
        parent: usize,
    ) -> (usize, Scope) {
        let mut attrs = vec![];
        if let Some(ref cls) = self.class_name {
            attrs.push(Attribute {
                name: QualName::new(None, ns!(), local_name!("class")),
                value: cls.clone(),
            });
        }
        if let Some(ref style) = self.style {
            attrs.push(Attribute {
                name: QualName::new(None, ns!(), local_name!("style")),
                value: style.clone(),
            });
        }

        let node_id =
            mutator.create_element(QualName::new(None, ns!(html), local_name!("div")), attrs);
        mutator.append_children(parent, &[node_id]);

        // Initial render
        let initial_html = self.source.get_value();
        mutator.set_inner_html(node_id, &initial_html);

        // Reactive binding — update innerHTML whenever the source changes
        let source = self.source.clone();
        let scope = reactor.bind_scoped(source, move |m, new_html| {
            m.set_inner_html(node_id, new_html);
        });

        (node_id, scope)
    }
}
