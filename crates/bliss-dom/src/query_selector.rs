use selectors::SelectorList;
use smallvec::SmallVec;
use style::dom_apis::{MayUseInvalidation, QueryAll, QueryFirst, query_selector};
use style::selector_parser::{SelectorImpl, SelectorParser};
use style_traits::ParseError;

use crate::{BaseDocument, Node};

impl BaseDocument {
    /// Find the node with the specified id attribute (if one exists)
    pub fn get_element_by_id(&self, id: &str) -> Option<usize> {
        self.nodes_to_id.get(id).copied()
    }

    /// Find the first node that matches the selector specified as a string
    /// Returns:
    ///   - Err(_) if parsing the selector fails
    ///   - Ok(None) if nothing matches
    ///   - Ok(Some(node_id)) with the first node ID that matches if one is found
    pub fn query_selector<'input>(
        &self,
        selector: &'input str,
    ) -> Result<Option<usize>, ParseError<'input>> {
        let selector_list = self.try_parse_selector_list(selector)?;
        Ok(self.query_selector_raw(&selector_list))
    }

    /// Find the first node that matches the selector(s) specified in selector_list
    pub fn query_selector_raw(&self, selector_list: &SelectorList<SelectorImpl>) -> Option<usize> {
        let root_node = self.root_node();
        let mut result = None;
        query_selector::<&Node, QueryFirst>(
            root_node,
            selector_list,
            &mut result,
            MayUseInvalidation::Yes,
        );

        result.map(|node| node.id)
    }

    /// Find all nodes that match the selector specified as a string
    /// Returns:
    ///   - `Err(_)` if parsing the selector fails
    ///   - `Ok(SmallVec<usize>)` with all matching nodes otherwise
    pub fn query_selector_all<'input>(
        &self,
        selector: &'input str,
    ) -> Result<SmallVec<[usize; 32]>, ParseError<'input>> {
        let selector_list = self.try_parse_selector_list(selector)?;
        Ok(self.query_selector_all_raw(&selector_list))
    }

    /// Find all nodes that match the selector(s) specified in selector_list
    pub fn query_selector_all_raw(
        &self,
        selector_list: &SelectorList<SelectorImpl>,
    ) -> SmallVec<[usize; 32]> {
        let root_node = self.root_node();
        let mut results = SmallVec::new();
        query_selector::<&Node, QueryAll>(
            root_node,
            selector_list,
            &mut results,
            MayUseInvalidation::Yes,
        );

        results.iter().map(|node| node.id).collect()
    }

    /// Element-scoped query: all nodes matching `selector` within the subtree
    /// rooted at `root_id`, EXCLUDING the root itself — DOM
    /// `element.querySelectorAll` semantics. Unlike the document-global
    /// [`query_selector_all`](Self::query_selector_all), this traverses from the
    /// given node, so it also queries **detached** subtrees (e.g. a `cloneNode`
    /// result not yet inserted) that the global query cannot reach.
    pub fn query_selector_all_from<'input>(
        &self,
        root_id: usize,
        selector: &'input str,
    ) -> Result<SmallVec<[usize; 32]>, ParseError<'input>> {
        let selector_list = self.try_parse_selector_list(selector)?;
        Ok(self.query_selector_all_from_raw(root_id, &selector_list))
    }

    /// Find all nodes matching `selector_list` in the subtree rooted at
    /// `root_id`, excluding the root. Empty if `root_id` is absent.
    pub fn query_selector_all_from_raw(
        &self,
        root_id: usize,
        selector_list: &SelectorList<SelectorImpl>,
    ) -> SmallVec<[usize; 32]> {
        let Some(root) = self.get_node(root_id) else {
            return SmallVec::new();
        };
        let mut results = SmallVec::new();
        query_selector::<&Node, QueryAll>(root, selector_list, &mut results, MayUseInvalidation::Yes);
        results
            .iter()
            .map(|node| node.id)
            .filter(|id| *id != root_id)
            .collect()
    }

    pub fn try_parse_selector_list<'input>(
        &self,
        input: &'input str,
    ) -> Result<SelectorList<SelectorImpl>, ParseError<'input>> {
        let url_extra_data = self.url.url_extra_data();
        SelectorParser::parse_author_origin_no_namespace(input, &url_extra_data)
    }
}
