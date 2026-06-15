//! ThemeToggle — A button that cycles through available themes.
//!
//! Generalized from the Khukuri theme toggle. Works with a list of theme
//! names and emits the next one on each click.

#[cfg(feature = "components")]
use crate::{Component, ComponentMetadata};

/// A theme toggle button that displays the current theme and cycles to the next.
#[cfg(feature = "components")]
pub struct ThemeToggle {
    current: String,
    next: String,
    class: String,
}

#[cfg(feature = "components")]
impl ThemeToggle {
    /// Create a new theme toggle showing current → next.
    pub fn new(current: &str, next: &str) -> Self {
        Self {
            current: current.to_string(),
            next: next.to_string(),
            class: String::new(),
        }
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        format!(
            r#"<button class="arniko-theme-toggle {}" title="Current: {}">{}</button>"#,
            self.class, self.current,
            format!("{} → {}", self.current, self.next)
        )
    }
}

#[cfg(feature = "components")]
impl Component for ThemeToggle {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec!["arniko-theme-toggle".to_string()],
            requires_gpu: false,
            capabilities: vec![],
        }
    }
}

// ── Reactive View ────────────────────────────────────────────────────────────

#[cfg(feature = "reactive")]
use crate::reactive::{Signal, View, ReactiveHtml};

/// The theme state for a reactive toggle — current theme name and the next one.
#[cfg(feature = "reactive")]
#[derive(Clone, Debug)]
pub struct ThemeState {
    pub current: String,
    pub next: String,
}

/// Create a reactive theme toggle that updates when the theme signal changes.
#[cfg(feature = "reactive")]
pub fn theme_toggle_reactive(
    theme_signal: &Signal<ThemeState>,
) -> Box<dyn View> {
    let html = theme_signal.derive(|state| {
        ThemeToggle::new(&state.current, &state.next).render()
    });
    Box::new(ReactiveHtml::new(html))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "components")]
    #[test]
    fn test_theme_toggle_render() {
        let toggle = ThemeToggle::new("🌙 Dark", "☀️ Light");
        let html = toggle.render();
        assert!(html.contains("arniko-theme-toggle"));
        assert!(html.contains("Dark"));
        assert!(html.contains("Light"));
    }
}
