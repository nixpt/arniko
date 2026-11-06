//! Agent tooling: dump the document — the DOM tree plus each node's resolved
//! layout rect — as JSON.
//!
//! This is arniko's analogue of Rune's `--tree` element-tree dump (see
//! `docs/RUNE_BORROW.md`, track 2). An agent editing markup blind can serialize
//! the laid-out tree, read tag names, attributes, text and rects, and diff
//! successive dumps to see what changed. It is read-only: it borrows a
//! `BaseDocument` and returns a `serde_json::Value`.
//!
//! The dump is structural, not a rendering: elements that Bliss inserts for
//! layout (anonymous block boxes) appear with an `"anonymous": true` flag, and
//! whitespace-only text nodes are kept. Rect values come straight from
//! `Node::final_layout` and are zero before layout has run.
//!
//! ```rust,ignore
//! use arniko::inspect::document_tree_string;
//!
//! // `doc` is a `bliss_dom::BaseDocument` (e.g. `view.doc.inner()`).
//! let json = document_tree_string(&doc);
//! println!("{json}");
//! ```

use bliss_dom::{BaseDocument, NodeData};
use serde_json::{Map, Value, json};

/// Serialize `doc`'s node tree to a JSON value.
///
/// Every node is an object with a `rect` (`[x, y, w, h]`) and, for container
/// nodes, a `children` array. Elements add `tag`, optional `attrs` and
/// `anonymous`; text nodes add `text`; the document root reports
/// `"#document"`.
pub fn document_tree(doc: &BaseDocument) -> Value {
    node_json(doc, doc.root_node().id).unwrap_or(Value::Null)
}

/// Pretty-printed JSON of [`document_tree`], ready to write to a `--tree` file
/// or print.
pub fn document_tree_string(doc: &BaseDocument) -> String {
    serde_json::to_string_pretty(&document_tree(doc)).unwrap_or_else(|_| "null".to_string())
}

fn node_json(doc: &BaseDocument, id: usize) -> Option<Value> {
    let node = doc.get_node(id)?;
    let mut obj = Map::new();

    match &node.data {
        NodeData::Text(text) => {
            obj.insert("text".to_owned(), Value::String(text.content.clone()));
        }
        NodeData::Comment => {
            obj.insert("comment".to_owned(), Value::Null);
        }
        NodeData::Document => {
            obj.insert("tag".to_owned(), json!("#document"));
        }
        NodeData::ShadowRoot { host } => {
            obj.insert("shadow_root".to_owned(), json!(host));
        }
        NodeData::Element(_) | NodeData::AnonymousBlock(_) => {
            if let Some(element) = node.data.downcast_element() {
                obj.insert("tag".to_owned(), json!(element.name.local.to_string()));
            }
            if matches!(node.data, NodeData::AnonymousBlock(_)) {
                obj.insert("anonymous".to_owned(), json!(true));
            }
            if let Some(attrs) = node.attrs().filter(|attrs| !attrs.is_empty()) {
                let mut map = Map::new();
                for attr in attrs {
                    map.insert(
                        attr.name.local.to_string(),
                        Value::String(attr.value.clone()),
                    );
                }
                obj.insert("attrs".to_owned(), Value::Object(map));
            }
        }
    }

    let layout = &node.final_layout;
    obj.insert(
        "rect".to_owned(),
        json!([
            layout.location.x,
            layout.location.y,
            layout.size.width,
            layout.size.height
        ]),
    );

    if !matches!(node.data, NodeData::Text(_) | NodeData::Comment) {
        let children: Vec<Value> = node
            .children
            .iter()
            .filter_map(|child| node_json(doc, *child))
            .collect();
        obj.insert("children".to_owned(), Value::Array(children));
    }

    Some(Value::Object(obj))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bliss_dom::{DocumentConfig, qual_name};

    /// Build: <div id="root"><p class="card">Hello<!-- c --></p></div>
    fn sample_doc() -> BaseDocument {
        let mut doc = BaseDocument::new(DocumentConfig::default());
        let mut mutator = doc.mutate();
        let root = mutator.create_element(qual_name!("div"), vec![]);
        mutator.set_attribute(root, qual_name!("id"), "root");
        let child = mutator.create_element(qual_name!("p"), vec![]);
        mutator.set_attribute(child, qual_name!("class"), "card");
        let text = mutator.create_text_node("Hello");
        mutator.append_children(child, &[text]);
        mutator.append_children(root, &[child]);
        mutator.append_children(0, &[root]);
        drop(mutator);
        doc
    }

    #[test]
    fn dumps_document_root_and_nesting() {
        let doc = sample_doc();
        let tree = document_tree(&doc);
        assert_eq!(tree["tag"], "#document");

        let root = &tree["children"][0];
        assert_eq!(root["tag"], "div");
        assert_eq!(root["attrs"]["id"], "root");

        let p = &root["children"][0];
        assert_eq!(p["tag"], "p");
        assert_eq!(p["attrs"]["class"], "card");
        assert_eq!(p["children"][0]["text"], "Hello");
    }

    #[test]
    fn every_node_carries_a_rect() {
        let doc = sample_doc();
        let tree = document_tree(&doc);
        let rect = tree["rect"].as_array().expect("rect is an array");
        assert_eq!(rect.len(), 4);
        // The text node also carries a rect.
        let text = &tree["children"][0]["children"][0]["children"][0];
        assert!(text["rect"].is_array());
    }

    #[test]
    fn text_nodes_have_no_children_key() {
        let doc = sample_doc();
        let tree = document_tree(&doc);
        let text = &tree["children"][0]["children"][0]["children"][0];
        assert!(text.get("children").is_none());
        assert!(text.get("tag").is_none());
    }

    #[test]
    fn string_form_round_trips() {
        let doc = sample_doc();
        let tree = document_tree(&doc);
        let text = document_tree_string(&doc);
        let parsed: Value = serde_json::from_str(&text).expect("valid json");
        assert_eq!(parsed, tree);
    }
}
