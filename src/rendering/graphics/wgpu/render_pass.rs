use crate::rendering::wgpu::{Wgpu, WgpuIndexBuffer, WgpuVertexBuffer};
use wgpu::*;

pub struct WgpuRenderPass {
    // pipeline_layout: Option<PipelineLayout>,
    pipeline: RenderPipeline,
}

impl WgpuRenderPass {
    pub fn new(
        wgpu: &Wgpu,
        label: &str,
        vertex_buffers: &[&WgpuVertexBuffer],
        bind_group_layouts: Option<&[&BindGroupLayout]>,
        shader: &ShaderModule,
    ) -> Self {
        let mut vertex_buffer_layout: Vec<VertexBufferLayout> = Vec::new();

        for vertex_buffer in vertex_buffers {
            vertex_buffer_layout.push(vertex_buffer.layout());
        }

        let pipeline_layout = bind_group_layouts.map(|bind_group_layouts| {
            wgpu.device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Label::from(format!("{label} pipeline layout").as_str()),
                bind_group_layouts,
                push_constant_ranges: &[],
            })
        });

        let pipeline = wgpu.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Label::from(format!("{label} pipeline").as_str()),
            layout: pipeline_layout.as_ref(),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vertex_main"),
                compilation_options: Default::default(),
                buffers: vertex_buffer_layout.as_slice(),
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fragment_main"),
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: wgpu.surface_configuration.format,
                    blend: Some(BlendState {
                        color: BlendComponent::REPLACE,
                        alpha: BlendComponent::REPLACE,
                    }),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });

        Self {
            // pipeline_layout,
            pipeline,
        }
    }

    pub fn render(
        &self,
        encoder: &mut CommandEncoder,
        texture_view: &TextureView,
        vertex_buffers: &[&WgpuVertexBuffer],
        index_buffer: Option<&WgpuIndexBuffer>,
    ) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: texture_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);

        let mut i = 0;
        let mut vertex_count = 0;
        while i < vertex_buffers.len() {
            render_pass.set_vertex_buffer(i as u32, vertex_buffers[i].slice());
            vertex_count += vertex_buffers[i].len() as u32;
            i += 1;
        }

        match index_buffer {
            None => {
                render_pass.draw(0..vertex_count, 0..1);
            }
            Some(index_buffer) => {
                render_pass.set_index_buffer(index_buffer.slice(), WgpuIndexBuffer::index_format());
                render_pass.draw_indexed(0..index_buffer.len() as u32, 0, 0..1);
            }
        }
    }
}
