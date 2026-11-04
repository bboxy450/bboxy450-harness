// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

// Adapted from bboxy450-term crates/bboxybook/src/window.rs: winit host and
// ordered ChromeRenderer surface pass. GPU setup is moved to bbt-rt.
// The undecorated window, the term's menu bar, the title-bar gestures and the
// desktop actions follow bboxy450-term crates/bboxy450-term/src/main.rs,
// desktop_menu.rs and desktop_window.rs (harness-spec.md Standalone window).
// https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
#![deny(clippy::disallowed_methods)]
use super::{
    bridge::Bridge,
    chrome::Intent,
    options::Options,
    view::{Action, HarnessView},
    zoom::{self, Zoom},
};
use crate::{
    engine::Config,
    policy::Policy,
    protocol::*,
    provider::{FailedProvider, ProviderError, profile::Unattended},
    redact::Set,
};
use bbt_desktop::{Action as Desktop, ClipboardContent};
use bbt_render::{
    chrome::{ChromeRenderer, ChromeTarget, materials as m},
    device::DeviceContext,
};
use bbt_rt::{DirtySet, Latest, Waker};
use bbt_ui::{
    modules::composer,
    text_edit::{Command as Edit, Motion},
};
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, Ime, MouseButton, MouseScrollDelta, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    keyboard::{Key, ModifiersState, NamedKey},
    window::{Window, WindowId},
};
/// The event loop's own events: a frame wake from the bridge, a menu action from the
/// term's menu bar (desktop_menu.rs), the saved zoom read at startup.
#[derive(Debug)]
enum Wake {
    Frame,
    Desktop(Desktop),
    Zoom(Zoom),
}
struct WakePort(EventLoopProxy<Wake>);
impl Waker for WakePort {
    fn wake(&self) {
        let _ = self.0.send_event(Wake::Frame);
    }
}
struct Gpu {
    surface: wgpu::Surface<'static>,
    context: DeviceContext,
    config: wgpu::SurfaceConfiguration,
}
struct App {
    options: Options,
    /// Private unattended startup witness: never show or activate a window.
    smoke_window: bool,
    rendered: bool,
    bridge: Bridge,
    view: HarnessView,
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    renderer: Option<ChromeRenderer>,
    config: Option<wgpu::SurfaceConfiguration>,
    /// The material pass for the alive rims (bbt-render L2), on the surface's format.
    materials: Option<m::Pass>,
    /// The pass's sampled backdrop, sized with the surface. Rims never sample it.
    backdrop: Option<wgpu::Texture>,
    gpu: Arc<Latest<Result<Gpu, String>>>,
    gpu_dirty: DirtySet,
    pointer: [f32; 2],
    modifiers: ModifiersState,
    error: Option<String>,
    /// The animation clock's zero; frames are timed from it.
    started: Instant,
    /// A timed wake the clock asked for (a caret blink edge), applied in `about_to_wait`.
    wake_at: Option<Instant>,
    proxy: EventLoopProxy<Wake>,
    /// The term's menu bar, retained while the window lives.
    menu: Option<bbt_desktop::MenuBar>,
    /// `$HOME/.bboxy450-harness`, where the zoom persists (zoom.rs). None without a HOME.
    settings_root: Option<PathBuf>,
}
fn warn(what: &str, error: impl std::fmt::Display) {
    eprintln!("bboxy450-harness gui: {what}: {error}");
}
fn backdrop(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("harness rim backdrop"),
        size: wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: config.format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}
