use crate::time;
use crate::ui::Ui;
use crate::wgpu_context::WgpuContext;
use cfg_if::cfg_if;
use std::cell::{Ref, RefCell, RefMut};
use std::sync::Arc;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use wasm_bindgen::prelude::*;
    } else {

    }
}

pub struct App {
    // 必须使用 Arc ，因为 wgpu::Instance::create_surface 要求实现 Send 和 Sync
    window: Option<Arc<winit::window::Window>>,
    wgpu_context: Option<RefCell<WgpuContext>>,
    last_frame_time: time::Instant,
    ui: Option<RefCell<Ui>>,
    #[cfg(target_arch = "wasm32")]
    wgpu_context_receiver: Option<futures::channel::oneshot::Receiver<WgpuContext>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            wgpu_context: None,
            ui: None,
            last_frame_time: time::Instant::now(),
            #[cfg(target_arch = "wasm32")]
            wgpu_context_receiver: None,
        }
    }
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

    fn try_receive(&mut self) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            let mut receive_success = false;
            if let Some(receiver) = self.wgpu_context_receiver.as_mut() {
                if let Ok(Some(wgpu_context)) = receiver.try_recv() {
                    self.ui = Some(RefCell::new(Ui::new(
                        self.window().as_ref(),
                        &wgpu_context.device,
                        wgpu_context.surface_configuration.format,
                        false,
                    )));
                    self.wgpu_context = Some(RefCell::new(wgpu_context));
                    receive_success = true;
                }
            }

            if receive_success {
                self.wgpu_context_receiver = None;
            }
        }

        self.wgpu_context.is_some()
    }

    #[allow(dead_code)]
    fn ui(&self) -> Ref<'_, Ui> {
        self.ui.as_ref().unwrap().borrow()
    }

    fn ui_mut(&self) -> RefMut<'_, Ui> {
        self.ui.as_ref().unwrap().borrow_mut()
    }

    fn window(&self) -> Arc<winit::window::Window> {
        self.window.as_ref().unwrap().clone()
    }

    fn wgpu(&self) -> Ref<'_, WgpuContext> {
        self.wgpu_context.as_ref().unwrap().borrow()
    }

    fn wgpu_mut(&self) -> RefMut<'_, WgpuContext> {
        self.wgpu_context.as_ref().unwrap().borrow_mut()
    }

    fn resize(&mut self, size: &winit::dpi::PhysicalSize<u32>) {
        self.wgpu_mut().resize(size, self.window());
    }

    fn render_update(&mut self) {
        let delta_time = self.last_frame_time.elapsed();
        let window = self.window();

        self.ui_mut().render_update(window, delta_time);

        self.last_frame_time = time::Instant::now();
    }

    fn render(&mut self) {
        let surface_texture = match self.wgpu().surface.get_current_texture() {
            Ok(surface_texture) => surface_texture,
            Err(e) => {
                eprintln!("dropped frame: {e:?}");
                return;
            }
        };
        let surface_view =
            surface_texture.texture.create_view(&wgpu::TextureViewDescriptor {
                label: wgpu::Label::default(),
                aspect: wgpu::TextureAspect::default(),
                format: Some(self.wgpu().surface_configuration.format),
                dimension: None,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });

        self.ui_mut().render(self.wgpu(), &surface_texture, &surface_view);

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
                let mut device_pixel_ratio = 1.0;
                let canvas = web_sys::window().and_then(|win| {
                    device_pixel_ratio = win.device_pixel_ratio();
                    width = (win.inner_width().unwrap().as_f64().unwrap() * device_pixel_ratio) as u32;
                    height = (win.inner_height().unwrap().as_f64().unwrap() * device_pixel_ratio) as u32;
                    win.document()
                })
                    .and_then(|doc|
                        {
                            let canvas = doc
                                .create_element("canvas").unwrap()
                                .dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
                            canvas.set_id("canvas");
                            doc.body().unwrap().append_child(&canvas).expect("panic");

                            Some(canvas)
                        }
                    );

                attributes = attributes.with_canvas(canvas).with_inner_size(winit::dpi::PhysicalSize::new(width, height));
            } else {
                let width = 1280;
                let height = 720;
                let version = env!("CARGO_PKG_VERSION");
                attributes = attributes.with_inner_size(winit::dpi::LogicalSize::new(width, height))
                                       .with_title(&format!("renderer {version}"));
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
                        let renderer = WgpuContext::new(window.clone()).await;
                        if sender.send(renderer).is_err() {
                            log::error!("Failed to create and send renderer!");
                        }
                    });
                } else {
                    let wgpu_context = pollster::block_on(WgpuContext::new(window.clone()));
                    self.ui = Some(RefCell::new(Ui::new(
                        window.as_ref(), &wgpu_context.device, wgpu_context.surface_configuration.format, false)));
                    self.wgpu_context = Some(RefCell::new(wgpu_context));
                }
            }
        }

        self.last_frame_time = time::Instant::now();
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: ()) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
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
                WindowEvent::Resized(size) => { self.resize(&size); }
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
                    event: winit::event::KeyEvent {
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
                    self.render_update();
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
