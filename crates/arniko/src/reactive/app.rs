use std::sync::{Arc, Mutex};

use anyrender_vello::VelloWindowRenderer;
use bliss::dom::{Document, DocumentConfig};
use bliss::html::HtmlDocument;
use bliss::shell::{
    BlissApplication, BlissShellEvent, BlissShellProxy, WindowConfig, create_default_event_loop,
};
use bliss::traits::net::DummyNetProvider;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use super::sink::{EventRouter, event_router};
use super::Reactor;

/// ApplicationHandler that wraps BlissApplication and flushes the reactor after each window event.
struct ReactiveApplication {
    inner: BlissApplication<VelloWindowRenderer>,
    reactor: Arc<Mutex<Reactor>>,
}

impl ApplicationHandler for ReactiveApplication {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.can_create_surfaces(event_loop);
    }

    fn destroy_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.destroy_surfaces(event_loop);
    }

    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.resumed(event_loop);
    }

    fn suspended(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.suspended(event_loop);
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.proxy_wake_up(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if matches!(event, WindowEvent::CloseRequested) {
            let window = self.inner.windows.remove(&window_id);
            drop(window);
            if self.inner.windows.is_empty() {
                event_loop.exit();
            }
            return;
        }

        if let Some(view) = self.inner.windows.get_mut(&window_id) {
            view.handle_winit_event(event);

            // Flush reactor: apply all dirty signal patches into the DOM
            let mut inner = view.doc.inner_mut();
            let mut mutator = inner.mutate();
            self.reactor.lock().unwrap().flush(&mut mutator);
            drop(mutator);
            drop(inner);
            view.request_redraw();
        }

        self.inner.proxy.send_event(BlissShellEvent::Poll { window_id });
    }
}

/// Launch a reactive arniko application.
///
/// `setup` receives a `DocumentMutator` (for mounting views), a `Reactor` (for signal bindings),
/// and an `EventRouter` (for click handler registration). Mount your root view and register
/// handlers here; the reactor flushes automatically after every input event.
///
/// # Example
/// ```no_run
/// use arniko::reactive::{Signal, ReactiveText, launch_reactive};
/// use arniko::button::ButtonVariant;
/// use arniko::{Button, ComponentView};
///
/// launch_reactive(|mutator, reactor, router| {
///     let count = Signal::new(0u32);
///     let body = mutator.doc.root_element().id; // approximate — use query_selector in practice
///
///     let text_id = ReactiveText::new(count.clone()).mount(mutator, reactor, body);
///     router.on_click(text_id, move || count.update(|n| n + 1));
/// });
/// ```
/// `root_id` passed to setup is the `<body id="arniko-root">` node — mount your views there.
pub fn launch_reactive(setup: impl FnOnce(&mut bliss::dom::DocumentMutator, &mut Reactor, &mut EventRouter, usize)) {
    let event_loop = create_default_event_loop();
    let (proxy, receiver) = BlissShellProxy::new(event_loop.create_proxy());

    let net_provider = Arc::new(DummyNetProvider);

    let mut doc = HtmlDocument::from_html(
        r#"<!DOCTYPE html><html><head></head><body id="arniko-root"></body></html>"#,
        DocumentConfig {
            net_provider: Some(net_provider),
            ..Default::default()
        },
    );

    let (mut router, sink) = event_router();
    let reactor = {
        let mut inner = doc.inner_mut();
        inner.set_event_sink(Arc::new(sink));
        inner.set_events_enabled(true);

        let root_id = inner
            .get_element_by_id("arniko-root")
            .unwrap_or_else(|| inner.root_element().id);

        let mut reactor = Reactor::new();
        let mut mutator = inner.mutate();
        setup(&mut mutator, &mut reactor, &mut router, root_id);
        reactor
    };

    let reactor = Arc::new(Mutex::new(reactor));
    let renderer = VelloWindowRenderer::new();
    let window = WindowConfig::new(Box::new(doc) as _, renderer);

    let mut application = BlissApplication::new(proxy, receiver);
    application.add_window(window);

    let reactive_app = ReactiveApplication {
        inner: application,
        reactor,
    };

    event_loop.run_app(reactive_app).unwrap();
}
