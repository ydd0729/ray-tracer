use crate::render::wgpu::{Wgpu, WgpuBindGroup};
use wgpu::*;

pub struct WgpuComputePass {
    label: Box<str>,
    pipeline: ComputePipeline,
    work_group_size: [u32; 3],
}

impl WgpuComputePass {
    pub fn new(
        wgpu: &Wgpu,
        label: &str,
        bind_group_layouts: Option<&[&BindGroupLayout]>,
        shader: &ShaderModule,
        work_group_size: [u32; 3],
    ) -> Self {
        let pipeline_layout = wgpu.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Label::from(format!("{label} compute pipeline layout").as_str()),
            bind_group_layouts: bind_group_layouts.unwrap_or_default(),
            push_constant_ranges: &[],
        });

        let pipeline = wgpu.device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Label::from(format!("{label} compute pipeline").as_str()),
            layout: Some(&pipeline_layout),
            module: shader,
            entry_point: Some("compute_main"),
            compilation_options: PipelineCompilationOptions {
                constants: &Default::default(), // overridable constants
                zero_initialize_workgroup_memory: false,
            },
            cache: None,
        });

        Self {
            label: label.into(),
            pipeline,
            work_group_size,
        }
    }

    pub fn render(&self, encoder: &mut CommandEncoder, bind_groups: Option<&[&WgpuBindGroup]>) {
        let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Label::from(self.label.as_ref()),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.pipeline);

        if let Some(bind_groups) = bind_groups {
            for bind_group in bind_groups {
                compute_pass.set_bind_group(bind_group.group_id(), bind_group.bind_group(), &[]);
            }
        }

        compute_pass.dispatch_workgroups(
            self.work_group_size[0],
            self.work_group_size[1],
            self.work_group_size[2],
        );
    }
}
