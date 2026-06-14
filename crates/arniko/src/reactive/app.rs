use std::sync::{Arc, Mutex};

use anyrender_vello::VelloWindowRenderer;
use bliss::dom::{Document, DocumentConfig};
use bliss::html::HtmlDocument;
use bliss::shell::{
    BlissApplication, BlissShellEvent, BlissShellProxy, WindowConfig, create_default_event_loop,
};
use bliss::traits::net::DummyNetProvider;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use super::sink::{EventRouter, HandlerMap, event_router};
use super::Reactor;

/// ApplicationHandler that wraps BlissApplication, flushes the reactor after each event,
/// and routes click events to registered handlers by walking the DOM ancestor chain.
struct ReactiveApplication {
    inner: BlissApplication<VelloWindowRenderer>,
    reactor: Arc<Mutex<Reactor>>,
    handlers: HandlerMap,
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

        // Detect left-button release before consuming the event — this is when Click fires.
        let is_left_release = matches!(
            &event,
            WindowEvent::PointerButton { state, button, .. }
                if *state == ElementState::Released
                    && matches!(button.clone().mouse_button(), Some(MouseButton::Left))
        );

        if let Some(view) = self.inner.windows.get_mut(&window_id) {
            view.handle_winit_event(event);

            // After the event, hover_node_id is the clicked node (may be a text node).
            // Walk ancestors to find a registered handler.
            if is_left_release {
                let hover_id = view.doc.inner().get_hover_node_id();
                if let Some(id) = hover_id {
                    let chain = view.doc.inner().node_chain(id);
                    let handlers = self.handlers.lock().unwrap();
                    for chain_id in chain {
                        if let Some(handler) = handlers.get(&chain_id) {
                            handler();
                            break;
                        }
                    }
                }
            }

            // Flush all dirty signal patches into the DOM.
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
/// `setup` receives:
/// - `&mut DocumentMutator` — create DOM nodes, mount views
/// - `&mut Reactor`         — signal→DOM patch bindings (registered by `View::mount`)
/// - `&mut EventRouter`     — register click handlers by node ID (from `View::mount` return value)
/// - `usize`                — the `<body id="arniko-root">` node; mount your root view here
///
/// The reactor flushes automatically after every input event, so `signal.set()` in a click
/// handler triggers a DOM patch + redraw on the next event.
///
/// For GPU effects (blur, transforms, …), use [`launch_reactive_configured`] instead.
pub fn launch_reactive(
    setup: impl FnOnce(&mut bliss::dom::DocumentMutator, &mut Reactor, &mut EventRouter, usize),
) {
    launch_reactive_configured(setup, |_| {});
}

/// Launch a reactive arniko application with renderer configuration.
///
/// Like [`launch_reactive`], but also accepts a `configure_renderer` closure that receives
/// `&mut VelloWindowRenderer` before the event loop starts. Use this to register GPU effects
/// via [`VelloWindowRenderer::set_scene_effects`]:
///
/// ```rust,ignore
/// launch_reactive_configured(
///     |mutator, reactor, router, root| { /* DOM + signal setup */ },
///     |renderer| {
///         let mut compositor = MustangCompositor::default();
///         let effects = vec![Effect::blur("panel", 20.0, 1280, 720)
///             .with_region(Region::new(100.0, 100.0, 300.0, 200.0))];
///         renderer.set_scene_effects(move |scene, w, h| {
///             let mut painter = VelloScenePainter::new(scene);
///             compositor.apply_scene_effects(&mut painter, &effects, (w, h));
///         });
///     },
/// );
/// ```
pub fn launch_reactive_configured(
    setup: impl FnOnce(&mut bliss::dom::DocumentMutator, &mut Reactor, &mut EventRouter, usize),
    configure_renderer: impl FnOnce(&mut VelloWindowRenderer),
) {
    let event_loop = create_default_event_loop();
    let (proxy, receiver) = BlissShellProxy::new(event_loop.create_proxy());

    let mut doc = HtmlDocument::from_html(
        r#"<!DOCTYPE html><html><head></head><body id="arniko-root"></body></html>"#,
        DocumentConfig {
            net_provider: Some(Arc::new(DummyNetProvider)),
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

    let handlers = Arc::clone(&router.handlers);
    let reactor = Arc::new(Mutex::new(reactor));

    let mut renderer = VelloWindowRenderer::new();
    configure_renderer(&mut renderer);

    let window = WindowConfig::new(Box::new(doc) as _, renderer);

    let mut application = BlissApplication::new(proxy, receiver);
    application.add_window(window);

    event_loop
        .run_app(ReactiveApplication {
            inner: application,
            reactor,
            handlers,
        })
        .unwrap();
}
