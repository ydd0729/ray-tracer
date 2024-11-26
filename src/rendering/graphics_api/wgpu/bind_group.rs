use crate::rendering::wgpu::bind_buffer::WgpuBindBuffer;
use crate::rendering::wgpu::{IWgpuBuffer, Wgpu};
use getset::*;
use std::rc::Rc;
use wgpu::*;

#[derive(CopyGetters, Getters)]
pub struct WgpuBindGroup {
    // label: Option<&'a str>,
    #[getset(get_copy = "pub")]
    group_id: u32,

    #[getset(get = "pub")]
    bind_group_layout: BindGroupLayout,

    #[getset(get = "pub")]
    bind_group: BindGroup,
}

impl WgpuBindGroup {
    pub fn new(wgpu: &Wgpu, label: Option<&str>, group_id: u32, buffers: Vec<Rc<WgpuBindBuffer>>) -> Self {
        let mut bind_group_layout_entries = Vec::new();
        let mut bind_group_entries = Vec::<BindGroupEntry>::new();

        for (i, buffer) in buffers.iter().enumerate() {
            bind_group_layout_entries.push(BindGroupLayoutEntry {
                binding: i as u32,
                visibility: buffer.visibility(),
                ty: BindingType::Buffer {
                    ty: buffer.binding_type(),
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None, // 只有在 BufferBindingType::Texture 时才需要此项
            });
            bind_group_entries.push(BindGroupEntry {
                binding: i as u32,
                resource: buffer.as_entire_binding(),
            });
        }

        let label = label.unwrap_or("");

        let bind_group_layout = wgpu.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Label::from(format!("{} bind group layout", label).as_str()),
            entries: bind_group_layout_entries.as_slice(),
        });

        let bind_group = wgpu.device.create_bind_group(&BindGroupDescriptor {
            label: Label::from(format!("{} bind group", label).as_str()),
            layout: &bind_group_layout,
            entries: bind_group_entries.as_slice(),
        });

        Self {
            group_id,
            bind_group,
            bind_group_layout,
        }
    }
}
