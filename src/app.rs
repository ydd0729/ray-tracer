use crate::gui::Gui;
use crate::rendering::wgpu::{Textures, Wgpu};
use crate::rendering::RenderingConfiguration;
use crate::time;
use cfg_if::cfg_if;
use std::cell::{Ref, RefCell, RefMut};
use std::sync::Arc;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use wasm_bindgen::prelude::*;
    }
}

#[derive(Default)]
pub struct App {
    last_frame_time: Option<time::Instant>,
    // 必须使用 Arc ，因为 wgpu::Instance::create_surface 要求实现 Send 和 Sync
    window: Option<Arc<winit::window::Window>>,
    wgpu: Option<RefCell<Wgpu>>,
    #[cfg(target_arch = "wasm32")]
    wgpu_context_receiver: Option<futures::channel::oneshot::Receiver<Wgpu>>,
    textures: Option<RefCell<Textures>>,
    rendering_configuration: RenderingConfiguration,
    gui: Option<RefCell<Gui>>,
}

impl App {
    pub fn run() {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
            } else {
                log4rs::init_file("log4rs.yml", Default::default()).unwrap();
            }
        }

        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let mut app = App::default();
        event_loop.run_app(&mut app).expect("panic");
    }

    fn resize(&mut self, size: &winit::dpi::PhysicalSize<u32>) {
        self.wgpu_mut().on_resize(size);
        self.rendering_context_mut().on_resize(size, self.wgpu());
    }

    fn update(&mut self) {
        let delta_time = self.update_delta_time();
        let window = self.window();

        self.ui_mut().update(window, delta_time);
    }

    fn render(&mut self) {
        let surface_texture = match self.wgpu().surface.get_current_texture() {
            Ok(surface_texture) => surface_texture,
            Err(e) => {
                eprintln!("dropped frame: {e:?}");
                return;
            }
        };

        let surface_view = surface_texture.texture.create_view(&Default::default());

        let mut encoder = self
            .wgpu()
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        if self.rendering_configuration.msaa {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self
                        .rendering_context_mut()
                        .create_multisampled_texture_view(&surface_texture, self.wgpu()),
                    resolve_target: Some(&surface_view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                label: Some("egui main render pass"),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            drop(render_pass);
        }

        self.ui_mut().render(self.wgpu(), &surface_view);

        surface_texture.present();
    }
}

impl winit::application::ApplicationHandler for App {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {}

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut attributes = winit::window::Window::default_attributes();

        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                use winit::platform::web::WindowAttributesExtWebSys;

                let mut width = 0;
                let mut height = 0;
                let canvas = web_sys::window().and_then(|window| {
                    let device_pixel_ratio = window.device_pixel_ratio();
                    let window_width = window.inner_width().unwrap().as_f64().unwrap();
                    let window_height = window.inner_height().unwrap().as_f64().unwrap();
                    width = (window_width * device_pixel_ratio) as u32;
                    height = (window_height * device_pixel_ratio) as u32;
                    window.document()
                }).and_then(|document|                            {
                            let canvas = document
                                .create_element("canvas").unwrap()
                                .dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
                            canvas.set_id("canvas");
                            document.body().unwrap().append_child(&canvas).expect("panic");

                            Some(canvas)
                        }
                    );

