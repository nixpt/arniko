use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bliss::traits::events::{DomEvent, DomEventData, EventPhase, EventSink};

pub(super) type HandlerMap = Arc<Mutex<HashMap<usize, Box<dyn Fn() + Send + Sync>>>>;

/// User-facing event registration surface. Register click handlers by DOM node ID.
/// Node IDs come from `View::mount()` return values.
pub struct EventRouter {
    pub(super) handlers: HandlerMap,
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
}

/// bliss-dom `EventSink` impl. Receives DOM events and dispatches to registered handlers.
/// Passed to `BaseDocument::set_event_sink`. Shares the handler map with `EventRouter`.
pub struct ArnikoEventSink {
    handlers: HandlerMap,
}

/// Create a linked `(EventRouter, ArnikoEventSink)` pair sharing the same handler map.
pub fn event_router() -> (EventRouter, ArnikoEventSink) {
    let handlers = Arc::new(Mutex::new(HashMap::new()));
    (
        EventRouter { handlers: Arc::clone(&handlers) },
        ArnikoEventSink { handlers },
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
        if matches!(event.data, DomEventData::Click(_)) {
            let handlers = self.handlers.lock().unwrap();
            if let Some(handler) = handlers.get(&current_target) {
                handler();
            }
        }
    }
}
