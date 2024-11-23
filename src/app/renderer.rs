use crate::rendering::configuration::RenderingConfiguration;
use crate::rendering::wgpu::{IWgpuBuffer, WgpuVertexBuffer};
use crate::rendering::wgpu::{Wgpu, WgpuRenderPass};
use crate::rendering::wgpu::{WgpuBindBuffer, WgpuBindGroup};
use crate::rendering::wgpu::{WgpuComputePass, WgpuIndexBuffer};
use crate::rendering::{RayTracingContext, Vertex};
use crate::{RAY_TRACING_SHADER, RESOLVE_SHADER};
use std::borrow::Cow;
use std::cell::Ref;
use std::rc::Rc;
use std::sync::Arc;
use wgpu::*;
use winit::dpi::PhysicalSize;

pub struct Renderer {
    #[allow(unused)]
    rendering_configuration: RenderingConfiguration,

    ray_tracing_context: RayTracingContext,
    ray_tracing_context_uniform_buffer: Rc<WgpuBindBuffer>,

    pixel_color_storage: Vec<[f32; 3]>,
    pixel_color_storage_buffer: Rc<WgpuBindBuffer>,

    resolve_vertex_buffer: WgpuVertexBuffer,
    resolve_index_buffer: WgpuIndexBuffer,
}

impl Renderer {
    pub fn new(wgpu: Ref<Wgpu>, window: Arc<winit::window::Window>) -> Self {
        let window_size = window.inner_size();
        let rendering_configuration = RenderingConfiguration {
            msaa: false,
            max_width: 3840, // UHD 4K
            max_height: 2160,
            width: window_size.width as usize,
            height: window_size.height as usize,
        };

        let ray_tracing_context = RayTracingContext {
            width: rendering_configuration.width as u32,
            height: rendering_configuration.width as u32,
        };

        let ray_tracing_context_uniform_buffer = Rc::new(WgpuBindBuffer::new(
            &wgpu,
            "ray tracing context",
            size_of_val(&ray_tracing_context) as BufferAddress,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
            true,
        ));
        ray_tracing_context_uniform_buffer.write(&wgpu, bytemuck::bytes_of(&ray_tracing_context));

        let pixel_color_storage_buffer = Rc::new(WgpuBindBuffer::new(
            &wgpu,
            "pixel color storage",
            (size_of::<f32>() * 3 * rendering_configuration.max_pixels()) as BufferAddress,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            ShaderStages::COMPUTE | ShaderStages::FRAGMENT,
            false,
        ));

        let resolve_vertices = [
            Vertex::default()
                .with_position(-1.0, 1.0, 0.0, 1.0)
                .with_tex_coords(0.0, 0.0),
            Vertex::default()
                .with_position(-1.0, -1.0, 0.0, 1.0)
                .with_tex_coords(0.0, 1.0),
            Vertex::default()
                .with_position(1.0, -1.0, 0.0, 1.0)
                .with_tex_coords(1.0, 1.0),
            Vertex::default()
                .with_position(1.0, 1.0, 0.0, 1.0)
                .with_tex_coords(1.0, 0.0),
        ];

        let resolve_index: [u32; 6] = [0, 1, 2, 2, 3, 0];

        let resolve_vertex_buffer = WgpuVertexBuffer::new(&wgpu, "resolve", resolve_vertices.len());
        resolve_vertex_buffer.write_vertex(&wgpu, &resolve_vertices);

        let resolve_index_buffer = WgpuIndexBuffer::new(&wgpu, "resolve", resolve_index.len());
        resolve_index_buffer.write_index(&wgpu, &resolve_index);

        Self {
            rendering_configuration,
            ray_tracing_context,
            ray_tracing_context_uniform_buffer,
            pixel_color_storage: Vec::new(),
            pixel_color_storage_buffer,
            resolve_vertex_buffer,
            resolve_index_buffer,
        }
    }

    pub fn render(
        &mut self,
        wgpu: Ref<Wgpu>,
        surface_view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        self.pixel_color_storage.clear();
        let screen_pixels = self.rendering_configuration.pixels();
        self.pixel_color_storage
            .resize(screen_pixels, Default::default());

        self.pixel_color_storage_buffer.write(
            &wgpu,
            bytemuck::cast_slice(self.pixel_color_storage.as_slice()),
        );

        let ray_tracing_bind_group = WgpuBindGroup::new(
            &wgpu,
            Option::from("ray tracing"),
            0,
            vec![
                Rc::clone(&self.ray_tracing_context_uniform_buffer),
                Rc::clone(&self.pixel_color_storage_buffer),
            ],
        );

        let ray_tracing_compute_pass = WgpuComputePass::new(
            &wgpu,
            "ray tracing",
            Some(&[ray_tracing_bind_group.bind_group_layout()]),
            &wgpu.device.create_shader_module(ShaderModuleDescriptor {
                label: Some("ray tracing shader"),
                source: ShaderSource::Wgsl(Cow::Borrowed(*RAY_TRACING_SHADER)),
            }),
            [
                (self.rendering_configuration.width as f32 / 16f32).ceil() as u32,
                (self.rendering_configuration.height as f32 / 16f32).ceil() as u32,
                1,
            ],
        );

        ray_tracing_compute_pass.render(encoder, Some(&[&ray_tracing_bind_group]));

        let resolve_bind_group = WgpuBindGroup::new(
            &wgpu,
            Option::from("resolve"),
            0,
            vec![
                Rc::clone(&self.ray_tracing_context_uniform_buffer),
                Rc::clone(&self.pixel_color_storage_buffer),
            ],
        );

        let resolve_render_pass = WgpuRenderPass::new(
            &wgpu,
            "resolve",
            &[&self.resolve_vertex_buffer],
            Some(&[resolve_bind_group.bind_group_layout()]),
            &wgpu.device.create_shader_module(ShaderModuleDescriptor {
                label: Some("resolve shader"),
                source: ShaderSource::Wgsl(Cow::Borrowed(*RESOLVE_SHADER)),
            }),
        );

        resolve_render_pass.render(
            encoder,
            surface_view,
            &[&self.resolve_vertex_buffer],
            Some(&self.resolve_index_buffer),
            Some(&[&ray_tracing_bind_group]),
            StoreOp::Store,
        );
    }

    pub fn on_resize(&mut self, wgpu: Ref<Wgpu>, size: &PhysicalSize<u32>) {
        self.ray_tracing_context.width = size.width;
        self.ray_tracing_context.height = size.height;

        self.ray_tracing_context_uniform_buffer
            .write(&wgpu, bytemuck::bytes_of(&self.ray_tracing_context));

        self.rendering_configuration.width = size.width as usize;
        self.rendering_configuration.height = size.height as usize;
    }
}
