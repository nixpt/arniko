# Plan

Captured by agents as they work. An unchecked box is an open item;
nobody has committed to doing it, only to not losing it.

- [ ] **issue** — launch_reactive never installed a real HtmlParserProvider: BaseDocument defaults to DummyHtmlParserProvider (silent no-op), so DocumentMutator::set_inner_html did nothing — ComponentView mounts (Button etc.) rendered empty. Fixed by adding html_parser_provider: Some(Arc::new(bliss::html::HtmlProvider)) in crates/arniko/src/reactive/app.rs. Guarded by bro-desktop unit test component_view_mounts_button_children.  _(unknown, 2026-09-14)_
- [ ] **issue** — EventDriver::run_handler_event calls event_sink.on_dom_event ONCE with current_target=event.target (crates/bliss-dom/src/events/driver.rs:316) — sink calls are NOT bubbled through the ancestor chain, so an arniko EventRouter handler registered on an ancestor node (e.g. the StaticHtml wrapper div that ComponentView::mount returns) can never fire when the click lands on an inner element. Two consumers workarounds exist; the real fix is calling the sink per node in the bubbling chain (target phase first, then bubble phases) or documenting that handlers must be registered on the hit node. Follow-up for reactive UX.  _(unknown, 2026-09-14)_
