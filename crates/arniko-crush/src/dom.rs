//! `dom.*` host capabilities over a [noonoo](https://github.com/nixpt/noonoo)
//! document.
//!
//! A [`DomHost`] owns one document behind a shared lock. [`register_dom`]
//! binds the `dom.*` capabilities to that document: `HostCap::call` has no
//! caller argument, so the document a capsule sees is fixed when the host
//! registers the caps.
//!
//! The read and write caps match surfer's `dom.*` contract:
//!
//! | cap | args | returns |
//! |-----|------|---------|
//! | `dom.query` | selector | `{node_id, tag, text}` or null |
//! | `dom.query_all` | selector | array of `{node_id, tag, text}` |
//! | `dom.text` | target | string |
//! | `dom.attr` | target, name | string or null |
//! | `dom.tag` | target | string or null |
//! | `dom.inner_html` | target | string |
//! | `dom.set_text` | target, text | nothing |
//! | `dom.set_attr` | target, name, value | nothing |
//! | `dom.set_style` | target, property, value | nothing |
//! | `dom.remove_style` | target, property | nothing |
//!
//! A *target* is a selector string (the first match), a node id, or a map
//! with a `node_id` field (what `dom.query` returns). A node id that is not
//! in the document, for example a removed node, is an error, not a no-op.
//!
//! ## Gating
//!
//! A host built with [`DomHost::new`] or [`DomHost::from_html`] is
//! **ungated**: it carries an empty [`Gate`], an explicit [`AllowAll`]
//! authorizer and the anonymous subject, so every read and write is allowed.
//! [`DomHost::with_gate`] swaps in a real [`Gate`] (which nodes need which
//! [`Scope`]), an [`Authorizer`] and the [`Subject`] the capsule acts as. The
//! nearest guard at or above a node applies, so a guard on an ancestor covers
//! its whole subtree. Call it before [`register_dom`]: caps registered earlier
//! keep the access they were registered with.
//!
//! Writes (`dom.set_text`, `dom.set_attr`, `dom.set_style`,
//! `dom.remove_style`) go through noonoo-caps' `GatedMutator`, which needs
//! [`Action::Write`](noonoo_caps::Action::Write) on the target (and, for `dom.set_text`, on each child
//! the new text replaces). A denied write returns `Err` naming the cap and
//! the node and leaves the document unchanged.
//!
//! Reads are gated too, with [`Action::Read`](noonoo_caps::Action::Read). `dom.query` and
//! `dom.query_all` skip nodes the subject cannot read; a direct target
//! (`dom.text`, `dom.attr`, `dom.tag`, `dom.inner_html`) that is unreadable is
//! an `Err`. Text a capsule reads elides unreadable descendants, and
//! `dom.inner_html` errors if any node it would serialize is unreadable
//! rather than emit a partial tree.
//!
//! `dom.set_style` and `dom.remove_style` edit the element's inline `style`
//! attribute through `noonoo-style`, so the next style computation sees the
//! change.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use crush_lang_sdk::{HostCap, HostCapSpec, HostCaps, Value};
use noonoo_caps::{
    AllowAll, Authorizer, CapsError, Gate, GatedDocument, GatedMutator, Scope, Subject,
};
use noonoo_dom::{Document, NodeId};

/// Who may do what to which node: a [`Gate`], the [`Authorizer`] that decides
/// it, and the [`Subject`] the capsule acts as.
struct Access {
    gate: Gate,
    authorizer: Box<dyn Authorizer + Send + Sync>,
    subject: Subject,
}

impl Access {
    /// The ungated default: no guards and an explicit [`AllowAll`].
    fn open() -> Self {
        Self {
            gate: Gate::new(),
            authorizer: Box::new(AllowAll),
            subject: Subject::anonymous(),
        }
    }
}

/// A noonoo document shared between the host and its `dom.*` caps.
///
/// Ungated unless [`DomHost::with_gate`] is called; see the module docs.
#[derive(Clone)]
pub struct DomHost {
    document: Arc<Mutex<Document>>,
    access: Arc<Access>,
}