                attributes = attributes
                    .with_canvas(canvas)
                    .with_inner_size(winit::dpi::PhysicalSize::new(width, height));
            } else {
                let width = 1280;
                let height = 720;
                let version = env!("CARGO_PKG_VERSION");
                attributes = attributes.with_inner_size(winit::dpi::LogicalSize::new(width, height))
                                       .with_title(format!("renderer {version}"));
            }
        }

        let window = Arc::new(event_loop.create_window(attributes).unwrap());
        let resumed_first_time = self.window.is_none();
        self.window = Some(window.clone());

        if resumed_first_time {
            cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    let (sender, receiver) =
                    futures::channel::oneshot::channel();
                    self.wgpu_context_receiver = Some(receiver);
                    wasm_bindgen_futures::spawn_local(async move {
                        let renderer = Wgpu::new(window.clone()).await;
                        if sender.send(renderer).is_err() {
                            log::error!("Failed to create and send renderer!");
                        }
                    });
                } else {
                    let wgpu_context = pollster::block_on(Wgpu::new(window.clone()));
                    self.gui = Some(RefCell::new(Gui::new(
                        window.as_ref(),
                        &wgpu_context.device,
                        wgpu_context.surface_configuration.format)
                    ));
                    self.wgpu = Some(RefCell::new(wgpu_context));
                    self.textures = Some(RefCell::new(Textures::new(
                        window, self.wgpu()
                    )));

                }
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: ()) {}

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: winit::window::WindowId, event: WindowEvent) {
        if !self.try_receive() {
            return;
        }

        if window_id == self.window().id() {
            // Receive gui window event
            if self.ui_mut().on_window_event(self.window().as_ref(), &event).consumed {
                return;
            }

            match &event {
                WindowEvent::ActivationTokenDone { .. } => {}
                WindowEvent::Resized(size) => {
                    self.resize(size);
                }
                WindowEvent::Moved(_) => {}
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::Destroyed => {}
                WindowEvent::DroppedFile(_) => {}
                WindowEvent::HoveredFile(_) => {}
                WindowEvent::HoveredFileCancelled => {}
                WindowEvent::Focused(_) => {}
                WindowEvent::KeyboardInput {
                    event:
                        winit::event::KeyEvent {
                            physical_key: winit::keyboard::PhysicalKey::Code(key_code),
                            ..
                        },
                    ..
                } => {
                    // Exit by pressing the escape key
                    if matches!(key_code, winit::keyboard::KeyCode::Escape) {
                        event_loop.exit();
                    }
                }
                WindowEvent::ModifiersChanged(_) => {}
                WindowEvent::Ime(_) => {}
                WindowEvent::CursorMoved { .. } => {}
                WindowEvent::CursorEntered { .. } => {}
                WindowEvent::CursorLeft { .. } => {}
                WindowEvent::MouseWheel { .. } => {}
                WindowEvent::MouseInput { .. } => {}
                WindowEvent::PinchGesture { .. } => {}
                WindowEvent::PanGesture { .. } => {}
                WindowEvent::DoubleTapGesture { .. } => {}
                WindowEvent::RotationGesture { .. } => {}
                WindowEvent::TouchpadPressure { .. } => {}
                WindowEvent::AxisMotion { .. } => {}
                WindowEvent::Touch(_) => {}
                WindowEvent::ScaleFactorChanged { .. } => {}
                WindowEvent::ThemeChanged(_) => {}
                WindowEvent::Occluded(_) => {}
                WindowEvent::RedrawRequested => {
                    self.update();
                    self.render();
                    self.window().request_redraw();
                }
                _ => {}
            }
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        match &event {
            DeviceEvent::Added => {}
            DeviceEvent::Removed => {}
            DeviceEvent::MouseMotion { .. } => {}
            DeviceEvent::MouseWheel { .. } => {}
            DeviceEvent::Motion { .. } => {}
            DeviceEvent::Button { .. } => {}
            DeviceEvent::Key(_) => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.try_receive();
        self.window().request_redraw();
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {}

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {}

    fn memory_warning(&mut self, _event_loop: &ActiveEventLoop) {}
}

impl App {
    fn try_receive(&mut self) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            let mut receive_success = false;
            if let Some(receiver) = self.wgpu_context_receiver.as_mut() {
                if let Ok(Some(wgpu_context)) = receiver.try_recv() {
                    self.gui = Some(RefCell::new(Gui::new(
                        self.window().as_ref(),
                        &wgpu_context.device,
                        wgpu_context.surface_configuration.format,
                    )));
                    self.wgpu = Some(RefCell::new(wgpu_context));
                    self.textures = Some(RefCell::new(Textures::new(self.window(), self.wgpu())));

                    receive_success = true;
                }
            }

            if receive_success {
                self.wgpu_context_receiver = None;
            }
        }

        self.wgpu.is_some()
    }

    fn update_delta_time(&mut self) -> time::Duration {
        let now = time::Instant::now();
        let delta_time = match &self.last_frame_time {
            None => time::Duration::from_secs(0),
            Some(last_frame_time) => now - *last_frame_time,
        };
        self.last_frame_time = Some(now);

        delta_time
    }

    fn window(&self) -> Arc<winit::window::Window> {
        self.window.as_ref().unwrap().clone()
    }
    // fn ui(&self) -> Ref<'_, Gui> {
    //     self.gui.as_ref().unwrap().borrow()
    // }
    fn ui_mut(&self) -> RefMut<'_, Gui> {
        self.gui.as_ref().unwrap().borrow_mut()
    }
    fn wgpu(&self) -> Ref<'_, Wgpu> {
        self.wgpu.as_ref().unwrap().borrow()
    }
    fn wgpu_mut(&self) -> RefMut<'_, Wgpu> {
        self.wgpu.as_ref().unwrap().borrow_mut()
    }
    // fn rendering_context(&self) -> Ref<'_, Textures> {
    //     self.textures.as_ref().unwrap().borrow()
    // }
    fn rendering_context_mut(&self) -> RefMut<'_, Textures> {
        self.textures.as_ref().unwrap().borrow_mut()
    }
}
