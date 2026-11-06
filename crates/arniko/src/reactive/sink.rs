use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

use bliss::traits::events::{BlissKeyEvent, DomEvent, DomEventData, EventPhase, EventSink};
use keyboard_types::{Key, Modifiers};

use crate::focus::{FocusIntent, FocusManager, FocusMove};

pub(super) type ClickHandler = Box<dyn Fn() + Send + Sync>;
pub(super) type KeyEventHandler = Box<dyn Fn(&BlissKeyEvent) + Send + Sync>;
pub(super) type InputHandler = Box<dyn Fn(String) + Send + Sync>;

pub(super) type HandlerMap = Arc<Mutex<HashMap<usize, ClickHandler>>>;
pub(super) type KeyHandlerMap = Arc<Mutex<HashMap<usize, Vec<KeyEventHandler>>>>;
pub(super) type InputHandlerMap = Arc<Mutex<HashMap<usize, InputHandler>>>;
pub(super) type GlobalKeyHandlerMap = Arc<Mutex<Vec<KeyEventHandler>>>;

/// User-facing event registration surface. Register click handlers by DOM node ID.
/// Node IDs come from `View::mount()` return values.
pub struct EventRouter {
    pub(super) handlers: HandlerMap,
    pub(super) key_handlers: KeyHandlerMap,
    pub(super) global_key_handlers: GlobalKeyHandlerMap,
    pub(super) input_handlers: InputHandlerMap,
}

impl EventRouter {
    /// Register a click handler for a mounted node. The handler should call `signal.set()`.
    pub fn on_click(&self, node_id: usize, handler: impl Fn() + Send + Sync + 'static) -> &Self {
        self.handlers.lock().insert(node_id, Box::new(handler));
        self
    }

    /// Remove a click handler.
    pub fn off_click(&self, node_id: usize) -> &Self {
        self.handlers.lock().remove(&node_id);
        self
    }

    /// Register a global key-down handler.
    ///
    /// The handler receives the [`BlissKeyEvent`] for every `keydown` event in the
    /// document. Use `event.key` / `event.modifiers` / `event.code` to filter.
    pub fn on_keydown(&self, handler: impl Fn(&BlissKeyEvent) + Send + Sync + 'static) -> &Self {
        self.global_key_handlers.lock().push(Box::new(handler));
        self
    }

    /// Register a key-down handler for a specific node.
    ///
    /// The handler receives the [`BlissKeyEvent`] when a keydown event occurs
    /// and the node (or its children) has focus.
    pub fn on_keydown_node(
        &self,
        node_id: usize,
        handler: impl Fn(&BlissKeyEvent) + Send + Sync + 'static,
    ) -> &Self {
        self.key_handlers
            .lock()
            .entry(node_id)
            .or_insert_with(Vec::new)
            .push(Box::new(handler));
        self
    }

    /// Register an input handler for a text input element.
    /// The handler receives the current input value as a String.
    pub fn on_input(
        &self,
        node_id: usize,
        handler: impl Fn(String) + Send + Sync + 'static,
    ) -> &Self {
        self.input_handlers
            .lock()
            .insert(node_id, Box::new(handler));
        self
    }

    /// Remove an input handler.
    pub fn off_input(&self, node_id: usize) -> &Self {
        self.input_handlers.lock().remove(&node_id);
        self
    }

    /// Install focus navigation (step 5 of `docs/KEY_ROUTING.md`) on the global
    /// keydown.
    ///
    /// Maps a key to a [`FocusIntent`] via [`focus_intent`], applies it to
    /// `manager`, and calls `on_change` with the newly focused node when focus
    /// moves. Returns whether the key was a navigation key (so a caller that
    /// wants to also route the key elsewhere can tell).
    ///
    /// ```
    /// use std::sync::Arc;
    /// use parking_lot::Mutex;
    /// use arniko::focus::{FocusItem, FocusManager};
    /// # use arniko::reactive::event_router;
    /// let (router, _sink) = event_router();
    /// let manager = Arc::new(Mutex::new(FocusManager::new()));
    /// manager.lock().register(FocusItem::new(1));
    /// manager.lock().register(FocusItem::new(2));
    /// router.on_focus_nav(manager, |_focused| {});
    /// ```
    pub fn on_focus_nav(
        &self,
        manager: Arc<Mutex<FocusManager>>,
        on_change: impl Fn(usize) + Send + Sync + 'static,
    ) -> &Self {
        self.on_keydown(move |event| {
            if let Some(intent) = focus_intent(event) {
                let mut manager = manager.lock();
                if manager.handle(intent)
                    && let Some(node) = manager.focused_node()
                {
                    on_change(node);
                }
            }
        })
    }
}

