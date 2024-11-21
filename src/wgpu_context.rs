use std::sync::Arc;
use wgpu::{Device, TextureView};
use winit::window::Window;

pub(crate) struct WgpuContext {
    // window: Arc<winit::window::Window>,
    pub(crate) surface_configuration: wgpu::SurfaceConfiguration,
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) depth_texture_view: wgpu::TextureView,
    // pub(crate) clear_color: wgpu::Color,
}

impl WgpuContext {
    pub(crate) async fn new(window: Arc<winit::window::Window>) -> Self {
        let instance_flags = wgpu::InstanceFlags::from_build_config().with_env();
        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: wgpu::util::backend_bits_from_env().unwrap_or_else(|| wgpu::Backends::PRIMARY),
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
        let adapter = instance.request_adapter(&request_adapter_options)
            .await.expect("Failed to request adapter!");

        let device_descriptor = wgpu::DeviceDescriptor {
            label: wgpu::Label::from("default device"),
            required_features: Default::default(),
            required_limits: Default::default(),
            memory_hints: Default::default(),
        };
        let (device, queue) = adapter.request_device(&device_descriptor, None)
            .await.expect("Failed to request a device!");

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::HtmlCanvasElement;
            use wasm_bindgen::JsCast;

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

                    canvas.set_attribute(
                        "style",
                        &format!(
                            "max-width:{0}px;max_height:{0}px",
                            device.limits().max_texture_dimension_2d as f64 / device_pixel_ratio
                        ),
                    ).expect("panic");

                    Some(())
                });
        }

        let physical_size = window.inner_size();
        let mut surface_configuration =
            surface.get_default_config(&adapter, physical_size.width, physical_size.height).unwrap();
        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb()) // egui wants a non-srgb surface texture
            .unwrap_or(surface_capabilities.formats[0]);
        surface_configuration.format = surface_format;
        surface.configure(&device, &surface_configuration);

        let depth_texture_view = Self::create_depth_texture(window, &device);

        Self {
            // window,
            surface_configuration,
            surface,
            device,
            queue,
            depth_texture_view,
            // clear_color: StaticColor::CLEAR,
        }
    }

    fn create_depth_texture(window: Arc<Window>, device: &Device) -> TextureView {
        let (width, height) = window.inner_size().into();
        let depth_texture_descriptor = wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let depth_texture = device.create_texture(&depth_texture_descriptor);
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Depth32Float),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            base_array_layer: 0,
            array_layer_count: None,
            mip_level_count: None,
        });
        depth_texture_view
    }

    pub(crate) fn resize(&mut self, size: &winit::dpi::PhysicalSize<u32>, window: Arc<Window>) {
        self.depth_texture_view = Self::create_depth_texture(window, &self.device);
        self.surface_configuration.width = size.width;
        self.surface_configuration.height = size.height;
        self.surface.configure(&self.device, &self.surface_configuration);
    }
}