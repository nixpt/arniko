use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;

use bliss::traits::events::{BlissKeyEvent, DomEvent, DomEventData, EventPhase, EventSink};

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
        self.handlers
            .lock()
            .insert(node_id, Box::new(handler));
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
    pub fn on_keydown_node(&self, node_id: usize, handler: impl Fn(&BlissKeyEvent) + Send + Sync + 'static) -> &Self {
        self.key_handlers
            .lock()
            .entry(node_id)
            .or_insert_with(Vec::new)
            .push(Box::new(handler));
        self
    }

    /// Register an input handler for a text input element.
    /// The handler receives the current input value as a String.
    pub fn on_input(&self, node_id: usize, handler: impl Fn(String) + Send + Sync + 'static) -> &Self {
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
