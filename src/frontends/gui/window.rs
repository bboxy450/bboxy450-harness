// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

// Adapted from bboxy450-term crates/bboxybook/src/window.rs: winit host and
// ordered ChromeRenderer surface pass. GPU setup is moved to bbt-rt.
// https://github.com/bboxy450/bboxy450-term/blob/main/ATTRIBUTION.md
#![deny(clippy::disallowed_methods)]
use super::{
    bridge::Bridge,
    options::Options,
    view::{Action, HarnessView},
};
use crate::{
    engine::Config,
    policy::Policy,
    protocol::*,
    provider::{FailedProvider, ProviderError, profile::Unattended},
    redact::Set,
};
use bbt_render::{
    chrome::{ChromeRenderer, ChromeTarget},
    device::DeviceContext,
};
use bbt_rt::{DirtySet, Latest, Waker};
use bbt_ui::{
    modules::composer,
    text_edit::{Command as Edit, Motion},
};
use std::{path::PathBuf, sync::Arc, time::Duration};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    keyboard::{Key, ModifiersState, NamedKey},
    window::{Window, WindowId},
};
#[derive(Debug)]
struct Wake;
struct WakePort(EventLoopProxy<Wake>);
impl Waker for WakePort {
    fn wake(&self) {
        let _ = self.0.send_event(Wake);
    }
}
struct Gpu {
    surface: wgpu::Surface<'static>,
    context: DeviceContext,
    config: wgpu::SurfaceConfiguration,
}
struct App {
    options: Options,
    bridge: Bridge,
    view: HarnessView,
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    renderer: Option<ChromeRenderer>,
    config: Option<wgpu::SurfaceConfiguration>,
    gpu: Arc<Latest<Result<Gpu, String>>>,
    gpu_dirty: DirtySet,
    pointer: [f32; 2],
    modifiers: ModifiersState,
    error: Option<String>,
}
impl App {
    fn init(&mut self, event_loop: &ActiveEventLoop) -> Result<(), String> {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("bboxy450-harness")
                        .with_inner_size(LogicalSize::new(1120., 860.)),
                )
                .map_err(|e| e.to_string())?,
        );
        window.set_ime_allowed(true);
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
                None => self.bridge.create(config(&self.options)?),
            }
        }
        self.window = Some(window);
        Ok(())
    }
    fn paint(&mut self) -> Result<(), String> {
        self.bridge.drain(Duration::from_micros(300));
        self.gpu_dirty.drain();
        if let Some(gpu) = self.gpu.take() {
            let gpu = gpu?;
            let size = self.window.as_ref().unwrap().inner_size();
            let mut config = gpu.config;
            config.width = size.width.max(1);
            config.height = size.height.max(1);
            gpu.surface.configure(&gpu.context.device, &config);
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
        let frame = self.view.render(
            &self.bridge.state,
            [config.width as f32 / scale, config.height as f32 / scale],
        )?;
        let renderer = self.renderer.as_mut().unwrap();
        let surface = self.surface.as_ref().unwrap();
        let texture = match surface.get_current_texture() {
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
        };
        let target = texture.texture.create_view(&Default::default());
        renderer.begin_frame();
        let mut encoder = renderer
            .context
            .device
            .create_command_encoder(&Default::default());
        renderer
            .draw(
                &frame.list,
                ChromeTarget {
                    width: config.width,
                    height: config.height,
                    scale,
                    view: &target,
                    format: config.format,
                },
                &mut encoder,
            )
            .map_err(|e| e.to_string())?;
        renderer.flush();
        renderer.context.queue.submit([encoder.finish()]);
        renderer.context.queue.present(texture);
        if let Some(layout) = &frame.layout
            && let Ok(regions) = composer::Regions::new(layout)
            && let Some(caret) = regions.caret
        {
            window.set_ime_cursor_area(
                winit::dpi::LogicalPosition::new(caret.x, caret.y),
                LogicalSize::new(caret.w, caret.h),
            );
        }
        Ok(())
    }
    fn edit(&mut self, command: Edit) {
        self.view.edit(command);
        self.view.invalidate();
    }
    fn key(&mut self, key: Key, text: Option<&str>) {
        route_key(&mut self.view, &self.bridge, key, text, self.modifiers);
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
    fn user_event(&mut self, _: &ActiveEventLoop, _: Wake) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
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
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -y * 24.,
                    MouseScrollDelta::PixelDelta(p) => {
                        -p.y as f32 / self.window.as_ref().unwrap().scale_factor() as f32
                    }
                };
                if self
                    .view
                    .scroll
                    .wheel(self.view.frame.scroll, self.view.frame.max_scroll, dy)
                {
                    self.view.invalidate();
                }
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
                let action = self
                    .view
                    .frame
                    .actions
                    .iter()
                    .find(|(r, _)| contains(*r))
                    .map(|(_, a)| a.clone());
                if let Some(action) = action {
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
    let root = PathBuf::from(std::env::var_os("HOME").ok_or("HOME unavailable")?)
        .join(".bboxy450-harness");
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
    let event_loop = EventLoop::<Wake>::with_user_event()
        .build()
        .map_err(|e| e.to_string())?;
    bbt_rt::register_ui_thread();
    let waker = Arc::new(WakePort(event_loop.create_proxy()));
    let gpu_dirty = DirtySet::new(1, waker.clone());
    let gpu = Arc::new(Latest::new(gpu_dirty.paint_source(0)));
    let mut app = App {
        options,
        bridge: Bridge::new(waker),
        view: HarnessView::default(),
        window: None,
        surface: None,
        renderer: None,
        config: None,
        gpu,
        gpu_dirty,
        pointer: [0.; 2],
        modifiers: ModifiersState::empty(),
        error: None,
    };
    event_loop.run_app(&mut app).map_err(|e| e.to_string())?;
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
            Key::Character(_) if !shortcut => text
                .filter(|t| !t.chars().any(char::is_control))
                .map(|t| MenuKey::Text(t.into())),
            _ => None,
        };
        if let Some(menu_key) = menu_key {
            view.profile_menu_key(menu_key, bridge);
            return;
        }
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
        Key::Character(_) if !shortcut && !view.composer.composing() => {
            if let Some(text) = text
                && !text.chars().any(char::is_control)
            {
                view.edit(Edit::Insert(text.into()));
            }
        }
        _ => {}
    }
}
