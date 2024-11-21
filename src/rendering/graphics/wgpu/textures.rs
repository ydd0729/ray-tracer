use crate::rendering::graphics::wgpu::Wgpu;
use std::cell::Ref;
use std::sync::Arc;
use wgpu::{
    Extent3d, Label, SurfaceTexture, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureView, TextureViewDescriptor,
};
use winit::dpi::PhysicalSize;

pub struct Textures {
    depth_texture_descriptor: TextureDescriptor<'static>,
    depth_texture: Texture,
    depth_texture_view_descriptor: TextureViewDescriptor<'static>,
    depth_texture_view: TextureView,
    multisampled_texture_descriptor: TextureDescriptor<'static>,
}

impl Textures {
    pub fn new(window: Arc<winit::window::Window>, wgpu: Ref<Wgpu>) -> Self {
        let (width, height) = window.inner_size().into();
        let depth_texture_descriptor = TextureDescriptor {
            label: Some("Depth Texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let depth_texture = wgpu.device.create_texture(&depth_texture_descriptor);
        let depth_texture_view_descriptor = TextureViewDescriptor {
            label: wgpu::Label::from("depth_texture_view_descriptor"),
            format: Some(TextureFormat::Depth32Float),
            ..Default::default()
        };
        let depth_texture_view = depth_texture.create_view(&depth_texture_view_descriptor);
        let multisampled_texture_descriptor = TextureDescriptor {
            label: Label::from("multisample_texture"),
            size: Extent3d::default(),
            mip_level_count: 1,
            sample_count: 4,
            dimension: TextureDimension::D2,
            format: TextureFormat::R8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };

        Self {
            depth_texture_descriptor,
            depth_texture,
            depth_texture_view_descriptor,
            depth_texture_view,
            multisampled_texture_descriptor,
        }
    }

    pub fn create_multisampled_texture_view(
        &mut self,
        surface_texture: &SurfaceTexture,
        wgpu: Ref<Wgpu>,
    ) -> TextureView {
        self.multisampled_texture_descriptor.format = surface_texture.texture.format();
        let multisampled_texture = wgpu.device.create_texture(&self.multisampled_texture_descriptor);
        multisampled_texture.create_view(&Default::default())
    }

    pub fn on_resize(&mut self, size: &PhysicalSize<u32>, wgpu: Ref<Wgpu>) {
        self.depth_texture_descriptor.size.width = size.width;
        self.depth_texture_descriptor.size.height = size.height;
        self.depth_texture = wgpu.device.create_texture(&self.depth_texture_descriptor);
        self.depth_texture_view = self.depth_texture.create_view(&self.depth_texture_view_descriptor);

        self.multisampled_texture_descriptor.size.width = size.width;
        self.multisampled_texture_descriptor.size.height = size.height;
    }
}