impl App {
    fn init(&mut self, event_loop: &ActiveEventLoop) -> Result<(), String> {
        // The standalone app is the term's window, not an OS window: no system chrome, the
        // title bar is drawn (chrome.rs), as bboxy450-term main.rs creates its window.
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("bboxy450-harness")
                        .with_inner_size(LogicalSize::new(1120., 860.))
                        .with_min_inner_size(LogicalSize::new(480., 360.))
                        .with_decorations(false)
                        .with_visible(!self.smoke_window),
                )
                .map_err(|e| e.to_string())?,
        );
        window.set_ime_allowed(true);
        self.install_menu();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::METAL,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| e.to_string())?;
        let size = window.inner_size();
        let slot = self.gpu.clone();
        bbt_rt::handle().spawn(async move {
            let result = async {
                let adapter = instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        compatible_surface: Some(&surface),
                        ..Default::default()
                    })
                    .await
                    .map_err(|e| e.to_string())?;
                let caps = surface.get_capabilities(&adapter);
                let format = *caps
                    .formats
                    .iter()
                    .find(|f| f.is_srgb())
                    .ok_or("no sRGB surface")?;
                let info = adapter.get_info();
                let (device, queue) = adapter
                    .request_device(&Default::default())
                    .await
                    .map_err(|e| e.to_string())?;
                let config = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format,
                    width: size.width.max(1),
                    height: size.height.max(1),
                    present_mode: wgpu::PresentMode::Fifo,
                    desired_maximum_frame_latency: 2,
                    alpha_mode: caps.alpha_modes[0],
                    view_formats: vec![format.remove_srgb_suffix()],
                    color_space: wgpu::SurfaceColorSpace::Srgb,
                };
                Ok(Gpu {
                    surface,
                    context: DeviceContext {
                        device,
                        queue,
                        adapter_info: info,
                    },
                    config,
                })
            }
            .await;
            slot.publish(result);
        });
        if let Some(path) = &self.options.replay {
            self.bridge.replay(path.clone());
        } else {
            match self.options.watch.clone() {
                Some((native, path)) => self.bridge.watch(config(&self.options)?, native, path),
                None => match self.options.resume {
                    Some(session) => self.bridge.resume(config(&self.options)?, session),
                    None => self.bridge.create(config(&self.options)?),
                },
            }
        }
        self.window = Some(window);
        Ok(())
    }
    /// The term's app menu (bboxy450-term-desktop-core `app_menu`) through its AppKit menu
    /// bar; every item dispatches a desktop action back onto this event loop, as
    /// bboxy450-term desktop_menu.rs does. Unavailable (no main thread, another OS): the
    /// keyboard shortcuts still work through `route_key`.
    fn install_menu(&mut self) {
        if self.menu.is_some() {
            return;
        }
        let proxy = self.proxy.clone();
        match bbt_desktop::MenuBar::install(Box::new(move |action| {
            let _ = proxy.send_event(Wake::Desktop(action));
        })) {
            Ok(menu) => self.menu = Some(menu),
            Err(error) => warn("desktop menu unavailable", error),
        }
    }
    fn paint(&mut self) -> Result<(), String> {
        let frame_start = Instant::now();
        self.view
            .anim
            .tick((frame_start - self.started).as_secs_f32() * 1000.);
        self.bridge.drain(Duration::from_micros(300));
        self.gpu_dirty.drain();
        if let Some(gpu) = self.gpu.take() {
            let gpu = gpu?;
            let size = self.window.as_ref().unwrap().inner_size();
            let mut config = gpu.config;
            config.width = size.width.max(1);
            config.height = size.height.max(1);
            gpu.surface.configure(&gpu.context.device, &config);
            self.materials = Some(m::Pass::new(&gpu.context.device, config.format));
            self.renderer = Some(
                ChromeRenderer::with_context(gpu.context, super::assets::icons())
                    .map_err(|e| e.to_string())?,
            );
            self.surface = Some(gpu.surface);
            self.config = Some(config);
        }
        self.gpu.complete_drain();
        let Some(config) = &self.config else {
            return Ok(());
        };
        let window = self.window.as_ref().unwrap();
        let scale = window.scale_factor() as f32;
        self.bridge.set_width(config.width as f32 / scale);
        // reduced motion, the flat fallback and occlusion freeze the rim shader's clock:
        // the rim's end state
        let animates = self.view.anim.animates();
        let time_ms = if animates { self.view.anim.now() } else { 0. };
        let frame = self.view.render(
            &self.bridge.state,
            [config.width as f32 / scale, config.height as f32 / scale],
        )?;
        let renderer = self.renderer.as_mut().unwrap();
        let surface = self.surface.as_ref().unwrap();
        // A hidden NSWindow is always occluded on macOS. Validate the real window,
        // surface configuration and live renderer, but encode into a private texture;
        // never order a smoke window onto the operator's desktop to get a drawable.
        let texture = if self.smoke_window {
            None
        } else {
            Some(match surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(v)
                | wgpu::CurrentSurfaceTexture::Suboptimal(v) => v,
                wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                    surface.configure(&renderer.context.device, config);
                    window.request_redraw();
                    return Ok(());
                }
                wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                    return Ok(());
                }
                wgpu::CurrentSurfaceTexture::Validation => {
                    return Err("surface validation failed".into());
                }
            })
        };
        if self
            .backdrop
            .as_ref()
            .is_none_or(|t| t.width() != config.width || t.height() != config.height)
        {
            self.backdrop = Some(backdrop(&renderer.context.device, config));
        }
        let pass = self.materials.as_ref().ok_or("material pass missing")?;
        let backdrop_view = self
            .backdrop
            .as_ref()
            .unwrap()
            .create_view(&Default::default());
        let smoke_target = self.smoke_window.then(|| {
            renderer
                .context
                .device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("private native startup frame"),
                    size: wgpu::Extent3d {
                        width: config.width,
                        height: config.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: config.format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &config.view_formats,
                })
        });
        let target_view = texture
            .as_ref()
            .map(|t| &t.texture)
            .or(smoke_target.as_ref())
            .ok_or("frame target missing")?
            .create_view(&Default::default());
        let target = || ChromeTarget {
            width: config.width,
            height: config.height,
            scale,
            view: &target_view,
            format: config.format,
        };
        renderer.begin_frame();
        pass.begin_frame();
        let mut encoder = renderer
            .context
            .device
            .create_command_encoder(&Default::default());
        // the face, then the alive rims through the material pass, then the menus and the
        // zoom pill over both (the order the term's host paints a block and its rim)
        renderer
            .draw(&frame.list, target(), &mut encoder)
            .map_err(|e| e.to_string())?;
        let alive = !frame.rims.is_empty();
        if alive {
            pass.draw(
                &renderer.context,
                &mut encoder,
                m::Frame {
                    target: target(),
                    backdrop: &backdrop_view,
                    items: &frame.rims,
                    time_ms,
                    theme: bbt_ui_model::Theme::Dark,
                    backdrop_key: None,
                },
            )
            .map_err(|e| e.to_string())?;
        }
        if !frame.over.primitives.is_empty() {
            renderer
                .draw_overlay(&frame.over, target(), &mut encoder)
                .map_err(|e| e.to_string())?;
        }
        pass.flush(&renderer.context);
        renderer.flush();
        renderer.context.queue.submit([encoder.finish()]);
        if let Some(texture) = texture {
            renderer.context.queue.present(texture);
        }
        self.rendered = true;
        if let Some(layout) = &frame.layout
            && let Ok(regions) = composer::Regions::new(layout)
            && let Some(caret) = regions.caret
        {
            window.set_ime_cursor_area(
                winit::dpi::LogicalPosition::new(caret.x, caret.y),
                LogicalSize::new(caret.w, caret.h),
            );
        }
        // The clock's word on the next frame: now while something moves, at a caret blink
        // edge otherwise, or never. The frame's cost counts against the 16.6 ms budget.
        self.view
            .anim
            .frame_cost(frame_start.elapsed().as_secs_f32() * 1000.);
        self.wake_at = match self.view.anim.next_frame() {
            Some(ms) if ms <= 0. => {
                window.request_redraw();
                None
            }
            Some(ms) => Some(Instant::now() + Duration::from_secs_f32(ms / 1000.)),
            None => None,
        };
        // The alive rim is a visible loop on the shader's clock (harness-spec.md Composer):
        // it draws every frame while the clock animates, and freezes under reduced motion,
        // the flat fallback and occlusion.
        if alive && animates {
            window.request_redraw();
            self.wake_at = None;
        }
        Ok(())
    }
    fn edit(&mut self, command: Edit) {
        self.view.edit(command);
        self.view.invalidate();
    }
    fn key(&mut self, key: Key, text: Option<&str>) {
        let before = self.view.zoom();
        route_key(&mut self.view, &self.bridge, key, text, self.modifiers);
        if self.view.zoom() != before {
            self.save_zoom();
        }
    }
    /// Persist the zoom (zoom.rs) off the UI thread; a failure is reported, never fatal.
    fn save_zoom(&self) {
        let Some(root) = self.settings_root.clone() else {
            return;
        };
        let zoom = self.view.zoom();
        bbt_rt::handle().spawn(async move {
            if let Err(error) = zoom::save(&root, zoom).await {
                warn("settings", error);
            }
        });
    }
    fn zoom_by(&mut self, delta: i8) {
        let next = self.view.zoom().step(delta);
        if self.view.set_zoom(next, &self.bridge) {
            self.save_zoom();
        }
    }
    /// A desktop action from the menu bar or a title-bar control, dispatched as the term's
    /// desktop_menu.rs does. The harness has one session: seats, splits and the settings
    /// surface have no counterpart here and are ignored.
    fn desktop_action(&mut self, action: Desktop, event_loop: &ActiveEventLoop) {
        match action {
            Desktop::IncreaseTerminalFont => self.zoom_by(1),
            Desktop::DecreaseTerminalFont => self.zoom_by(-1),
            Desktop::CloseWindow | Desktop::HardQuit => {
                self.bridge.request_detach();
                event_loop.exit();
            }
            Desktop::ShowWindow
            | Desktop::HideWindow
            | Desktop::ToggleFullscreen
            | Desktop::Minimize
            | Desktop::Zoom => {
                if let Some(window) = &self.window
                    && let Err(error) = bbt_desktop::apply_window_action(window.as_ref(), action)
                {
                    warn("desktop window", error);
                }
            }
            Desktop::Paste => match bbt_desktop::read_clipboard() {
                Ok(Some(ClipboardContent::Text(text))) => {
                    let text: String = text
                        .chars()
                        .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
                        .collect();
                    if !text.is_empty() {
                        self.edit(Edit::Insert(text));
                    }
                }
                Ok(_) => {}
                Err(error) => warn("clipboard", error),
            },
            Desktop::Copy => {
                if let Some(text) = self.view.composer.selected_text()
                    && let Err(error) = bbt_desktop::write_clipboard_text(&text)
                {
                    warn("clipboard", error);
                }
            }
            Desktop::NewTab
            | Desktop::SplitRight
            | Desktop::SplitDown
            | Desktop::Settings
            | Desktop::CloseFocusedSeat
            | Desktop::MuteFocusedSeat => {}
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
    /// A press on the drawn title bar (chrome.rs): a window control, the system's
    /// double-click action, a drag through the OS, or the theme control.
    fn titlebar_intent(&mut self, intent: Intent, event_loop: &ActiveEventLoop) {
        match intent {
            Intent::Window(action) => self.desktop_action(action, event_loop),
            Intent::Titlebar(action) => {
                if let Some(window) = &self.window
                    && let Err(error) = bbt_desktop::apply_titlebar_action(window.as_ref(), action)
                {
                    warn("desktop titlebar", error);
                }
            }
            Intent::Drag => {
                if let Some(window) = &self.window
                    && let Err(error) = window.drag_window()
                {
                    warn("desktop window drag", error);
                }
            }
            // The face has one theme (view::context). The control is drawn as the term's
            // bar has it; cycling the theme waits for a light face.
            Intent::Theme => {}
        }
    }
}
impl ApplicationHandler<Wake> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none()
            && let Err(error) = self.init(event_loop)
        {
            self.error = Some(error);
            event_loop.exit();
        }
    }
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Wake) {
        match event {
            Wake::Frame => {}
            Wake::Desktop(action) => self.desktop_action(action, event_loop),
            Wake::Zoom(zoom) => {
                self.view.restore_zoom(zoom, &self.bridge);
            }
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
    /// A timed wake the clock asked for (a caret blink edge) has come: draw that frame.
    fn new_events(&mut self, _: &ActiveEventLoop, cause: StartCause) {
        if matches!(cause, StartCause::ResumeTimeReached { .. })
            && let Some(window) = &self.window
        {
            self.wake_at = None;
            window.request_redraw();
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.smoke_window {
            if let Err(error) = self.paint() {
                self.error = Some(error);
                event_loop.exit();
                return;
            }
            if self.rendered && self.bridge.state.ready && self.bridge.state.prepared.is_some() {
                if self.window.as_ref().and_then(|w| w.is_visible()) != Some(false) {
                    self.error = Some("smoke window must stay hidden".into());
                } else {
                    eprintln!(
                        "smoke-window: native window created; surface configured; live frame submitted to private texture; visible=false"
                    );
                }
                self.bridge.request_detach();
                event_loop.exit();
            } else {
                event_loop.set_control_flow(ControlFlow::WaitUntil(
                    Instant::now() + Duration::from_millis(16),
                ));
            }
            return;
        }
        event_loop.set_control_flow(match self.wake_at {
            Some(at) => ControlFlow::WaitUntil(at),
            None => ControlFlow::Wait,
        });
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.bridge.request_detach();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = self.paint() {
                    self.error = Some(error);
                    event_loop.exit();
                }
                return;
            }
            // Loops pause while the window is occluded (harness-spec.md Frame budget).
            WindowEvent::Occluded(occluded) => {
                self.view.anim.pause(occluded);
                if occluded {
                    self.wake_at = None;
                    return;
                }
            }
            WindowEvent::Resized(size) if size.width > 0 && size.height > 0 => {
                if let (Some(config), Some(surface), Some(renderer)) =
                    (&mut self.config, &self.surface, &self.renderer)
                {
                    config.width = size.width;
                    config.height = size.height;
                    surface.configure(&renderer.context.device, config);
                }
                self.view.invalidate();
            }
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::Focused(focused) => {
                self.view.focused = focused;
                self.view.invalidate();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let scale = self.window.as_ref().unwrap().scale_factor() as f32;
                self.pointer = [position.x as f32 / scale, position.y as f32 / scale];
                // a drag cannot complete a title-bar double-click
                self.view.titlebar.moved(self.pointer);
                // menu rows take a hover fill (harness-spec.md Motion: hover)
                self.view.pointer(self.pointer);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -y * 24.,
                    MouseScrollDelta::PixelDelta(p) => {
                        -p.y as f32 / self.window.as_ref().unwrap().scale_factor() as f32
                    }
                };
                self.view.wheel(self.pointer, dy);
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                let contains = |r: bbt_ui_model::layout::Rect| {
                    self.pointer[0] >= r.x
                        && self.pointer[0] < r.right()
                        && self.pointer[1] >= r.y
                        && self.pointer[1] < r.bottom()
                };
                // the drawn title bar first: its controls, double-click and drag
                let intent = self.view.chrome.then(|| {
                    self.view.titlebar.press(
                        self.pointer,
                        Instant::now(),
                        bbt_desktop::double_click_interval(),
                        bbt_desktop::titlebar_action(),
                    )
                });
                let action = self
                    .view
                    .frame
                    .actions
                    .iter()
                    .find(|(r, _)| contains(*r))
                    .map(|(_, a)| a.clone());
                if let Some(Some(intent)) = intent {
                    self.titlebar_intent(intent, event_loop);
                } else if let Some(action) = action {
                    self.view.activate(action, &self.bridge);
                } else if let Some(layout) = &self.view.frame.layout
                    && let Ok(regions) = composer::Regions::new(layout)
                {
                    match regions.hit(self.pointer) {
                        Some(composer::Hit::Send) => {
                            let action = if self.view.send_stops(&self.bridge.state) {
                                Action::Interrupt
                            } else {
                                Action::Send
                            };
                            self.view.activate(action, &self.bridge)
                        }
                        Some(composer::Hit::Body) => {
                            self.view
                                .composer
                                .handle(composer::Command::Focus(composer::Field::Body));
                            self.view.invalidate();
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                self.key(event.logical_key, event.text.as_deref())
            }
            WindowEvent::Ime(Ime::Commit(text)) => self.edit(Edit::Commit(text)),
            WindowEvent::Ime(Ime::Preedit(text, cursor)) => {
                let caret = cursor
                    .and_then(|(at, _)| text.get(..at))
                    .map(|prefix| prefix.encode_utf16().count());
                self.edit(Edit::Preedit { text, caret });
            }
            WindowEvent::Ime(Ime::Disabled) => self.edit(Edit::CancelPreedit),
            _ => return,
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
fn config(options: &Options) -> Result<Config, String> {
    config_at_home(options, std::env::var_os("HOME"))
}
fn config_at_home(options: &Options, home: Option<std::ffi::OsString>) -> Result<Config, String> {
    let profiles = super::bridge::machine_profiles(home)
        .ok_or("Profile catalogue unavailable · HOME is absent")?;
    let root = profiles.parent().expect("profiles root").to_path_buf();
    let session = SessionId::new();
    let redactor = Set::default();
    Ok(Config {
        meta: SessionMeta {
            source: None,
            session,
            cwd: options.cwd.clone(),
            profile: Some(options.profile.clone()),
            fork: None,
            protocol: PROTOCOL,
            harness: env!("CARGO_PKG_VERSION").into(),
            created_ms: crate::store::now_ms(),
        },
        sessions: root.join("sessions"),
        profiles: root.join("profiles"),
        model: "unconfigured".into(),
        provider: Arc::new(FailedProvider(ProviderError::new(
            ErrorClass::Config,
            "profile not loaded",
        ))),
        profile_error: Some("profile not loaded".into()),
        policy: Policy::new(Unattended::Deny),
        redactor,
        max_rounds: 50,
    })
}
/// Returns the detached background drain to main, after all UI callbacks have
/// ended. Main keeps the process alive until already-admitted turns finish.
pub fn run(options: Options) -> Result<Option<tokio::task::JoinHandle<()>>, String> {
    let smoke_window = std::env::var("BBOXY450_HARNESS_SMOKE_WINDOW").as_deref() == Ok("1");
    let mut builder = EventLoop::<Wake>::with_user_event();
    #[cfg(target_os = "macos")]
    if smoke_window {
        use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
        builder
            .with_activation_policy(ActivationPolicy::Prohibited)
            .with_activate_ignoring_other_apps(false)
            .with_default_menu(false);
    }
    let event_loop = builder.build().map_err(|e| e.to_string())?;
    bbt_rt::register_ui_thread();
    let proxy = event_loop.create_proxy();
    let waker = Arc::new(WakePort(proxy.clone()));
    let gpu_dirty = DirtySet::new(1, waker.clone());
    let gpu = Arc::new(Latest::new(gpu_dirty.paint_source(0)));
    let mut view = HarnessView::standalone(Zoom::default());
    // the window animates; reduced motion (the OS setting or BBOXY_REDUCED_MOTION=1) draws
    // end states and keeps only the caret blink
    view.anim = super::anim::Clock::live(super::anim::Clock::reduced_motion_preferred());
    let settings_root =
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".bboxy450-harness"));
    if let Some(root) = settings_root.clone() {
        // the saved zoom, read off the UI thread and applied through the event loop
        let proxy = proxy.clone();
        bbt_rt::handle().spawn(async move {
            let zoom = zoom::load(&root).await;
            let _ = proxy.send_event(Wake::Zoom(zoom));
        });
    }
    let mut app = App {
        options,
        smoke_window,
        rendered: false,
        bridge: Bridge::new(waker),
        view,
        window: None,
        surface: None,
        renderer: None,
        config: None,
        materials: None,
        backdrop: None,
        gpu,
        gpu_dirty,
        pointer: [0.; 2],
        modifiers: ModifiersState::empty(),
        error: None,
        started: Instant::now(),
        wake_at: None,
        proxy,
        menu: None,
        settings_root,
    };
    event_loop.run_app(&mut app).map_err(|e| e.to_string())?;
    drop(app.menu.take());
    let job = app.bridge.detach();
    if let Some(error) = app.error {
        return Err(error);
    }
    Ok(job)
}

/// Keyboard control-lane routing shared by the host and headless witness.
pub fn is_interrupt(key: &Key, modifiers: ModifiersState) -> bool {
    *key == Key::Named(NamedKey::Escape)
        || (modifiers.control_key()
            && matches!(key, Key::Character(s) if s.eq_ignore_ascii_case("c")))
}

/// Cmd+= / Cmd++ zoom in, Cmd+- zooms out, Cmd+0 resets (harness-spec.md Standalone window).
/// The term's menu items carry the same keys; whichever side AppKit hands the press to,
/// one step happens. Follows the active layout as bboxy450-term main.rs does: plus can be
/// a shifted `=` or its own character.
pub fn zoom_key(key: &Key, modifiers: ModifiersState) -> Option<i8> {
    if !(modifiers.super_key() || modifiers.control_key()) || modifiers.alt_key() {
        return None;
    }
    match key {
        Key::Character(s) if s == "=" || s == "+" => Some(1),
        Key::Character(s) if s == "-" => Some(-1),
        Key::Character(s) if s == "0" => Some(0),
        _ => None,
    }
}

/// The same key routing used by the native event handler, independently of window creation.
pub fn route_key(
    view: &mut HarnessView,
    bridge: &Bridge,
    key: Key,
    text: Option<&str>,
    modifiers: ModifiersState,
) {
    let control = modifiers.control_key();
    let shortcut = control || modifiers.super_key();
    let shift = modifiers.shift_key();
    // The open profile switcher takes keys first: the menu floats over everything.
    if view.profile_menu_open() {
        use super::view::MenuKey;
        let menu_key = match &key {
            Key::Named(NamedKey::Escape) => {
                view.toggle_profile_menu();
                return;
            }
            Key::Named(NamedKey::ArrowUp) => Some(MenuKey::Up),
            Key::Named(NamedKey::ArrowDown | NamedKey::Tab) => Some(MenuKey::Down),
            Key::Named(NamedKey::Enter) => Some(MenuKey::Enter),
            Key::Named(NamedKey::Backspace) => Some(MenuKey::Backspace),
            Key::Character(_) | Key::Named(NamedKey::Space) if !shortcut => text
                .filter(|t| !t.chars().any(char::is_control))
                .map(|t| MenuKey::Text(t.into())),
            _ => None,
        };
        if let Some(menu_key) = menu_key {
            view.profile_menu_key(menu_key, bridge);
            return;
        }
    }
    if let Some(step) = zoom_key(&key, modifiers) {
        let zoom = if step == 0 {
            Zoom::default()
        } else {
            view.zoom().step(step)
        };
        view.set_zoom(zoom, bridge);
        return;
    }
    if key == Key::Named(NamedKey::Escape) && view.cancel_prompt(bridge) {
        return;
    }
    let choice = match key {
        Key::Named(NamedKey::ArrowUp) => Some(bbt_ui::modules::approval::Key::Up),
        Key::Named(NamedKey::ArrowDown) => Some(bbt_ui::modules::approval::Key::Down),
        Key::Named(NamedKey::Home) => Some(bbt_ui::modules::approval::Key::Home),
        Key::Named(NamedKey::End) => Some(bbt_ui::modules::approval::Key::End),
        Key::Named(NamedKey::Enter) => Some(bbt_ui::modules::approval::Key::Enter),
        _ => None,
    };
    if let Some(choice) = choice {
        if view.approval_key(choice, bridge) {
            return;
        }
        if matches!(key, Key::Named(NamedKey::ArrowUp | NamedKey::ArrowDown))
            && view.complete(choice)
        {
            return;
        }
    }
    if key == Key::Named(NamedKey::Enter) && view.run_completion(bridge) {
        return;
    }
    if is_interrupt(&key, modifiers) {
        view.activate(Action::Interrupt, bridge);
        return;
    }
    match key {
        Key::Named(NamedKey::Tab) => {
            view.complete(bbt_ui::modules::approval::Key::Enter);
        }
        Key::Named(NamedKey::ArrowUp | NamedKey::ArrowDown) => {
            view.composer.vertical(key == Key::Named(NamedKey::ArrowUp));
            view.invalidate();
        }
        Key::Named(NamedKey::Enter) if !shift && !view.composer.composing() => {
            view.activate(Action::Send, bridge)
        }
        Key::Named(NamedKey::Enter) if !view.composer.composing() => {
            view.edit(Edit::Insert("\n".into()))
        }
        Key::Named(NamedKey::Backspace) => view.edit(Edit::Delete { backwards: true }),
        Key::Named(NamedKey::Delete) => view.edit(Edit::Delete { backwards: false }),
        Key::Named(
            k @ (NamedKey::ArrowLeft | NamedKey::ArrowRight | NamedKey::Home | NamedKey::End),
        ) => {
            let motion = match k {
                NamedKey::ArrowLeft => Motion::Left,
                NamedKey::ArrowRight => Motion::Right,
                NamedKey::Home => Motion::Start,
                _ => Motion::End,
            };
            view.edit(Edit::Move {
                motion,
                extend: shift,
            });
        }
        Key::Character(ref s) if shortcut && s.eq_ignore_ascii_case("a") => {
            view.edit(Edit::SelectAll)
        }
        Key::Character(_) | Key::Named(NamedKey::Space)
            if !shortcut && !view.composer.composing() =>
        {
            if let Some(text) = text
                && !text.chars().any(char::is_control)
            {
                view.edit(Edit::Insert(text.into()));
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod acp_controls_path_tests {
    use super::*;
    #[test]
    fn acp_controls_standalone_uses_same_machine_catalogue_as_embedded() {
        let options = Options::parse(Vec::<String>::new()).unwrap();
        let private = PathBuf::from("/private/tmp/bboxy-private-home");
        let cfg = config_at_home(&options, Some(private.clone().into_os_string())).unwrap();
        assert_eq!(
            cfg.profiles,
            super::super::bridge::machine_profiles(Some(private.into_os_string())).unwrap()
        );
        assert!(config_at_home(&options, None).is_err());
        assert!(config_at_home(&options, Some("".into())).is_err());
    }
}
