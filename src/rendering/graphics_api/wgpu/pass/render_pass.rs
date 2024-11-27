use crate::rendering::wgpu::bind_group::WgpuBindGroup;
use crate::rendering::wgpu::index_buffer::WgpuIndexBuffer;
use crate::rendering::wgpu::vertex_buffer::WgpuVertexBuffer;
use crate::rendering::wgpu::{IWgpuBuffer, Wgpu};
use wgpu::*;

pub struct WgpuRenderPass {
    label: Box<str>,
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

        let pipeline_layout = wgpu.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Label::from(format!("{label} render pipeline layout").as_str()),
            bind_group_layouts: bind_group_layouts.unwrap_or_default(),
            push_constant_ranges: &[],
        });

        let pipeline = wgpu.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Label::from(format!("{label} pipeline").as_str()),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: shader,
                entry_point: Some("vertex_main"),
                compilation_options: Default::default(),
                buffers: vertex_buffer_layout.as_slice(),
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: shader,
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
            label: label.into(),
            pipeline,
        }
    }

    pub fn render(
        &self,
        encoder: &mut CommandEncoder,
        texture_view: Option<&TextureView>,
        vertex_buffers: &[&WgpuVertexBuffer],
        index_buffer: Option<&WgpuIndexBuffer>,
        bind_groups: Option<&[&WgpuBindGroup]>,
        load_op: LoadOp<Color>,
        store_op: StoreOp,
    ) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some(&self.label),
            color_attachments: &if let Some(texture_view) = texture_view {
                [Some(RenderPassColorAttachment {
                    view: texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: load_op,
                        store: store_op,
                    },
                })]
            } else {
                [None]
            },
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);

        if let Some(bind_groups) = bind_groups {
            for bind_group in bind_groups {
                render_pass.set_bind_group(bind_group.group_id(), bind_group.bind_group(), &[]);
            }
        }

        let mut vertex_count = 0;
        for (i, vertex_buffer) in vertex_buffers.iter().enumerate() {
            render_pass.set_vertex_buffer(i as u32, vertex_buffer.slice());
            vertex_count += vertex_buffer.len() as u32;
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
