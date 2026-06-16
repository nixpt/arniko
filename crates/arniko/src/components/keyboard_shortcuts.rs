//! KeyboardShortcuts — Modal overlay showing keyboard shortcut cheatsheet.
//!
//! Generalized from the Khukuri ShortcutHelp. Renders a modal dialog listing
//! keyboard shortcuts with their associated actions.

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
                    s.keys, s.action
                )
            })
            .collect();

        format!(
            r#"<div class="arniko-shortcut-overlay">
                <div class="arniko-shortcut-modal {}">
                    <div class="arniko-shortcut-header">
                        <span class="arniko-shortcut-title">{}</span>
                        <span class="arniko-shortcut-close">✕</span>
                    </div>
                    <div class="arniko-shortcut-body">{}</div>
                    <div class="arniko-shortcut-footer">{}</div>
                </div>
            </div>"#,
            self.class, self.title, rows, self.footer
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
    }
}
