pub mod camera;
pub mod egui_renderer;
pub mod gui_state;
pub mod input;
mod renderer;
mod scene;

use crate::app::gui_state::GuiState;
use crate::app::input::PressRecord;
use crate::app::renderer::{Renderer, RendererParameters};
use crate::app::scene::Scene;
use crate::rendering::mesh::Mesh;
use crate::rendering::wgpu::{Wgpu, WgpuTexture, WgpuTextureBindingInstruction, WgpuTextureBindingType};
use crate::time;
use camera::{Camera, CameraUpdateParameters};
use cfg_if::cfg_if;
use getset::Getters;
use log::info;
use nalgebra::{Point4, Vector2, Vector3};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{ShaderStages, TextureSampleType};
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use wasm_bindgen::prelude::*;
    }
}

#[derive(Default, Getters)]
pub struct App {
    last_frame_time: Option<time::Instant>,
    size: PhysicalSize<u32>,
    // 必须使用 Arc ，因为 wgpu::Instance::create_surface 要求实现 Send 和 Sync
    window: Option<Arc<winit::window::Window>>,
    wgpu: Option<RefCell<Wgpu>>,
    #[cfg(target_arch = "wasm32")]
    wgpu_context_receiver: Option<futures::channel::oneshot::Receiver<Wgpu>>,
    renderer: Option<RefCell<Renderer>>,

    gui_state: RefCell<GuiState>,
    camera: RefCell<Camera>,
    scene: RefCell<Scene>,

    allow_input: bool,
    key_records: HashMap<KeyCode, PressRecord>,
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

        let keys = vec![
            (KeyCode::KeyW, PressRecord::default()),
            (KeyCode::KeyA, PressRecord::default()),
            (KeyCode::KeyS, PressRecord::default()),
            (KeyCode::KeyD, PressRecord::default()),
            (KeyCode::KeyQ, PressRecord::default()),
            (KeyCode::KeyE, PressRecord::default()),
        ];
        let key_records: HashMap<KeyCode, PressRecord> = keys.into_iter().collect();

        let scene = RefCell::new(Scene::scene_cornell_box());
        // let scene = RefCell::new(Scene::scene_light());
        // let scene = RefCell::new(Scene::scene_light_huge());
        let scene_ref = scene.borrow();

        let gui_state = RefCell::new(GuiState::new(
            500,
            32,
            CameraUpdateParameters {
                vfov: scene_ref.camera_parameters.vfov,
                focus_distance: scene_ref.camera_parameters.focus_distance,
                defocus_angle: scene_ref.camera_parameters.defocus_angle,
                movement_speed: scene_ref.camera_parameters.movement_speed,
                rotation_scale: scene_ref.camera_parameters.rotation_scale,
            },
        ));

        let camera = RefCell::new(Camera::new(&scene_ref.camera_parameters));

        drop(scene_ref);

        let mut app = Self {
            last_frame_time: None,
            size: PhysicalSize::new(0, 0),
            window: None,
            wgpu: None,
            #[cfg(target_arch = "wasm32")]
            wgpu_context_receiver: None,
            renderer: None,
            gui_state,
            camera,
            scene,
            allow_input: false,
            key_records,
        };

        event_loop.run_app(&mut app).expect("panic");
    }

    fn resize(&mut self, size: &PhysicalSize<u32>) {
        if self.size == *size {
            return;
        }
        self.size = *size;

        info!("Resizing to {:?}", size);

        if size.width == 0 || size.height == 0 {
            return;
        }

        self.wgpu_mut().on_resize(size);
        self.renderer_mut().on_resize(self.wgpu(), size, self.camera());
    }

    fn update(&mut self) {
        let delta_time = self.update_delta_time();
        let window = self.window();

        let mut translation = Vector3::<f32>::zeros();

        translation.x = self.key_records.get_mut(&KeyCode::KeyD).unwrap().delta()
            - self.key_records.get_mut(&KeyCode::KeyA).unwrap().delta();

        translation.y = self.key_records.get_mut(&KeyCode::KeyE).unwrap().delta()
            - self.key_records.get_mut(&KeyCode::KeyQ).unwrap().delta();

        translation.z = self.key_records.get_mut(&KeyCode::KeyW).unwrap().delta()
            - self.key_records.get_mut(&KeyCode::KeyS).unwrap().delta();

        self.camera_mut().translate(translation);

        self.camera_mut().on_update(self.gui_state().camera_update_parameters());
        self.renderer_mut()
            .on_update(window, self.wgpu(), delta_time, self.camera_mut(), self.gui_state_mut());

        // info!("camera position: {:?}", self.camera.position());
        // info!("camera rotation: {:?}", self.camera.rotation());
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

        let status = self.renderer_mut().render(self.wgpu(), wgpu_surface_storage);
        self.gui_state_mut().update(status);

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
                            state,
                            ..
                        },
                    ..
                } => {
                    // Exit by pressing the escape key
                    if matches!(key_code, winit::keyboard::KeyCode::Escape) {
                        event_loop.exit();
                    }

                    if self.allow_input && self.key_records.contains_key(key_code) {
                        self.key_records.entry(*key_code).or_default().update(*state);
                    }
                }
                WindowEvent::ModifiersChanged(_) => {}
                WindowEvent::Ime(_) => {}
                WindowEvent::CursorMoved { .. } => {}
                WindowEvent::CursorEntered { .. } => {}
                WindowEvent::CursorLeft { .. } => {}
                WindowEvent::MouseWheel { .. } => {}
                WindowEvent::MouseInput {
                    button: MouseButton::Right,
                    state,
                    ..
                } => match state {
                    ElementState::Pressed => {
                        self.allow_input = true;
                    }
                    ElementState::Released => {
                        self.allow_input = false;
                        for record in &mut self.key_records.values_mut() {
                            record.release()
                        }
                    }
                },
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
            // (x, y) change in position in unspecified units.
            // Different devices may use different units.
            DeviceEvent::MouseMotion { delta: (x, y), .. } => {
                if self.allow_input {
                    self.camera_mut().rotate(&Vector2::new(-x as f32, -y as f32));
                }
            }
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
        let mut primitives = Vec::new();
        let mut important_indices = Vec::new();

        self.scene_mut().primitives(&mut primitives, &mut important_indices);
        info!("important_indices={:?}", important_indices);
        let scene = self.scene();
        let render_parameter = RendererParameters {
            samples_per_pixel: self.gui_state().samples_per_pixel(),
            max_ray_bounces: self.gui_state().max_ray_bounces(),
            max_width: 3840,
            max_height: 2160,
            clear_color: Point4::new(0.0, 0.0, 0.0, 1.0),
            window: self.window(),
            camera: self.camera(),
            primitives: &primitives,
            important_indices: &important_indices,
            materials: &scene.materials,
        };
        let renderer = RefCell::new(Renderer::new(self.wgpu(), &render_parameter));

        drop(render_parameter);
        drop(scene);

        self.renderer = Some(renderer);
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
    fn scene(&self) -> Ref<'_, Scene> {
        self.scene.borrow()
    }
    fn scene_mut(&self) -> RefMut<'_, Scene> {
        self.scene.borrow_mut()
    }
    fn camera(&self) -> Ref<'_, Camera> {
        self.camera.borrow()
    }
    fn camera_mut(&self) -> RefMut<'_, Camera> {
        self.camera.borrow_mut()
    }
}