impl DomHost {
    /// Wrap an existing document. The host is ungated: every read and write
    /// is allowed (an empty [`Gate`] with an [`AllowAll`] authorizer).
    pub fn new(document: Document) -> Self {
        Self {
            document: Arc::new(Mutex::new(document)),
            access: Arc::new(Access::open()),
        }
    }

    /// Gate this host: nodes `gate` guards need a scope `authorizer` grants to
    /// `subject`. Replaces any earlier gate. Call it before [`register_dom`];
    /// caps already registered keep the access they had. Clones made earlier
    /// share the document but not the new gate.
    ///
    /// Guards are keyed by node, so build the [`Gate`] from ids you looked up
    /// with [`DomHost::with_document`].
    pub fn with_gate(
        mut self,
        gate: Gate,
        authorizer: impl Authorizer + Send + Sync + 'static,
        subject: Subject,
    ) -> Self {
        self.access = Arc::new(Access {
            gate,
            authorizer: Box::new(authorizer),
            subject,
        });
        self
    }

    /// Parse `html` into a new document.
    pub fn from_html(html: &str) -> Self {
        Self::new(noonoo_html::parse(html))
    }

    /// Run `f` with read access to the document.
    pub fn with_document<R>(&self, f: impl FnOnce(&Document) -> R) -> R {
        f(&self.lock())
    }

    /// Run `f` with write access to the document.
    pub fn with_document_mut<R>(&self, f: impl FnOnce(&mut Document) -> R) -> R {
        f(&mut self.lock())
    }

