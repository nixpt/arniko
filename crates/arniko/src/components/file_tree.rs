//! FileTree — Hierarchical file browser component.
//!
//! Generalized from the Khukuri FileTree. Renders a scrollable list of
//! file and directory entries with indentation, icons, and selection state.

#[cfg(feature = "components")]
use crate::{Component, ComponentMetadata};

// ── Data Types ───────────────────────────────────────────────────────────────

/// A single entry in a file tree.
#[derive(Clone, Debug)]
pub struct FileTreeEntry {
    pub path: String,
    pub is_directory: bool,
    pub depth: usize,
    pub metadata_class: Option<String>,
}

impl FileTreeEntry {
    pub fn new(path: &str, is_directory: bool, depth: usize) -> Self {
        Self {
            path: path.to_string(),
            is_directory,
            depth,
            metadata_class: None,
        }
    }

    pub fn metadata_class(mut self, class: &str) -> Self {
        self.metadata_class = Some(class.to_string());
        self
    }

    /// Create a test file entry (highlighted visually to distinguish test files).
    pub fn test(path: &str, depth: usize) -> Self {
        Self {
            path: path.to_string(),
            is_directory: false,
            depth,
            metadata_class: Some("test".to_string()),
        }
    }
}

/// The full file tree state.
#[derive(Clone, Debug, Default)]
pub struct FileTreeState {
    pub root: String,
    pub entries: Vec<FileTreeEntry>,
    pub selected_file: Option<String>,
    pub file_content: Option<String>,
}

// ── HTML Component ───────────────────────────────────────────────────────────

/// A file tree browser panel.
#[cfg(feature = "components")]
pub struct FileTree {
    state: FileTreeState,
    title: String,
    title_icon: String,
    empty_message: String,
    class: String,
}

#[cfg(feature = "components")]
impl FileTree {
    pub fn new() -> Self {
        Self {
            state: FileTreeState::default(),
            title: "Files".to_string(),
            title_icon: "📁".to_string(),
            empty_message: "No files to display.".to_string(),
            class: String::new(),
        }
    }

    pub fn state(mut self, state: FileTreeState) -> Self {
        self.state = state;
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn title_icon(mut self, icon: &str) -> Self {
        self.title_icon = icon.to_string();
        self
    }

    pub fn empty_message(mut self, msg: &str) -> Self {
        self.empty_message = msg.to_string();
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        if self.state.entries.is_empty() {
            return self.render_empty();
        }

        let rows: String = self
            .state
            .entries
            .iter()
            .map(|e| {
                let indent = "&nbsp;&nbsp;".repeat(e.depth);
                let icon = if e.is_directory {
                    "📁"
                } else {
                    match e.metadata_class.as_deref() {
                        Some("test") => "🧪",
                        _ => "📄",
                    }
                };
                let name = e.path.split('/').last().unwrap_or(&e.path);
                let row_class = if e.is_directory {
                    "arniko-filetree-dir"
                } else {
                    match e.metadata_class.as_deref() {
                        Some("test") => "arniko-filetree-test",
                        _ => "arniko-filetree-file",
                    }
                };
                let selected_class = self
                    .state
                    .selected_file
                    .as_ref()
                    .map(|s| s == &e.path)
                    .unwrap_or(false);
                let selected_attr = if selected_class {
                    " arniko-filetree-selected"
                } else {
                    ""
                };
                format!(
                    r#"<div class="arniko-filetree-row {}{}" data-path="{}">
                    <span class="arniko-filetree-indent">{}</span>
                    <span class="arniko-filetree-icon">{}</span>
                    <span class="arniko-filetree-name">{}</span>
                </div>"#,
                    row_class, selected_attr, e.path, indent, icon, name
                )
            })
            .collect();

        let content_preview = match &self.state.file_content {
            Some(content) => {
                let preview = content.lines().take(20).collect::<Vec<_>>().join("\n");
                let escaped = preview
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");
                format!(
                    r##"<div class="arniko-filetree-preview">
                        <div class="arniko-filetree-preview-header">Preview</div>
                        <pre class="arniko-filetree-preview-body">{}</pre>
                    </div>"##,
                    escaped
                )
            }
            None => String::new(),
        };

        format!(
            r##"<div class="arniko-filetree {}">
                <div class="arniko-filetree-header">
                    <span class="arniko-filetree-header-icon">{}</span>
                    <span class="arniko-filetree-header-title">{}</span>
                    <span class="arniko-filetree-root">{}</span>
                </div>
                <div class="arniko-filetree-body">
                    <div class="arniko-filetree-list">{}</div>
                    {}
                </div>
            </div>"##,
            self.class, self.title_icon, self.title, self.state.root, rows, content_preview
        )
    }

    fn render_empty(&self) -> String {
        format!(
            r##"<div class="arniko-filetree {}">
                <div class="arniko-filetree-header">
                    <span class="arniko-filetree-header-icon">{}</span>
                    <span class="arniko-filetree-header-title">{}</span>
                </div>
                <div class="arniko-filetree-empty">{}</div>
            </div>"##,
            self.class, self.title_icon, self.title, self.empty_message
        )
    }
}

#[cfg(feature = "components")]
impl Default for FileTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "components")]
impl Component for FileTree {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec!["arniko-filetree".to_string()],
            requires_gpu: false,
            capabilities: vec![],
        }
    }
}

