use crate::color::StaticColor;
use pollster::block_on;
use std::sync::Arc;

pub(crate) struct WgpuContext {
    // window: Arc<winit::window::Window>,
    pub(crate) surface_configuration: wgpu::SurfaceConfiguration,
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) clear_color: wgpu::Color,
}

impl WgpuContext {
    pub(crate) fn new(window: Arc<winit::window::Window>) -> Self {
        let instance_flags = wgpu::InstanceFlags::from_build_config().with_env();
        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: instance_flags,
            dx12_shader_compiler: Default::default(),
            gles_minor_version: Default::default(),
        };
        let instance = wgpu::Instance::new(instance_descriptor);
        let surface = instance.create_surface(window.clone()).unwrap();

        let request_adapter_options = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        };
        let adapter = block_on(instance.request_adapter(&request_adapter_options)).unwrap();

        let device_descriptor = wgpu::DeviceDescriptor {
            label: wgpu::Label::from("default device"),
            required_features: Default::default(),
            required_limits: Default::default(),
            memory_hints: Default::default(),
        };
        let (device, queue) = block_on(adapter.request_device(&device_descriptor, None)).unwrap();

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::HtmlCanvasElement;
            use wasm_bindgen::JsCast;

            let mut scale = 1.0;
            web_sys::window()
                .and_then(|win| {
                    scale = win.device_pixel_ratio();
                    win.document()
                })
                .and_then(|doc| {
                    let canvas: HtmlCanvasElement = doc
                        .get_element_by_id("wgpu_canvas")
                        .expect("failed to get element by id")
                        .dyn_into::<HtmlCanvasElement>()
                        .map_err(|_| "element is not a HtmlCanvasElement")
                        .unwrap();

                    canvas.set_attribute(
                        "style",
                        &format!(
                            "max-width:{0}px;max_height:{0}px",
                            device.limits().max_texture_dimension_2d as f64 / scale
                        ),
                    ).expect("panic");

                    Some(())
                });
        }

        let physical_size = window.inner_size();
        let surface_configuration =
            surface.get_default_config(&adapter, physical_size.width, physical_size.height).unwrap();
        surface.configure(&device, &surface_configuration);

        Self {
            // window,
            surface_configuration,
            surface,
            device,
            queue,
            clear_color: StaticColor::CLEAR,
        }
    }

    pub(crate) fn resize(&mut self, size: &winit::dpi::PhysicalSize<u32>) {
        self.surface_configuration.width = size.width;
        self.surface_configuration.height = size.height;
        self.surface.configure(&self.device, &self.surface_configuration);
    }
}