    fn lock(&self) -> MutexGuard<'_, Document> {
        // A panic while the lock was held cannot leave the tree half-built:
        // every mutation goes through `DocumentMutator`, which checks before
        // it writes. Keep serving the document.
        self.document
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Register the `dom.*` capabilities, bound to `host`'s document.
pub fn register_dom(caps: &mut HostCaps, host: &DomHost) {
    for op in DomOp::ALL {
        caps.register(Box::new(DomCap {
            op,
            host: host.clone(),
        }));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DomOp {
    Query,
    QueryAll,
    Text,
    Attr,
    Tag,
    InnerHtml,
    SetText,
    SetAttr,
    SetStyle,
    RemoveStyle,
}

impl DomOp {
    const ALL: [DomOp; 10] = [
        DomOp::Query,
        DomOp::QueryAll,
        DomOp::Text,
        DomOp::Attr,
        DomOp::Tag,
        DomOp::InnerHtml,
        DomOp::SetText,
        DomOp::SetAttr,
        DomOp::SetStyle,
        DomOp::RemoveStyle,
    ];

    fn name(self) -> &'static str {
        match self {
            DomOp::Query => "dom.query",
            DomOp::QueryAll => "dom.query_all",
            DomOp::Text => "dom.text",
            DomOp::Attr => "dom.attr",
            DomOp::Tag => "dom.tag",
            DomOp::InnerHtml => "dom.inner_html",
            DomOp::SetText => "dom.set_text",
            DomOp::SetAttr => "dom.set_attr",
            DomOp::SetStyle => "dom.set_style",
            DomOp::RemoveStyle => "dom.remove_style",
        }
    }

    fn argc(self) -> usize {
        match self {
            DomOp::Query | DomOp::QueryAll | DomOp::Text | DomOp::Tag | DomOp::InnerHtml => 1,
            DomOp::Attr | DomOp::SetText | DomOp::RemoveStyle => 2,
            DomOp::SetAttr | DomOp::SetStyle => 3,
        }
    }

    fn returns(self) -> bool {
        !matches!(
            self,
            DomOp::SetText | DomOp::SetAttr | DomOp::SetStyle | DomOp::RemoveStyle
        )
    }
}

/// One `dom.*` capability bound to a shared document.
struct DomCap {
    op: DomOp,
    host: DomHost,
}

impl HostCap for DomCap {
    fn spec(&self) -> HostCapSpec {
        HostCapSpec {
            name: self.op.name().to_string(),
            argc: Some(self.op.argc()),
            returns: self.op.returns(),
        }
    }

    fn call(&self, args: Vec<Value>) -> Result<Option<Value>, String> {
        let cap = self.op.name();
        if args.len() != self.op.argc() {
            return Err(format!(
                "{cap}: expected {} argument(s), got {}",
                self.op.argc(),
                args.len()
            ));
        }
        let mut guard = self.host.lock();
        let doc: &mut Document = &mut guard;
        let access = &*self.host.access;
        match self.op {
            DomOp::Query => {
                let selector = arg_string(cap, &args, 0)?;
                let view = view(access, doc);
                let found = noonoo_query::query_selector_all(doc, &selector)
                    .map_err(|e| format!("{cap}: invalid selector {selector:?}: {e}"))?
                    .into_iter()
                    .find(|&id| view.visible(id));
                Ok(Some(match found {
                    Some(id) => node_summary(&view, id),
                    None => Value::Null,
                }))
            }
            DomOp::QueryAll => {
                let selector = arg_string(cap, &args, 0)?;
                let view = view(access, doc);
                let found = noonoo_query::query_selector_all(doc, &selector)
                    .map_err(|e| format!("{cap}: invalid selector {selector:?}: {e}"))?;
                Ok(Some(Value::new_array(
                    found
                        .into_iter()
                        .filter(|&id| view.visible(id))
                        .map(|id| node_summary(&view, id))
                        .collect(),
                )))
            }
            DomOp::Text => {
                let id = target(cap, doc, &args, 0)?;
                let view = view(access, doc);
                require_read(cap, &view, id)?;
                Ok(Some(Value::Str(view.visible_text(id))))
            }
            DomOp::Attr => {
                let id = target(cap, doc, &args, 0)?;
                let name = arg_string(cap, &args, 1)?;
                require_read(cap, &view(access, doc), id)?;
                Ok(Some(
                    doc.element(id)
                        .and_then(|el| el.attr(&name))
                        .map_or(Value::Null, |v| Value::Str(v.to_string())),
                ))
            }
            DomOp::Tag => {
                let id = target(cap, doc, &args, 0)?;
                require_read(cap, &view(access, doc), id)?;
                Ok(Some(
                    doc.element(id)
                        .map_or(Value::Null, |el| Value::Str(el.name.clone())),
                ))
            }
            DomOp::InnerHtml => {
                let id = target(cap, doc, &args, 0)?;
                let view = view(access, doc);
                require_read(cap, &view, id)?;
                // Serializing would leak an unreadable descendant, so refuse
                // the whole call instead of returning a partial tree.
                if view.visible_descendants(id).len() != doc.tree().descendants(id).len() {
                    return Err(format!(
                        "{cap}: node {} contains nodes this subject may not read",
                        id.index()
                    ));
                }
                Ok(Some(Value::Str(noonoo_html::serialize::inner_html(
                    doc, id,
                ))))
            }
            DomOp::SetText => {
                let id = target(cap, doc, &args, 0)?;
                let text = arg_string(cap, &args, 1)?;
                mutator(access, doc)
                    .set_text(id, text)
                    .map_err(|e| write_err(cap, id, e))?;
                Ok(None)
            }
            DomOp::SetAttr => {
                let id = target(cap, doc, &args, 0)?;
                let name = arg_string(cap, &args, 1)?;
                let value = arg_string(cap, &args, 2)?;
                mutator(access, doc)
                    .set_attribute(id, &name, value)
                    .map_err(|e| write_err(cap, id, e))?;
                Ok(None)
            }
            DomOp::SetStyle => {
                let id = target(cap, doc, &args, 0)?;
                let name = arg_string(cap, &args, 1)?;
                let value = arg_string(cap, &args, 2)?;
                mutator(access, doc)
                    .set_style_property(id, &name, &value, false)
                    .map_err(|e| write_err(cap, id, e))?;
                Ok(None)
            }
            DomOp::RemoveStyle => {
                let id = target(cap, doc, &args, 0)?;
                let name = arg_string(cap, &args, 1)?;
                mutator(access, doc)
                    .remove_style_property(id, &name)
                    .map_err(|e| write_err(cap, id, e))?;
                Ok(None)
            }
        }
    }
}

fn view<'a>(access: &'a Access, doc: &'a Document) -> GatedDocument<'a> {
    GatedDocument::new(doc, &access.gate, &*access.authorizer, &access.subject)
}

fn mutator<'a>(access: &'a Access, doc: &'a mut Document) -> GatedMutator<'a> {
    GatedMutator::new(doc, &access.gate, &*access.authorizer, &access.subject)
}

fn require_read(cap: &str, view: &GatedDocument<'_>, id: NodeId) -> Result<(), String> {
    if view.visible(id) {
        Ok(())
    } else {
        let scope = view.gate().required(view.document(), id);
        Err(format!(
            "{cap}: {} denied read on node {}{}",
            view.subject(),
            id.index(),
            scope.map_or(String::new(), scope_note)
        ))
    }
}

fn scope_note(scope: &Scope) -> String {
    format!(" ({scope})")
}

/// Name the cap and node in a refused write. Denials and mutation errors both
/// leave the document untouched.
fn write_err(cap: &str, id: NodeId, err: CapsError) -> String {
    match err {
        CapsError::Denied(denied) => format!("{cap}: node {}: {denied}", id.index()),
        other => format!("{cap}: node {}: {other}", id.index()),
    }
}

fn arg_string(cap: &str, args: &[Value], idx: usize) -> Result<String, String> {
    match args.get(idx) {
        Some(Value::Str(s)) => Ok(s.clone()),
        Some(other) => Err(format!(
            "{cap}: argument {idx} must be a string, got {}",
            kind(other)
        )),
        None => Err(format!("{cap}: missing argument {idx}")),
    }
}

/// Resolve argument `idx` to a node in the document: a selector string (first
/// match), a node id, or a map with a `node_id` field.
fn target(cap: &str, doc: &Document, args: &[Value], idx: usize) -> Result<NodeId, String> {
    let index = match args.get(idx) {
        Some(Value::Str(selector)) => {
            return noonoo_query::query_selector(doc, selector)
                .map_err(|e| format!("{cap}: invalid selector {selector:?}: {e}"))?
                .ok_or_else(|| format!("{cap}: no element matches {selector:?}"));
        }
        Some(Value::Int(n)) => *n,
        Some(Value::Map(map)) => match map.borrow().get("node_id") {
            Some(Value::Int(n)) => *n,
            _ => {
                return Err(format!("{cap}: argument {idx} map has no integer node_id"));
            }
        },
        Some(other) => {
            return Err(format!(
                "{cap}: argument {idx} must be a selector, node id or node, got {}",
                kind(other)
            ));
        }
        None => return Err(format!("{cap}: missing argument {idx}")),
    };
    usize::try_from(index)
        .ok()
        .and_then(|n| node_by_index(doc, n))
        .ok_or_else(|| format!("{cap}: node {index} is not in the document"))
}

/// Find the connected node whose id has index `n`. Searching the live tree
/// (rather than building a `NodeId` from the integer) means a removed or
/// detached node is reported missing instead of silently edited.
fn node_by_index(doc: &Document, n: usize) -> Option<NodeId> {
    let root = doc.root();
    if root.index() == n {
        return Some(root);
    }
    doc.tree()
        .descendants(root)
        .into_iter()
        .find(|id| id.index() == n)
}

fn node_summary(view: &GatedDocument<'_>, id: NodeId) -> Value {
    let doc = view.document();
    let mut map = HashMap::new();
    map.insert("node_id".to_string(), Value::Int(id.index() as i64));
    map.insert(
        "tag".to_string(),
        doc.element(id)
            .map_or(Value::Null, |el| Value::Str(el.name.clone())),
    );
    map.insert(
        "text".to_string(),
        Value::Str(view.visible_text(id).trim().to_string()),
    );
    Value::new_map(map)
}

fn kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Int(_) => "int",
        Value::Float(_) => "float",
        Value::Str(_) => "string",
        Value::Bool(_) => "bool",
        Value::Array(_) => "array",
        Value::Map(_) => "map",
        _ => "value",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noonoo_caps::{Grant, Policy};
    use noonoo_css::{Color, Stylesheet};

    const PAGE: &str = r#"<!doctype html><html><body>
        <div id="panel" class="arniko-card" data-k="v">
          <h2 id="title">Hello</h2>
          <p class="line">one</p>
          <p class="line">two</p>
          <button id="save">Save &amp; exit</button>
        </div>
      </body></html>"#;

    /// Call `name` on a fresh registry bound to `host`.
    fn call(host: &DomHost, name: &str, args: Vec<Value>) -> Result<Option<Value>, String> {
        let op = DomOp::ALL
            .into_iter()
            .find(|op| op.name() == name)
            .expect("known cap");
        DomCap {
            op,
            host: host.clone(),
        }
        .call(args)
    }

    fn s(v: &str) -> Value {
        Value::Str(v.to_string())
    }

    fn str_of(v: Option<Value>) -> String {
        match v {
            Some(Value::Str(s)) => s,
            other => panic!("expected a string, got {other:?}"),
        }
    }

    fn node_id(v: &Value) -> i64 {
        match v {
            Value::Map(m) => match m.borrow().get("node_id") {
                Some(Value::Int(n)) => *n,
                other => panic!("no node_id: {other:?}"),
            },
            other => panic!("expected a node map, got {other:?}"),
        }
    }

    #[test]
    fn caps_are_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DomHost>();
        assert_send_sync::<DomCap>();
    }