/// Map a key event to a focus-navigation [`FocusIntent`], or `None` if it is not
/// a navigation key.
///
/// Covers step 5 of `docs/KEY_ROUTING.md`: Tab / Shift-Tab, arrows, Home/End, and
/// printable characters for type-select.
pub fn focus_intent(event: &BlissKeyEvent) -> Option<FocusIntent> {
    let shift = event.modifiers.contains(Modifiers::SHIFT);
    match &event.key {
        Key::Tab => Some(if shift {
            FocusIntent::TabBackward
        } else {
            FocusIntent::TabForward
        }),
        Key::ArrowDown | Key::ArrowRight => Some(FocusIntent::Move(FocusMove::Next)),
        Key::ArrowUp | Key::ArrowLeft => Some(FocusIntent::Move(FocusMove::Prev)),
        Key::Home => Some(FocusIntent::Move(FocusMove::First)),
        Key::End => Some(FocusIntent::Move(FocusMove::Last)),
        Key::Character(text) => text.chars().next().map(FocusIntent::Type),
        _ => None,
    }
}

/// bliss-dom `EventSink` impl. Receives DOM events and dispatches to registered handlers.
/// Passed to `BaseDocument::set_event_sink`. Shares the handler map with `EventRouter`.
pub struct ArnikoEventSink {
    handlers: HandlerMap,
    key_handlers: KeyHandlerMap,
    global_key_handlers: GlobalKeyHandlerMap,
    input_handlers: InputHandlerMap,
}

/// Create a linked `(EventRouter, ArnikoEventSink)` pair sharing the same handler map.
pub fn event_router() -> (EventRouter, ArnikoEventSink) {
    let handlers = Arc::new(Mutex::new(HashMap::new()));
    let key_handlers = Arc::new(Mutex::new(HashMap::new()));
    let global_key_handlers = Arc::new(Mutex::new(Vec::new()));
    let input_handlers = Arc::new(Mutex::new(HashMap::new()));
    (
        EventRouter {
            handlers: Arc::clone(&handlers),
            key_handlers: Arc::clone(&key_handlers),
            global_key_handlers: Arc::clone(&global_key_handlers),
            input_handlers: Arc::clone(&input_handlers),
        },
        ArnikoEventSink {
            handlers,
            key_handlers,
            global_key_handlers,
            input_handlers,
        },
    )
}

impl EventSink for ArnikoEventSink {
    fn on_dom_event(
        &self,
        _doc_id: usize,
        event: &DomEvent,
        _phase: EventPhase,
        current_target: usize,
    ) {
        match &event.data {
            DomEventData::Click(_) => {
                let handlers = self.handlers.lock();
                if let Some(handler) = handlers.get(&current_target) {
                    handler();
                }
            }
            DomEventData::KeyDown(key_event) => {
                // First, fire per-node handlers for the current target
                let node_handlers = self.key_handlers.lock();
                if let Some(target_handlers) = node_handlers.get(&current_target) {
                    for handler in target_handlers.iter() {
                        handler(key_event);
                    }
                }

                // Then, fire global handlers
                let global_handlers = self.global_key_handlers.lock();
                for handler in global_handlers.iter() {
                    handler(key_event);
                }
            }
            DomEventData::Input(input_event) => {
                let handlers = self.input_handlers.lock();
                if let Some(handler) = handlers.get(&current_target) {
                    handler(input_event.value.clone());
                }
            }
            _ => {}
        }
    }
}
