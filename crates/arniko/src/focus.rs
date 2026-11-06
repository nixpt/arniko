//! Focus navigation math: the tab order and roving focus groups.
//!
//! This is the pure-logic half of arniko's focus model (`docs/RUNE_BORROW.md`
//! track 4). The engine routes key and pointer events; these functions decide
//! *where focus moves* given a set of focusable items. Nothing here touches the
//! DOM, so it can drive either an engine-side focus manager or a reactive view.
//!
//! ## Tab order (`tabindex`)
//!
//! As CSS does:
//! - a **positive** `tabindex` puts an item in the tab order, ordered by the
//!   number ascending — before any `0` item;
//! - `tabindex == 0` (or a naturally focusable control) follows in document
//!   order;
//! - a **negative** `tabindex` is focusable by pointer or script but is not a
//!   Tab stop.
//!
//! ## Roving groups
//!
//! A `focus_group` (a menu, listbox, list, tree, or a segmented control) holds
//! one Tab stop and moves within itself with the arrow keys. Menus, segments
//! and tabs **wrap**; lists, trees and grids **stop** at the ends.
//!
//! ```
//! use arniko::focus::{FocusMove, roving};
//!
//! // A 3-item menu wraps: Down from the last goes to the first.
//! assert_eq!(roving(3, Some(2), FocusMove::Next, true), Some(0));
//! // A list does not: Down from the last stays put.
//! assert_eq!(roving(3, Some(2), FocusMove::Next, false), Some(2));
//! ```

/// A linear focus move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusMove {
    /// Down/Right, or Tab forward.
    Next,
    /// Up/Left, or Tab backward.
    Prev,
    /// Home: the first item.
    First,
    /// End: the last item.
    Last,
}

/// The tab order for focusable items, given their `tabindex` values.
///
/// Returns item indices (positions in the input) in Tab order: items with a
/// positive `tabindex` first, ascending by that number and stable within equal
/// values, then items with `tabindex == 0` in document order. Items with a
/// negative `tabindex` are omitted (they are not Tab stops).
pub fn tab_order(tab_indices: &[i32]) -> Vec<usize> {
    let mut positive: Vec<usize> = (0..tab_indices.len())
        .filter(|&i| tab_indices[i] > 0)
        .collect();
    // Stable sort by tabindex keeps document order within equal values.
    positive.sort_by_key(|&i| tab_indices[i]);
    let zero = (0..tab_indices.len()).filter(|&i| tab_indices[i] == 0);
    positive.into_iter().chain(zero).collect()
}

/// The next Tab stop in `order` from `current` (an item index, or `None` for
/// "before the first"). Wraps at both ends, as the browser does.
pub fn tab_step(order: &[usize], current: Option<usize>, forward: bool) -> Option<usize> {
    if order.is_empty() {
        return None;
    }
    let Some(current) = current else {
        return Some(if forward {
            order[0]
        } else {
            order[order.len() - 1]
        });
    };
    let at = order.iter().position(|&i| i == current)?;
    let len = order.len();
    let next = if forward {
        (at + 1) % len
    } else {
        (at + len - 1) % len
    };
    Some(order[next])
}

/// Roving focus within a group of `len` items.
///
/// `current` is the focused item index, or `None` when nothing is focused.
/// `wrap` is true for menus/segments/tabs (the move wraps around) and false for
/// lists/trees/grids (the move stops at the end). Returns the new index, or
/// `None` for an empty group.
pub fn roving(len: usize, current: Option<usize>, mv: FocusMove, wrap: bool) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let last = len - 1;
    match mv {
        FocusMove::First => Some(0),
        FocusMove::Last => Some(last),
        FocusMove::Next => Some(match current {
            None => 0,
            Some(i) if i >= last => {
                if wrap {
                    0
                } else {
                    last
                }
            }
            Some(i) => i + 1,
        }),
        FocusMove::Prev => Some(match current {
            None => last,
            Some(0) => {
                if wrap {
                    last
                } else {
                    0
                }
            }
            Some(i) => i - 1,
        }),
    }
}

/// Type-select: the first item from `current` onward whose label starts with
/// `ch` (case-insensitive), wrapping past the end back to the start.
///
/// When `current` is `None`, the search starts at the first item. A repeated
/// keypress with no match between cycles to the next match; a full
/// prefix/timeout buffer is a later refinement.
pub fn type_select(labels: &[&str], current: Option<usize>, ch: char) -> Option<usize> {
    let len = labels.len();
    if len == 0 {
        return None;
    }
    let needle = ch.to_ascii_lowercase();
    let start = current.map_or(0, |i| (i + 1) % len);
    // Check every item once, starting just after the current one.
    (0..len).map(|offset| (start + offset) % len).find(|&i| {
        labels[i]
            .chars()
            .next()
            .is_some_and(|c| c.to_ascii_lowercase() == needle)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_order_puts_positive_first_then_zero() {
        // idx: 0 -> 0, 1 -> 2, 2 -> 5, 3 -> 0, 4 -> -1 (not a stop)
        let order = tab_order(&[0, 2, 5, 0, -1]);
        assert_eq!(order, vec![1, 2, 0, 3]);
    }

    #[test]
    fn tab_order_is_stable_within_equal_positive_values() {
        let order = tab_order(&[1, 1, 0]);
        assert_eq!(order, vec![0, 1, 2]);
    }

    #[test]
    fn tab_step_wraps_both_ways() {
        let order = vec![10, 20, 30];
        assert_eq!(tab_step(&order, None, true), Some(10));
        assert_eq!(tab_step(&order, None, false), Some(30));
        assert_eq!(tab_step(&order, Some(10), true), Some(20));
        assert_eq!(tab_step(&order, Some(30), true), Some(10), "forward wraps");
        assert_eq!(
            tab_step(&order, Some(10), false),
            Some(30),
            "backward wraps"
        );
        assert_eq!(tab_step(&[], None, true), None);
    }

    #[test]
    fn roving_wraps_only_when_asked() {
        assert_eq!(roving(3, Some(2), FocusMove::Next, true), Some(0));
        assert_eq!(roving(3, Some(2), FocusMove::Next, false), Some(2));
        assert_eq!(roving(3, Some(0), FocusMove::Prev, true), Some(2));
        assert_eq!(roving(3, Some(0), FocusMove::Prev, false), Some(0));
    }

    #[test]
    fn roving_home_end_and_unfocused_start() {
        assert_eq!(roving(4, Some(2), FocusMove::First, true), Some(0));
        assert_eq!(roving(4, Some(2), FocusMove::Last, true), Some(3));
        // Nothing focused yet: Next starts at the top, Prev at the bottom.
        assert_eq!(roving(4, None, FocusMove::Next, false), Some(0));
        assert_eq!(roving(4, None, FocusMove::Prev, false), Some(3));
        assert_eq!(roving(0, None, FocusMove::Next, true), None);
    }

    #[test]
    fn type_select_wraps_and_is_case_insensitive() {
        let labels = ["Apple", "banana", "Cherry", "apricot"];
        assert_eq!(type_select(&labels, None, 'b'), Some(1));
        // From Apple, 'a' finds apricot (wrapping past the end).
        assert_eq!(type_select(&labels, Some(0), 'a'), Some(3));
        assert_eq!(
            type_select(&labels, Some(3), 'a'),
            Some(0),
            "wraps to Apple"
        );
        assert_eq!(type_select(&labels, Some(0), 'z'), None);
        assert_eq!(type_select(&[], None, 'a'), None);
    }
}
