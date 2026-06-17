//! Toast notification component for Arniko.
//!
//! Provides both an HTML `Component` for static rendering and a reactive
//! mount helper that shows/hides based on a `Signal<Option<String>>`.

#[cfg(feature = "components")]
use crate::{Component, ComponentMetadata};
#[cfg(feature = "components")]
use crate::components::escape_html;

#[cfg(feature = "reactive")]
use crate::reactive::{Div, Reactor, Scope, Signal, Text, View};
#[cfg(feature = "reactive")]
use bliss_dom::DocumentMutator;

/// Toast position on screen.
#[derive(Clone, PartialEq, Default)]
pub enum ToastPosition {
    #[default]
    Top,
    Bottom,
}

/// Toast variant determines background color.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ToastVariant {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

// ── HTML Component ───────────────────────────────────────────────────────────

/// A static toast notification. Renders as a fixed-position banner.
#[cfg(feature = "components")]
pub struct Toast {
    message: String,
    variant: ToastVariant,
    position: ToastPosition,
    class: String,
}

#[cfg(feature = "components")]
impl Toast {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            variant: ToastVariant::Info,
            position: ToastPosition::Top,
            class: String::new(),
        }
    }

    pub fn variant(mut self, v: ToastVariant) -> Self {
        self.variant = v;
        self
    }

    pub fn position(mut self, p: ToastPosition) -> Self {
        self.position = p;
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        let variant_class = match self.variant {
            ToastVariant::Info => "arniko-toast-info",
            ToastVariant::Success => "arniko-toast-success",
            ToastVariant::Warning => "arniko-toast-warning",
            ToastVariant::Error => "arniko-toast-error",
        };
        let position_style = match self.position {
            ToastPosition::Top => "top:20px;",
            ToastPosition::Bottom => "bottom:20px;",
        };

        format!(
            r#"<div class="arniko-toast {} {}" style="{}" role="status" aria-live="polite">{}</div>"#,
            variant_class, escape_html(&self.class), position_style, escape_html(&self.message)
        )
    }
}

#[cfg(feature = "components")]
impl Component for Toast {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec!["arniko-toast".to_string()],
            requires_gpu: false,
            capabilities: vec![],
        }
    }
}

// ── Reactive Mount ───────────────────────────────────────────────────────────

/// Mount a reactive toast notification at the top-center of the screen.
///
/// The toast is visible when `toast_message` is `Some` and hidden (opacity 0)
/// when `None`. Text content updates reactively via the reactor.
/// Returns the container node ID and a Scope for lifecycle management.
#[cfg(feature = "reactive")]
pub fn mount_toast(
    mutator: &mut DocumentMutator,
    reactor: &mut Reactor,
    parent_id: usize,
    toast_message: &Signal<Option<String>>,
) -> (usize, Scope) {
    mount_toast_with_variant(
        mutator,
        reactor,
        parent_id,
        toast_message,
        ToastVariant::Error,
    )
}

/// Mount a reactive toast notification with a specific variant styling.
/// Returns the container node ID and a Scope for lifecycle management.
#[cfg(feature = "reactive")]
pub fn mount_toast_with_variant(
    mutator: &mut DocumentMutator,
    reactor: &mut Reactor,
    parent_id: usize,
    toast_message: &Signal<Option<String>>,
    variant: ToastVariant,
) -> (usize, Scope) {
    let bg_color = match variant {
        ToastVariant::Info => "rgba(59,130,246,0.9)",
        ToastVariant::Success => "rgba(34,197,94,0.9)",
        ToastVariant::Warning => "rgba(245,158,11,0.9)",
        ToastVariant::Error => "rgba(239,68,68,0.9)",
    };
    let inner_style = format!(
        "padding:10px 20px; border-radius:6px; background:{}; color:#fff; font-size:13px; font-weight:600; box-shadow:0 4px 12px rgba(0,0,0,0.3); white-space:nowrap;",
        bg_color
    );
    let toast_container = Div::styled(
        "position:fixed; top:20px; left:50%; transform:translateX(-50%) translateY(-10px); z-index:9999; opacity:0; pointer-events:none; transition:opacity 0.3s ease-out, transform 0.3s ease-out;",
        vec![Box::new(Div::styled(
            inner_style,
            vec![Box::new(Text("".to_string()))],
        ))],
    );
    let (container_id, scope) = toast_container.mount(mutator, reactor, parent_id);

    let inner_id = mutator.child_ids(container_id)[0];
    let text_id = mutator.child_ids(inner_id)[0];

    // Apply initial state immediately (reactor binding only fires on version changes)
    let initial = toast_message.get();
    match initial {
        Some(ref text) => {
            mutator.set_node_text(text_id, text);
            mutator.set_style_property(container_id, "opacity", "1");
            mutator.set_style_property(container_id, "transform", "translateX(-50%) translateY(0)");
        }
        None => {}
    }

    let msg = toast_message.clone();
    reactor.bind(msg, move |m, opt| match opt.as_ref() {
        Some(text) => {
            m.set_node_text(text_id, text);
            m.set_style_property(container_id, "opacity", "1");
            m.set_style_property(container_id, "transform", "translateX(-50%) translateY(0)");
        }
        None => {
            m.set_style_property(container_id, "opacity", "0");
            m.set_style_property(
                container_id,
                "transform",
                "translateX(-50%) translateY(-10px)",
            );
        }
    });

    (container_id, scope)
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "components")]
    #[test]
    fn test_toast_render() {
        let toast = super::Toast::new("Hello world");
        let html = toast.render();
        assert!(html.contains("arniko-toast"));
        assert!(html.contains("arniko-toast-info"));
        assert!(html.contains("Hello world"));
        assert!(html.contains(r#"role="status""#));
    }

    #[cfg(feature = "components")]
    #[test]
    fn test_toast_variants() {
        use super::ToastVariant;
        let variants = [
            (ToastVariant::Info, "arniko-toast-info"),
            (ToastVariant::Success, "arniko-toast-success"),
            (ToastVariant::Warning, "arniko-toast-warning"),
            (ToastVariant::Error, "arniko-toast-error"),
        ];
        for (variant, expected) in variants {
            let toast = super::Toast::new("test").variant(variant);
            assert!(toast.render().contains(expected));
        }
    }
}
