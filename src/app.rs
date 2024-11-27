pub mod camera;
pub mod egui_renderer;
pub mod gui_state;
mod renderer;
mod scene;

use crate::app::camera::CameraParameter;
use crate::app::gui_state::GuiState;
use crate::app::renderer::{Renderer, RendererParameters};
use crate::app::scene::Scene;
use crate::math::UNIT_Y;
use crate::rendering::primitive::PrimitiveProvider;
use crate::rendering::wgpu::{Wgpu, WgpuTexture, WgpuTextureBindingInstruction, WgpuTextureBindingType};
use crate::time;
use camera::Camera;
use cfg_if::cfg_if;
use getset::Getters;
use log::info;
use nalgebra::Point4;
use std::cell::{Ref, RefCell, RefMut};
use std::sync::Arc;
use wgpu::{ShaderStages, TextureSampleType};
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use wasm_bindgen::prelude::*;
    }
}

#[derive(Default, Getters)]
pub struct App {
    last_frame_time: Option<time::Instant>,
    // 必须使用 Arc ，因为 wgpu::Instance::create_surface 要求实现 Send 和 Sync
    window: Option<Arc<winit::window::Window>>,
    wgpu: Option<RefCell<Wgpu>>,
    #[cfg(target_arch = "wasm32")]
    wgpu_context_receiver: Option<futures::channel::oneshot::Receiver<Wgpu>>,
    renderer: Option<RefCell<Renderer>>,

    gui_state: RefCell<GuiState>,
    #[getset(get = "pub")]
    camera: Camera,
    scene: Scene,
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

        app.scene = Scene::scene_quad();
        app.gui_state = RefCell::new(GuiState {
            checkbox: false,
            samples_per_pixel: 1,
            max_ray_bounces: 0,
            camera_parameter: CameraParameter {
                position: app.scene.camera_initial_position,
                look_at: app.scene.camera_initial_look_at,
                vfov: 45.0,
                up: *UNIT_Y,
                focus_distance: 1.0,
                defocus_angle: 0.0,
                movement_speed: 0.0,
                rotation_scale: Default::default(),
            },
        });
        app.camera = Camera::new(app.gui_state.borrow().camera_parameter());

        event_loop.run_app(&mut app).expect("panic");
    }

    fn resize(&mut self, size: &winit::dpi::PhysicalSize<u32>) {
        info!("Resizing to {:?}", size);

        if size.width == 0 || size.height == 0 {
            return;
        }

        self.renderer_mut().on_resize(self.wgpu(), size, &self.camera);
        self.wgpu_mut().on_resize(size);
    }

    fn update(&mut self) {
        let delta_time = self.update_delta_time();
        let window = self.window();

        self.renderer_mut().on_update(window, delta_time, self.gui_state_mut());
    }

    fn render(&mut self) {
        let (width, height): (u32, u32) = self.window().inner_size().into();

        if width == 0 || height == 0 {
            return;
        }

        let surface_texture = match self.wgpu().surface.get_current_texture() {
            Ok(surface_texture) => surface_texture,
            Err(e) => {
                eprintln!("dropped frame: {e:?}");
                return;
            }
        };

        let wgpu_surface_storage = WgpuTexture::new_from_texture(
            "surface",
            &surface_texture.texture,
            WgpuTextureBindingInstruction {
                visibility: ShaderStages::COMPUTE,
                binding_type: WgpuTextureBindingType::StorageTexture,
                storage_access: None,
                sample_type: Some(TextureSampleType::Float { filterable: false }),
            },
        );

        self.renderer_mut().render(self.wgpu(), wgpu_surface_storage);

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
                    let wgpu = pollster::block_on(Wgpu::new(window.clone()));
                    self.wgpu = Some(RefCell::new(wgpu));
                    self.on_wgpu_received();

                }
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: ()) {}

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: winit::window::WindowId, event: WindowEvent) {
        if !self.try_receive_wgpu() {
            return;
        }

        if window_id == self.window().id() {
            // Receive gui window event
            if self.renderer_mut().on_window_event(self.window(), &event).consumed {
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
                WindowEvent::Occluded(_) => {
                    info!("Occluded");
                }
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
        self.try_receive_wgpu();
        self.window().request_redraw();
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {}

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {}

    fn memory_warning(&mut self, _event_loop: &ActiveEventLoop) {}
}

impl App {
    fn try_receive_wgpu(&mut self) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            let mut receive_success = false;
            if let Some(receiver) = self.wgpu_context_receiver.as_mut() {
                if let Ok(Some(wgpu)) = receiver.try_recv() {
                    self.wgpu = Some(RefCell::new(wgpu));
                    self.on_wgpu_received();

                    receive_success = true;
                }
            }

            if receive_success {
                self.wgpu_context_receiver = None;
            }
        }

        self.wgpu.is_some()
    }

    fn on_wgpu_received(&mut self) {
        let camera = Camera::new(self.gui_state().camera_parameter());
        self.camera = camera;

        let render_parameter = RendererParameters {
            samples_per_pixel: self.gui_state().samples_per_pixel(),
            max_ray_bounces: self.gui_state().max_ray_bounces(),
            max_width: 3840,
            max_height: 2160,
            clear_color: Point4::new(0.0, 0.0, 0.0, 1.0),
        };

        self.renderer = Some(RefCell::new(Renderer::new(
            self.wgpu(),
            self.window(),
            self.camera(),
            &render_parameter,
            &self.scene.primitives(),
        )));
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
    fn wgpu(&self) -> Ref<'_, Wgpu> {
        self.wgpu.as_ref().unwrap().borrow()
    }
    fn wgpu_mut(&self) -> RefMut<'_, Wgpu> {
        self.wgpu.as_ref().unwrap().borrow_mut()
    }
    fn renderer_mut(&self) -> RefMut<'_, Renderer> {
        self.renderer.as_ref().unwrap().borrow_mut()
    }
    fn gui_state(&self) -> Ref<'_, GuiState> {
        self.gui_state.borrow()
    }
    fn gui_state_mut(&self) -> RefMut<'_, GuiState> {
        self.gui_state.borrow_mut()
    }
}