// ── Reactive View ────────────────────────────────────────────────────────────

#[cfg(feature = "reactive")]
use crate::reactive::{ReactiveHtml, Signal, View};

/// Create a reactive file tree that updates when the state signal changes.
#[cfg(feature = "reactive")]
pub fn file_tree_reactive(state_signal: Signal<FileTreeState>) -> Box<dyn View> {
    let html = state_signal.derive(move |state| {
        FileTree {
            state,
            ..FileTree::default()
        }
        .render()
    });
    Box::new(ReactiveHtml::new(html))
}

/// Create a reactive file tree with a builder for full configuration.
#[cfg(feature = "reactive")]
pub fn file_tree_reactive_with<F>(state_signal: Signal<FileTreeState>, build: F) -> Box<dyn View>
where
    F: Fn(&mut FileTree) + Send + Sync + 'static,
{
    let html = state_signal.derive(move |state| {
        let mut tree = FileTree::new();
        build(&mut tree);
        tree.state = state;
        tree.render()
    });
    Box::new(ReactiveHtml::new(html))
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "components")]
    #[test]
    fn test_file_tree_empty() {
        let tree = FileTree::new();
        let html = tree.render();
        assert!(html.contains("arniko-filetree"));
        assert!(html.contains("No files to display"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_file_tree_with_entries() {
        let state = FileTreeState {
            root: "/home/user/project".to_string(),
            entries: vec![
                FileTreeEntry::new("src", true, 0),
                FileTreeEntry::new("src/main.rs", false, 1),
                FileTreeEntry::new("src/lib.rs", false, 1).metadata_class("test"),
            ],
            selected_file: Some("src/main.rs".to_string()),
            file_content: None,
        };
        let tree = FileTree::new().state(state);
        let html = tree.render();
        assert!(html.contains("src/main.rs"));
        assert!(html.contains("arniko-filetree-selected"));
        assert!(html.contains("arniko-filetree-dir"));
        assert!(html.contains("arniko-filetree-test"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_file_tree_custom_title() {
        let tree = FileTree::new().title("Project Source").title_icon("🗂️");
        let html = tree.render();
        assert!(html.contains("Project Source"));
        assert!(html.contains("🗂️"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_file_tree_with_preview() {
        let state = FileTreeState {
            root: "/".to_string(),
            entries: vec![FileTreeEntry::new("main.rs", false, 0)],
            selected_file: Some("main.rs".to_string()),
            file_content: Some("fn main() {}".to_string()),
        };
        let tree = FileTree::new().state(state);
        let html = tree.render();
        assert!(html.contains("fn main()"));
        assert!(html.contains("arniko-filetree-preview"));
    }
}