    #[test]
    fn register_dom_registers_every_cap() {
        let host = DomHost::from_html(PAGE);
        let mut caps = crush_lang_sdk::HostCapsBuilder::new().build();
        register_dom(&mut caps, &host);
        for op in DomOp::ALL {
            assert!(
                caps.get(op.name()).is_some(),
                "{} not registered",
                op.name()
            );
        }
    }

    #[test]
    fn query_returns_a_node_summary() {
        let host = DomHost::from_html(PAGE);
        let found = call(&host, "dom.query", vec![s("#title")])
            .unwrap()
            .unwrap();
        let Value::Map(m) = &found else {
            panic!("expected a map, got {found:?}")
        };
        let m = m.borrow();
        assert!(matches!(m.get("tag"), Some(Value::Str(t)) if t == "h2"));
        assert!(matches!(m.get("text"), Some(Value::Str(t)) if t == "Hello"));
        assert!(matches!(m.get("node_id"), Some(Value::Int(_))));
    }

    #[test]
    fn query_miss_is_null_and_bad_selector_is_an_error() {
        let host = DomHost::from_html(PAGE);
        let miss = call(&host, "dom.query", vec![s("#nope")]).unwrap();
        assert!(matches!(miss, Some(Value::Null)));
        let err = call(&host, "dom.query", vec![s("p[")]).unwrap_err();
        assert!(err.starts_with("dom.query: invalid selector"), "{err}");
    }

