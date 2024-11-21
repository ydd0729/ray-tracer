pub mod textures;

use log::info;
use std::sync::Arc;
pub use textures::*;
use wgpu::*;

pub struct Wgpu {
    // window: Arc<winit::window::Window>,
    pub surface_configuration: SurfaceConfiguration,
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
}

impl Wgpu {
    pub async fn new(window: Arc<winit::window::Window>) -> Self {
        let instance_flags = InstanceFlags::from_build_config().with_env();
        let instance_descriptor = InstanceDescriptor {
            backends: util::backend_bits_from_env().unwrap_or(Backends::PRIMARY),
            flags: instance_flags,
            dx12_shader_compiler: Dx12Compiler::default(),
            gles_minor_version: Gles3MinorVersion::Automatic,
        };
        let instance = Instance::new(instance_descriptor);
        let surface = instance.create_surface(window.clone()).unwrap();

        let request_adapter_options = RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        };
        let adapter = instance
            .request_adapter(&request_adapter_options)
            .await
            .expect("Failed to request adapter!");

        info!("{:?}", adapter.get_info());

        let device_descriptor = DeviceDescriptor {
            label: wgpu::Label::from("default device"),
            required_features: Features::empty(),
            required_limits: Limits::default(),
            memory_hints: MemoryHints::default(),
        };
        let (device, queue) = adapter
            .request_device(&device_descriptor, None)
            .await
            .expect("Failed to request a device!");

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use web_sys::HtmlCanvasElement;

            let mut device_pixel_ratio = 1.0;
            web_sys::window()
                .and_then(|win| {
                    device_pixel_ratio = win.device_pixel_ratio();
                    win.document()
                })
                .and_then(|doc| {
                    let canvas: HtmlCanvasElement = doc
                        .get_element_by_id("canvas")
                        .expect("failed to get element by id")
                        .dyn_into::<HtmlCanvasElement>()
                        .map_err(|_| "element is not a HtmlCanvasElement")
                        .unwrap();

                    canvas
                        .set_attribute(
                            "style",
                            &format!(
                                "max-width:{0}px;max_height:{0}px",
                                device.limits().max_texture_dimension_2d as f64 / device_pixel_ratio
                            ),
                        )
                        .expect("panic");

                    Some(())
                });
        }

        let physical_size = window.inner_size();
        let surface_configuration = surface
            .get_default_config(&adapter, physical_size.width, physical_size.height)
            .unwrap();
        surface.configure(&device, &surface_configuration);

        Self {
            // window,
            surface_configuration,
            surface,
            device,
            queue,
        }
    }

    pub fn on_resize(&mut self, size: &winit::dpi::PhysicalSize<u32>) {
        self.surface_configuration.width = size.width;
        self.surface_configuration.height = size.height;
        self.surface.configure(&self.device, &self.surface_configuration);
    }
}
