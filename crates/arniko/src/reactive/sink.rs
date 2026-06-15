use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bliss::traits::events::{BlissKeyEvent, DomEvent, DomEventData, EventPhase, EventSink};

pub(super) type HandlerMap = Arc<Mutex<HashMap<usize, Box<dyn Fn() + Send + Sync>>>>;
pub(super) type KeyHandlerMap = Arc<Mutex<Vec<Box<dyn Fn(&BlissKeyEvent) + Send + Sync>>>>;
pub(super) type InputHandlerMap = Arc<Mutex<HashMap<usize, Box<dyn Fn(String) + Send + Sync>>>>;

/// User-facing event registration surface. Register click handlers by DOM node ID.
/// Node IDs come from `View::mount()` return values.
pub struct EventRouter {
    pub(super) handlers: HandlerMap,
    pub(super) key_handlers: KeyHandlerMap,
    pub(super) input_handlers: InputHandlerMap,
}

impl EventRouter {
    /// Register a click handler for a mounted node. The handler should call `signal.set()`.
    pub fn on_click(&self, node_id: usize, handler: impl Fn() + Send + Sync + 'static) {
        self.handlers.lock().unwrap().insert(node_id, Box::new(handler));
    }

    /// Remove a click handler.
    pub fn off_click(&self, node_id: usize) {
        self.handlers.lock().unwrap().remove(&node_id);
    }

    /// Register a global key-down handler.
    ///
    /// The handler receives the [`BlissKeyEvent`] for every `keydown` event in the
    /// document. Use `event.key` / `event.modifiers` / `event.code` to filter.
    pub fn on_keydown(&self, handler: impl Fn(&BlissKeyEvent) + Send + Sync + 'static) {
        self.key_handlers.lock().unwrap().push(Box::new(handler));
    }

    /// Register an input handler for a text input element.
    /// The handler receives the current input value as a String.
    pub fn on_input(&self, node_id: usize, handler: impl Fn(String) + Send + Sync + 'static) {
        self.input_handlers.lock().unwrap().insert(node_id, Box::new(handler));
    }

    /// Remove an input handler.
    pub fn off_input(&self, node_id: usize) {
        self.input_handlers.lock().unwrap().remove(&node_id);
    }
}

/// bliss-dom `EventSink` impl. Receives DOM events and dispatches to registered handlers.
/// Passed to `BaseDocument::set_event_sink`. Shares the handler map with `EventRouter`.
pub struct ArnikoEventSink {
    handlers: HandlerMap,
    key_handlers: KeyHandlerMap,
    input_handlers: InputHandlerMap,
}

/// Create a linked `(EventRouter, ArnikoEventSink)` pair sharing the same handler map.
pub fn event_router() -> (EventRouter, ArnikoEventSink) {
    let handlers = Arc::new(Mutex::new(HashMap::new()));
    let key_handlers = Arc::new(Mutex::new(Vec::new()));
    let input_handlers = Arc::new(Mutex::new(HashMap::new()));
    (
        EventRouter {
            handlers: Arc::clone(&handlers),
            key_handlers: Arc::clone(&key_handlers),
            input_handlers: Arc::clone(&input_handlers),
        },
        ArnikoEventSink {
            handlers,
            key_handlers,
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
                let handlers = self.handlers.lock().unwrap();
                if let Some(handler) = handlers.get(&current_target) {
                    handler();
                }
            }
            DomEventData::KeyDown(key_event) => {
                let handlers = self.key_handlers.lock().unwrap();
                if !handlers.is_empty() {
                    for handler in handlers.iter() {
                        handler(key_event);
                    }
                }
            }
            DomEventData::Input(input_event) => {
                let handlers = self.input_handlers.lock().unwrap();
                if let Some(handler) = handlers.get(&current_target) {
                    handler(input_event.value.clone());
                }
            }
            _ => {}
        }
    }
}
