use crate::rendering::wgpu::{Wgpu, WgpuIndexBuffer, WgpuRenderPass, WgpuVertexBuffer};
use crate::rendering::RenderingConfiguration;
use crate::rendering::Vertex;
use crate::RESOLVE_SHADER;
use std::borrow::Cow;
use std::cell::Ref;
use wgpu::{CommandEncoder, ShaderModuleDescriptor, ShaderSource, TextureView};

pub struct Renderer {
    #[allow(unused)]
    rendering_configuration: RenderingConfiguration,
    resolve_vertex_buffer: WgpuVertexBuffer,
    resolve_index_buffer: WgpuIndexBuffer,
    resolve_render_pass: WgpuRenderPass,
}

impl Renderer {
    pub fn new(wgpu: Ref<Wgpu>) -> Self {
        let resolve_vertices = [
            Vertex::default()
                .with_position(-1.0, 1.0, 0.0, 1.0)
                .with_color(0.9, 0.3, 0.6, 1.0),
            Vertex::default()
                .with_position(-1.0, -1.0, 0.0, 1.0)
                .with_color(0.3, 0.6, 0.9, 1.0),
            Vertex::default()
                .with_position(1.0, -1.0, 0.0, 1.0)
                .with_color(0.6, 0.9, 0.3, 1.0),
            Vertex::default()
                .with_position(1.0, 1.0, 0.0, 1.0)
                .with_color(0.9, 0.9, 0.6, 1.0),
        ];

        let resolve_index: [u16; 6] = [0, 1, 2, 2, 3, 0];

        let resolve_vertex_buffer = WgpuVertexBuffer::new(&wgpu, "resolve", resolve_vertices.len());
        resolve_vertex_buffer.write(&wgpu, &resolve_vertices);

        let resolve_index_buffer = WgpuIndexBuffer::new(&wgpu, "resolve", &resolve_index);
        resolve_index_buffer.write(&wgpu, &resolve_index);

        let resolve_render_pass = WgpuRenderPass::new(
            &wgpu,
            "resolve",
            &[&resolve_vertex_buffer],
            None,
            &wgpu.device.create_shader_module(ShaderModuleDescriptor {
                label: Some("resolve shader"),
                source: ShaderSource::Wgsl(Cow::Borrowed(*RESOLVE_SHADER)),
            }),
        );

        Self {
            rendering_configuration: Default::default(),
            resolve_vertex_buffer,
            resolve_index_buffer,
            resolve_render_pass,
        }
    }

    pub fn render(&self, surface_view: &TextureView, encoder: &mut CommandEncoder) {
        self.resolve_render_pass.render(
            encoder,
            surface_view,
            &[&self.resolve_vertex_buffer],
            Some(&self.resolve_index_buffer),
        );
    }
}
