//! Phase 0 spike (SHELL-SPLIT-0-1) — proves the core "render content to its own
//! GPU texture, then blit it into chrome's scene" primitive works end-to-end in
//! THIS codebase, using ONLY existing `anyrender_vello` API surface (no changes
//! to window_renderer.rs are exercised here — Phase 1 adds `set_overlay_texture`
//! on top of this, see `crates/anyrender_vello/src/window_renderer.rs`).
//!
//! It deliberately reuses the exact mechanism the existing `<canvas>` element
//! already uses end-to-end (see `bliss-dom`'s `mutator.rs::load_custom_paint_src`
//! and `bliss-paint`'s `render.rs::draw_canvas`):
//!   1. A `CustomPaintSource` impl (`SolidColorPaintSource`) renders a small
//!      scene to an OFFSCREEN `wgpu::Texture` — via its OWN `vello::Renderer`,
//!      on the SAME `wgpu::Device`/`Queue` as the live window (obtained from the
//!      `DeviceHandle` handed to `CustomPaintSource::resume`).
//!   2. It's registered with `renderer.register_custom_paint_source(..)`.
//!   3. A `<canvas src="0" style="...">` element in the live DOM triggers
//!      `Paint::Custom` → `render_custom_source` → `ctx.register_texture(..)` →
//!      the texture is sampled directly by Vello as an image fill, positioned
//!      and sized by ordinary CSS layout (so window resize "just works" — no
//!      spike-specific resize handling needed).
//!
//! Run with:
//!   CARGO_TARGET_DIR=/build/target-arniko cargo run -p arniko --features reactive --example texture_overlay_spike
//!
//! NOTE: the DOM content below is built via direct `DocumentMutator::create_element`
//! calls (mirroring `Div::styled`'s own implementation) rather than `StaticHtml`/
//! `set_inner_html`. During this spike, raw-HTML-string content mounted via
//! `set_inner_html` (`StaticHtml`) did not appear on screen at all (confirmed on a
//! plain `<div>+<p>` with no canvas/positioning involved — and `todo_list.rs`'s own
//! `StaticHtml("<h1>Todo List</h1>")` is silently absent from its rendered output
//! too), while content built through the structured `View`/mutator API (`Div`/`Text`)
//! rendered correctly. This looks like a real, separate, pre-existing bug in
//! `set_inner_html`'s fragment-parsing path — out of scope here (not a
//! window_renderer.rs/anyrender_vello concern), flagged in the SHELL-SPLIT-0-1
//! report for a follow-up ticket.
//!
//! Manual verification results (screenshots captured live under an X11 session with
//! GPU passthrough — see SHELL-SPLIT-0-1 report):
//!   - [x] Orange rect renders at the expected position/size on window open.
//!   - [x] Survives window resize (canvas is laid out via CSS `width`/`height` like
//!         any other element — no special-cased resize logic exists or was needed).
//!   - [~] Minimize/restore and X11 unmap/map cycles do not crash or leak visibly,
//!         but did NOT trigger a second `CustomPaintSource::suspend()`/`resume()`
//!         call (winit's `destroy_surfaces`/`can_create_surfaces` hooks are Wayland/
//!         mobile-oriented; this X11 desktop session never revokes the surface on
//!         minimize). The resume() path itself is exercised once at startup with no
//!         issues; a true suspend→resume cycle remains formally unverified live in
//!         this environment.

use anyrender_vello::{
    CustomPaintCtx, CustomPaintSource, DeviceHandle, TextureHandle, vello, wgpu,
};
use arniko::reactive::{Div, Text, View, launch_reactive_configured};
use bliss_dom::{Attribute, QualName, local_name, ns};

fn canvas_name() -> QualName {
    QualName::new(None, ns!(html), local_name!("canvas"))
}

fn style_attr(value: impl Into<String>) -> Attribute {
    Attribute {
        name: QualName::new(None, ns!(), local_name!("style")),
        value: value.into(),
    }
}

const OFFSCREEN_W: u32 = 220;
const OFFSCREEN_H: u32 = 160;

/// GPU state that only exists while the paint source is resumed (i.e. while the
/// window has a live surface). Rebuilt from scratch on every `resume()`.
#[allow(
    dead_code,
    reason = "renderer/device_handle held for lifetime, not re-read after setup"
)]
struct GpuState {
    /// A second, independent `vello::Renderer` used ONLY to rasterize into the
    /// offscreen texture. It shares the window's device/queue but is otherwise
    /// unrelated to the window's own renderer (the one inside `VelloWindowRenderer`,
    /// which is what actually samples this texture via `ctx.register_texture`).
    /// Kept alive (not just used transiently in `resume()`) so a future iteration
    /// of this spike can re-render the offscreen scene on demand.
    renderer: vello::Renderer,
    device_handle: DeviceHandle,
    texture: wgpu::Texture,
}

/// Renders one solid-color scene to an offscreen texture and hands it to Vello's
/// image atlas each frame — the same mechanism `<canvas>` elements already use.
struct SolidColorPaintSource {
    state: Option<GpuState>,
}

impl SolidColorPaintSource {
    fn new() -> Self {
        Self { state: None }
    }

