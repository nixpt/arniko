//! KeyboardShortcuts — Modal overlay showing keyboard shortcut cheatsheet.
//!
//! Generalized from the Khukuri ShortcutHelp. Renders a modal dialog listing
//! keyboard shortcuts with their associated actions.
//!
//! ## Keyboard Navigation
//!
//! The close button (✕) is keyboard-focusable (`tabindex="0"`).
//! To enable Escape-to-close, register a keydown handler at the app level:
//!
//! ```ignore
//! router.on_keydown(|event| {
//!     if event.key == "Escape" {
//!         show_shortcuts.set(false);
//!     }
//! });
//! ```

#[cfg(feature = "components")]
use crate::components::escape_html;
#[cfg(feature = "components")]
use crate::{Component, ComponentMetadata};

/// A single shortcut entry.
#[derive(Clone, Debug)]
pub struct ShortcutEntry {
    pub keys: String,
    pub action: String,
}

impl ShortcutEntry {
    pub fn new(keys: &str, action: &str) -> Self {
        Self {
            keys: keys.to_string(),
            action: action.to_string(),
        }
    }
}

// ── HTML Component ───────────────────────────────────────────────────────────

/// A keyboard shortcuts cheatsheet rendered as a modal overlay.
/// # Examples
///
/// ```rust,no_run
/// use arniko::{KeyboardShortcuts, ShortcutEntry};
///
/// let shortcuts = KeyboardShortcuts::new(vec![
///     ShortcutEntry::new("Ctrl+S", "Save file"),
///     ShortcutEntry::new("?", "Show this help"),
/// ]);
///
/// let html = shortcuts.render();
/// ```
///
/// With the `reactive` feature:
///
/// ```rust,no_run
/// # #[cfg(feature = "reactive")] {
/// use arniko::{ShortcutEntry, shortcut_help_reactive, reactive::Signal};
///
/// let visible = Signal::new(false);
/// let view = shortcut_help_reactive(
///     &visible,
///     vec![ShortcutEntry::new("Esc", "Close")]
/// );
/// # }
/// ```
#[cfg(feature = "components")]
pub struct KeyboardShortcuts {
    shortcuts: Vec<ShortcutEntry>,
    title: String,
    footer: String,
    class: String,
}

#[cfg(feature = "components")]
impl KeyboardShortcuts {
    /// Create a new shortcuts modal with the given shortcuts.
    pub fn new(shortcuts: Vec<ShortcutEntry>) -> Self {
        Self {
            shortcuts,
            title: "⌨️ Keyboard Shortcuts".to_string(),
            footer: "Press ? or Esc to close".to_string(),
            class: String::new(),
        }
    }

    /// Set the modal title.
    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Set the footer text.
    pub fn footer(mut self, footer: &str) -> Self {
        self.footer = footer.to_string();
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        let rows: String = self
            .shortcuts
            .iter()
            .map(|s| {
                format!(
                    r#"<div class="arniko-shortcut-row">
                    <span class="arniko-shortcut-keys">{}</span>
                    <span class="arniko-shortcut-action">{}</span>
                </div>"#,
                    escape_html(&s.keys),
                    escape_html(&s.action)
                )
            })
            .collect();

        format!(
            r#"<div class="arniko-shortcut-overlay" role="dialog" aria-label="Keyboard shortcuts">
                <div class="arniko-shortcut-modal {}">
                    <div class="arniko-shortcut-header">
                        <span class="arniko-shortcut-title">{}</span>
                        <span class="arniko-shortcut-close" role="button" aria-label="Close" tabindex="0">✕</span>
                    </div>
                    <div class="arniko-shortcut-body">{}</div>
                    <div class="arniko-shortcut-footer">{}</div>
                </div>
            </div>"#,
            escape_html(&self.class),
            escape_html(&self.title),
            rows,
            escape_html(&self.footer)
        )
    }
}

#[cfg(feature = "components")]
impl Component for KeyboardShortcuts {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec!["arniko-shortcut-overlay".to_string()],
            requires_gpu: false,
            capabilities: vec![],
        }
    }
}

// ── Reactive View ────────────────────────────────────────────────────────────

#[cfg(feature = "reactive")]
use crate::reactive::{ReactiveHtml, Signal, View};

/// Create a reactive shortcut help modal whose visibility is controlled by
/// a `Signal<bool>`. When `false`, an empty string is rendered (hidden).
#[cfg(feature = "reactive")]
pub fn shortcut_help_reactive(
    visible_signal: &Signal<bool>,
    shortcuts: Vec<ShortcutEntry>,
) -> Box<dyn View> {
    let html = visible_signal.derive({
        let shortcuts = shortcuts.clone();
        move |visible| {
            if !visible {
                return String::new();
            }
            KeyboardShortcuts::new(shortcuts.clone()).render()
        }
    });
    Box::new(ReactiveHtml::new(html))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "components")]
    #[test]
    fn test_keyboard_shortcuts_render() {
        let modal = KeyboardShortcuts::new(vec![
            ShortcutEntry::new("F5", "Refresh"),
            ShortcutEntry::new("Ctrl+C", "Cancel"),
        ]);
        let html = modal.render();
        assert!(html.contains("F5"));
        assert!(html.contains("Refresh"));
        assert!(html.contains("Ctrl+C"));
        assert!(html.contains("Cancel"));
        assert!(html.contains("Keyboard Shortcuts"));
        assert!(html.contains(r#"role="dialog""#));
        assert!(html.contains(r#"tabindex="0""#));
    }
}