    #[test]
    fn query_all_uses_full_selectors() {
        let host = DomHost::from_html(PAGE);
        let all = call(&host, "dom.query_all", vec![s("#panel > p.line")])
            .unwrap()
            .unwrap();
        let Value::Array(items) = all else {
            panic!("expected an array")
        };
        let texts: Vec<String> = items
            .borrow()
            .iter()
            .map(|v| match v {
                Value::Map(m) => match m.borrow().get("text") {
                    Some(Value::Str(t)) => t.clone(),
                    _ => String::new(),
                },
                _ => String::new(),
            })
            .collect();
        assert_eq!(texts, ["one", "two"]);
    }

    #[test]
    fn reads_accept_node_ids_maps_and_selectors() {
        let host = DomHost::from_html(PAGE);
        let node = call(&host, "dom.query", vec![s("#panel")])
            .unwrap()
            .unwrap();
        let id = node_id(&node);

        assert_eq!(
            str_of(call(&host, "dom.tag", vec![Value::Int(id)]).unwrap()),
            "div"
        );
        assert_eq!(
            str_of(call(&host, "dom.tag", vec![node.clone()]).unwrap()),
            "div"
        );
        assert_eq!(
            str_of(call(&host, "dom.tag", vec![s("#panel")]).unwrap()),
            "div"
        );
        assert_eq!(
            str_of(call(&host, "dom.attr", vec![node.clone(), s("data-k")]).unwrap()),
            "v"
        );
        assert!(matches!(
            call(&host, "dom.attr", vec![node, s("missing")]).unwrap(),
            Some(Value::Null)
        ));
        assert_eq!(
            str_of(call(&host, "dom.text", vec![s("#title")]).unwrap()),
            "Hello"
        );
    }

