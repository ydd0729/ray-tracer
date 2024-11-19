use crate::ui::Ui;
use crate::wgpu_context::WgpuContext;
use std::cell::{Ref, RefCell, RefMut};
use std::sync::Arc;
use std::time;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;

pub struct App {
    // 必须使用 Arc ，因为 wgpu::Instance::create_surface 要求实现 Send 和 Sync
    window: Option<Arc<winit::window::Window>>,
    wgpu_context: Option<RefCell<WgpuContext>>,
    last_frame_time: time::Instant,
    ui: Option<RefCell<Ui>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            wgpu_context: None,
            ui: None,
            last_frame_time: time::Instant::now(),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
impl App {
    pub fn run() {
        // cfg_if::cfg_if! {
        //     if #[cfg(target_arch = "wasm32")] {
        //         std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        //         console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
        //     } else {
        //         env_logger::init();
        //     }
        // }

        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let mut app = App::default();
        event_loop.run_app(&mut app).expect("panic");
    }

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
        self.wgpu_mut().resize(size);
    }

    fn update(&mut self) {
        let delta_time = self.last_frame_time.elapsed();
        let window = self.window();

        self.ui_mut().update(window, &delta_time);

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
            surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.ui_mut().render(self.wgpu(), &surface_view, true);

        surface_texture.present();
    }
}

impl winit::application::ApplicationHandler for App {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {}

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let version = env!("CARGO_PKG_VERSION");
        let attributes = winit::window::Window::default_attributes()
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .with_title(&format!("renderer {version}"));

        let window = Arc::new(event_loop.create_window(attributes).unwrap());

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;

            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wasm")?;
                    let canvas = web_sys::Element::from(window.canvas()?);
                    dst.append_child(&canvas).ok()?;
                    canvas.set_id("wgpu_canvas");

                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }

        let wgpu_context = WgpuContext::new(window.clone());

        self.ui = Some(RefCell::new(Ui::new(&window, &wgpu_context)));
        self.last_frame_time = time::Instant::now();
        self.window = Some(window.clone());
        self.wgpu_context = Some(RefCell::new(wgpu_context));
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ()) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if window_id == self.window.as_ref().unwrap().id() {
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
                WindowEvent::KeyboardInput { .. } => {}
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
                    self.window.as_ref().unwrap().request_redraw();
                }
            }
        }

        let window = self.window().clone();
        self.ui_mut().window_event(window, event_loop, window_id, event);
    }

    fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) {}

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {}

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {}

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {}

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {}
}