    fn render_solid_scene(
        renderer: &mut vello::Renderer,
        device_handle: &DeviceHandle,
        texture: &wgpu::Texture,
    ) {
        let mut scene = vello::Scene::new();
        scene.fill(
            peniko::Fill::NonZero,
            kurbo::Affine::IDENTITY,
            peniko::Color::from_rgb8(255, 100, 30), // solid orange — visually distinct from the chrome background
            None,
            &kurbo::Rect::new(0.0, 0.0, OFFSCREEN_W as f64, OFFSCREEN_H as f64),
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        renderer
            .render_to_texture(
                &device_handle.device,
                &device_handle.queue,
                &scene,
                &view,
                &vello::RenderParams {
                    base_color: peniko::Color::from_rgb8(255, 100, 30),
                    width: OFFSCREEN_W,
                    height: OFFSCREEN_H,
                    antialiasing_method: vello::AaConfig::Area,
                },
            )
            .expect("phase0-spike: offscreen render_to_texture failed");
    }
}

impl CustomPaintSource for SolidColorPaintSource {
    fn resume(&mut self, device_handle: &DeviceHandle) {
        // `register_texture`/`override_image` require Rgba8Unorm + COPY_SRC (so the
        // texture can be copied into Vello's image atlas each frame). `render_to_texture`
        // separately requires STORAGE_BINDING (Vello rasterizes via a compute pipeline).
        // NOTE: this is a correction vs. the SurfaceRenderer intermediate-texture recipe
        // (STORAGE_BINDING | TEXTURE_BINDING) — that recipe is for a texture that gets
        // *sampled by a TextureBlitter*, not one that gets registered with
        // `Renderer::register_texture`. TEXTURE_BINDING is not required for this path.
        let texture = device_handle
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("shell-split-phase0-offscreen"),
                size: wgpu::Extent3d {
                    width: OFFSCREEN_W,
                    height: OFFSCREEN_H,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });

        let mut renderer = vello::Renderer::new(
            &device_handle.device,
            vello::RendererOptions {
                use_cpu: false,
                antialiasing_support: vello::AaSupport::area_only(),
                num_init_threads: None,
                pipeline_cache: None,
            },
        )
        .expect("phase0-spike: failed to create offscreen vello::Renderer");

        Self::render_solid_scene(&mut renderer, device_handle, &texture);

        self.state = Some(GpuState {
            renderer,
            device_handle: device_handle.clone(),
            texture,
        });
        eprintln!(
            "[phase0-spike] resumed: offscreen {OFFSCREEN_W}x{OFFSCREEN_H} texture created + rendered"
        );
    }

    fn suspend(&mut self) {
        // Drop the offscreen renderer + texture. If the next resume() doesn't panic,
        // the primitive survives a suspend/resume (minimize/restore) cycle cleanly.
        self.state = None;
        eprintln!("[phase0-spike] suspended: offscreen GPU state dropped");
    }

    fn render(
        &mut self,
        mut ctx: CustomPaintCtx<'_>,
        _width: u32,
        _height: u32,
        _scale: f64,
    ) -> Option<TextureHandle> {
        let state = self.state.as_ref()?;
        // NOTE (matches the existing <canvas> mechanism exactly): `register_texture`
        // is called fresh every frame here because `TextureHandle`'s inner `ImageData`
        // is only constructible via `CustomPaintCtx::register_texture` (its field is
        // `pub(crate)`), so a `CustomPaintSource` impl living outside `anyrender_vello`
        // has no way to cache and reuse a handle across frames. Each call inserts a new
        // entry into the window's `vello::Renderer`'s `image_overrides` map that is never
        // cleared unless `unregister_texture` is called — this is a pre-existing
        // characteristic of the canvas mechanism this spike intentionally mirrors, not
        // something introduced here. Phase 1's `set_overlay_texture` (window_renderer.rs)
        // avoids this by caching + unregistering internally, since it has crate-internal
        // access to a `TextureHandle`'s wrapped `ImageData`.
        Some(ctx.register_texture(state.texture.clone()))
    }
}

fn main() {
    launch_reactive_configured(
        // ── DOM setup: a <canvas src="0"> positioned/sized by ordinary CSS ────────
        // Built via direct `DocumentMutator::create_element` calls (the same path
        // `Div`/`Text` use under the hood) rather than `StaticHtml`/`set_inner_html` —
        // see SHELL-SPLIT-0-1 report: raw-HTML-string content mounted via
        // `set_inner_html` was not appearing on screen in this build, while content
        // built through the structured `View`/mutator API renders correctly. `<canvas>`
        // has no structured `View` wrapper, so it's built by hand here, mirroring
        // `Div::styled`'s own implementation (`create_element` + `append_children`).
        |mutator, reactor, _router, root, _rt| {
            let shell = Div::styled(
                "padding:40px; background:#f4f4f5; color:#111111; \
                 font-family:ui-sans-serif,system-ui,sans-serif; \
                 display:flex; flex-direction:column; gap:16px; \
                 min-height:100vh; box-sizing:border-box;",
                vec![Box::new(Text(
                    "Phase 0 spike: orange rect below is an offscreen GPU texture, \
                     blitted via register_texture (not merged into the chrome scene)."
                        .into(),
                ))],
            );
            let (shell_id, _scope) = shell.mount(mutator, reactor, root);

            let canvas_id = mutator.create_element(
                canvas_name(),
                vec![
                    Attribute {
                        name: QualName::new(None, ns!(), local_name!("src")),
                        value: "0".to_string(),
                    },
                    style_attr(
                        "width:220px; height:160px; border:2px solid #a855f7; display:block;",
                    ),
                ],
            );
            mutator.append_children(shell_id, &[canvas_id]);
        },
        // ── Renderer configuration: register the offscreen paint source ───────────
        // `register_custom_paint_source` assigns IDs from a process-global counter
        // starting at 0. This is the only paint source registered in this example's
        // process, so its ID is deterministically 0 — matching the DOM's `<canvas
        // src="0">` above. (A real embedder would thread the returned ID into the DOM
        // instead of relying on registration order.)
        |renderer| {
            let source_id =
                renderer.register_custom_paint_source(Box::new(SolidColorPaintSource::new()));
            debug_assert_eq!(
                source_id, 0,
                "phase0-spike: <canvas src=\"0\"> assumes first-registered id"
            );
        },
    );
}