    #[test]
    fn character_references_decode_and_reserialize() {
        let host = DomHost::from_html(PAGE);
        assert_eq!(
            str_of(call(&host, "dom.text", vec![s("#save")]).unwrap()),
            "Save & exit"
        );
        let html = str_of(call(&host, "dom.inner_html", vec![s("#save")]).unwrap());
        assert_eq!(html, "Save &amp; exit");
    }

    #[test]
    fn inner_html_serializes_children() {
        let host = DomHost::from_html("<ul id=l><li>a</li><li>b</li></ul>");
        assert_eq!(
            str_of(call(&host, "dom.inner_html", vec![s("#l")]).unwrap()),
            "<li>a</li><li>b</li>"
        );
    }

    #[test]
    fn set_text_and_set_attr_mutate_the_shared_document() {
        let host = DomHost::from_html(PAGE);
        let none = call(&host, "dom.set_text", vec![s("#title"), s("Bye <b>")]).unwrap();
        assert!(none.is_none());
        call(
            &host,
            "dom.set_attr",
            vec![s("#panel"), s("data-k"), s("w")],
        )
        .unwrap();

        host.with_document(|doc| {
            let title = doc.get_element_by_id("title").unwrap();
            assert_eq!(doc.text_content(title), "Bye <b>");
            let panel = doc.get_element_by_id("panel").unwrap();
            assert_eq!(doc.element(panel).unwrap().attr("data-k"), Some("w"));
        });
        // Text is text, not markup.
        assert_eq!(
            str_of(call(&host, "dom.inner_html", vec![s("#title")]).unwrap()),
            "Bye &lt;b&gt;"
        );
    }

    #[test]
    fn writes_report_a_missing_target() {
        let host = DomHost::from_html(PAGE);
        let err = call(&host, "dom.set_text", vec![s("#nope"), s("x")]).unwrap_err();
        assert_eq!(err, r##"dom.set_text: no element matches "#nope""##);
    }

    #[test]
    fn a_removed_node_is_an_error_not_a_no_op() {
        let host = DomHost::from_html(PAGE);
        let id = node_id(
            &call(&host, "dom.query", vec![s("#title")])
                .unwrap()
                .unwrap(),
        );
        host.with_document_mut(|doc| {
            let title = doc.get_element_by_id("title").unwrap();
            doc.mutate().remove(title).unwrap();
        });
        let err = call(&host, "dom.set_text", vec![Value::Int(id), s("x")]).unwrap_err();
        assert_eq!(
            err,
            format!("dom.set_text: node {id} is not in the document")
        );
        let err = call(&host, "dom.text", vec![Value::Int(-1)]).unwrap_err();
        assert_eq!(err, "dom.text: node -1 is not in the document");
    }

    #[test]
    fn argument_errors_name_the_cap() {
        let host = DomHost::from_html(PAGE);
        let err = call(&host, "dom.query", vec![Value::Int(1)]).unwrap_err();
        assert_eq!(err, "dom.query: argument 0 must be a string, got int");
        let err = call(&host, "dom.attr", vec![s("#panel")]).unwrap_err();
        assert_eq!(err, "dom.attr: expected 2 argument(s), got 1");
        let err = call(&host, "dom.tag", vec![Value::Bool(true)]).unwrap_err();
        assert_eq!(
            err,
            "dom.tag: argument 0 must be a selector, node id or node, got bool"
        );
    }

    /// Gate: the Arniko golden stylesheet parses and cascades over a page of
    /// rendered Arniko components without panicking.
    #[test]
    fn arniko_golden_stylesheet_renders_without_panic() {
        let sheet = Stylesheet::parse(arniko::ARNIKO_STYLES)
            .unwrap_or_else(|e| panic!("golden stylesheet failed to parse: {e}"));

        // Render through the existing `arniko.*` caps so the page carries the
        // real component markup and classes.
        use crate::caps::*;
        let rendered: Vec<Box<dyn HostCap>> = vec![
            Box::new(ArnikoButtonCap),
            Box::new(ArnikoCardCap),
            Box::new(ArnikoAlertCap),
            Box::new(ArnikoBadgeCap),
            Box::new(ArnikoInputCap),
            Box::new(ArnikoKbdCap),
            Box::new(ArnikoToastCap),
            Box::new(ArnikoProgressBarCap),
        ];
        let body: String = rendered
            .iter()
            .map(|cap| str_of(cap.call(vec![s("Save &amp; exit"), s("Body")]).unwrap()))
            .collect();
        let host = DomHost::from_html(&format!("<!doctype html><html><body>{body}</body></html>"));

        host.with_document(|doc| {
            let styles = noonoo_style::compute(doc, std::slice::from_ref(&sheet));
            let btn = noonoo_query::query_selector(doc, ".arniko-btn")
                .unwrap()
                .expect("button rendered");
            assert!(styles.get(btn).is_some(), "button has no computed style");

            // Layout and paint, with the stand-in metric shaper (no fonts).
            let viewport = noonoo_geometry::Size::new(800.0, 600.0);
            let layout =
                noonoo_layout::layout(doc, &styles, viewport, &noonoo_layout::MetricShaper);
            assert!(layout.roots().count() > 0, "layout produced no boxes");
            let commands = noonoo_paint::paint(&styles, &layout);
            assert!(!commands.is_empty(), "paint produced no commands");
            let json = commands.to_json().to_string();
            assert!(
                json.contains("Save"),
                "button label missing from paint: {json}"
            );
        });
    }

    /// Gate: a style change made through `dom.set_style` reaches computed
    /// style, is inherited by a child, and `dom.remove_style` reverts it.
    #[test]
    fn style_change_reaches_computed_style() {
        let sheet = Stylesheet::parse("#panel { color: rgb(0, 0, 255); }").unwrap();
        let host = DomHost::from_html(r#"<div id="panel"><span id="child">x</span></div>"#);
        let color_of = |id: &str| {
            host.with_document(|doc| {
                let styles = noonoo_style::compute(doc, std::slice::from_ref(&sheet));
                let node = doc.get_element_by_id(id).unwrap();
                styles.get(node).unwrap().color
            })
        };
        let rgb = |r, g, b| Color { r, g, b, a: 255 };

        assert_eq!(color_of("panel"), rgb(0, 0, 255));

        call(
            &host,
            "dom.set_style",
            vec![s("#panel"), s("color"), s("rgb(255, 0, 0)")],
        )
        .unwrap();
        assert_eq!(color_of("panel"), rgb(255, 0, 0));
        assert_eq!(color_of("child"), rgb(255, 0, 0), "inherited by the child");

        call(&host, "dom.remove_style", vec![s("#panel"), s("color")]).unwrap();
        assert_eq!(color_of("panel"), rgb(0, 0, 255));
        assert_eq!(color_of("child"), rgb(0, 0, 255));
    }

    #[test]
    fn set_style_rejects_invalid_css() {
        let host = DomHost::from_html(r#"<div id="panel"></div>"#);
        let err = call(
            &host,
            "dom.set_style",
            vec![s("#panel"), s("color"), s("{")],
        );
        assert!(err.is_err(), "{err:?}");
        assert!(err.unwrap_err().starts_with("dom.set_style: "));
    }

    // --- gating ---

    const VAULT: &str = r#"<div id="vault"><p id="inner">secret <b id="deep">core</b></p></div>
        <p id="open">hello</p>"#;

    fn vault_scope() -> Scope {
        Scope::path("noonoo.dom", "#vault")
    }

    /// A host with `#vault` guarded, acting as `subject`, granted `grants`.
    fn gated(subject: &str, grants: Vec<Grant>) -> DomHost {
        let host = DomHost::from_html(VAULT);
        let vault = host.with_document(|d| d.get_element_by_id("vault").unwrap());
        let mut gate = Gate::new();
        gate.guard(vault, vault_scope());
        let mut policy = Policy::new();
        for g in grants {
            policy.add(g);
        }
        host.with_gate(gate, policy, Subject::new(subject))
    }

    fn html_of(host: &DomHost) -> String {
        host.with_document(|d| noonoo_html::serialize::inner_html(d, d.root()))
    }

    #[test]
    fn gated_allowed_write_succeeds() {
        let host = gated(
            "did:exo:alice",
            vec![Grant::all(Subject::new("did:exo:alice"), vault_scope())],
        );
        call(
            &host,
            "dom.set_attr",
            vec![s("#inner"), s("data-x"), s("1")],
        )
        .unwrap();
        call(
            &host,
            "dom.set_style",
            vec![s("#vault"), s("color"), s("red")],
        )
        .unwrap();
        call(&host, "dom.remove_style", vec![s("#vault"), s("color")]).unwrap();
        call(&host, "dom.set_text", vec![s("#deep"), s("new")]).unwrap();
        assert!(html_of(&host).contains(r#"data-x="1""#));
        assert!(html_of(&host).contains(">new<"));
    }

    #[test]
    fn gated_denied_write_errors_and_changes_nothing() {
        let host = gated("did:exo:mallory", vec![]);
        let before = html_of(&host);
        for (cap, args) in [
            ("dom.set_text", vec![s("#vault"), s("x")]),
            ("dom.set_attr", vec![s("#vault"), s("a"), s("b")]),
            ("dom.set_style", vec![s("#vault"), s("color"), s("red")]),
            ("dom.remove_style", vec![s("#vault"), s("color")]),
        ] {
            let err = call(&host, cap, args).unwrap_err();
            assert!(err.starts_with(&format!("{cap}: node ")), "{err}");
            assert!(err.contains("did:exo:mallory denied write"), "{err}");
        }
        assert_eq!(html_of(&host), before, "a denied write changed the tree");
        // Unguarded nodes stay writable.
        call(&host, "dom.set_text", vec![s("#open"), s("ok")]).unwrap();
    }

    #[test]
    fn read_only_grant_cannot_write() {
        let host = gated(
            "did:exo:bob",
            vec![Grant::read(Subject::new("did:exo:bob"), vault_scope())],
        );
        assert_eq!(
            str_of(call(&host, "dom.text", vec![s("#inner")]).unwrap()),
            "secret core"
        );
        let err = call(&host, "dom.set_text", vec![s("#inner"), s("x")]).unwrap_err();
        assert!(err.contains("denied write"), "{err}");
    }

    #[test]
    fn guard_on_ancestor_covers_descendant() {
        let host = gated("did:exo:mallory", vec![]);
        // #deep has no guard of its own; the nearest guard is #vault.
        let err = call(&host, "dom.set_attr", vec![s("#deep"), s("a"), s("b")]).unwrap_err();
        assert!(err.contains("denied write"), "{err}");
        let err = call(&host, "dom.text", vec![s("#deep")]).unwrap_err();
        assert!(err.contains("denied read"), "{err}");
    }

    #[test]
    fn gated_reads_elide_unreadable_nodes() {
        let host = gated("did:exo:mallory", vec![]);
        assert_eq!(
            call(&host, "dom.query", vec![s("#inner")]).unwrap(),
            Some(Value::Null)
        );
        let Some(Value::Array(all)) = call(&host, "dom.query_all", vec![s("p")]).unwrap() else {
            panic!("expected an array")
        };
        assert_eq!(all.borrow().len(), 1, "only #open is readable");
        // The page-level text drops the vault.
        let body = str_of(call(&host, "dom.text", vec![s("body")]).unwrap());
        assert!(!body.contains("secret") && body.contains("hello"), "{body}");
        let err = call(&host, "dom.inner_html", vec![s("body")]).unwrap_err();
        assert!(err.contains("may not read"), "{err}");
        assert!(call(&host, "dom.inner_html", vec![s("#open")]).is_ok());
    }

    #[test]
    fn ungated_host_is_unchanged() {
        let host = DomHost::from_html(VAULT);
        call(&host, "dom.set_text", vec![s("#deep"), s("x")]).unwrap();
        assert_eq!(
            str_of(call(&host, "dom.text", vec![s("#inner")]).unwrap()),
            "secret x"
        );
        assert!(html_of(&host).contains(">x<"));
    }
}
