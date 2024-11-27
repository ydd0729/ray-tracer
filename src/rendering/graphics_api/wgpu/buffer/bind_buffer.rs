use crate::rendering::wgpu::buffer::WgpuBuffer;
use crate::rendering::wgpu::{IWgpuBuffer, Wgpu, WgpuBindable};
use getset::CopyGetters;
use wgpu::{
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferAddress,
    BufferBindingType, BufferUsages, ShaderStages,
};

#[derive(CopyGetters)]
pub struct WgpuBindBuffer {
    buffer: WgpuBuffer,
    visibility: ShaderStages,
    buffer_binding_type: BufferBindingType,
    // read_only: bool,
}

impl IWgpuBuffer for WgpuBindBuffer {
    fn buffer(&self) -> &Buffer {
        self.buffer.buffer()
    }
}

impl WgpuBindBuffer {
    pub fn new(
        wgpu: &Wgpu,
        label: &str,
        size: BufferAddress,
        usage: BufferUsages,
        visibility: ShaderStages,
        read_only: bool,
    ) -> Self {
        let buffer = WgpuBuffer::new(wgpu, label, size, usage);

        let buffer_binding_type: BufferBindingType;
        if usage.contains(BufferUsages::UNIFORM) {
            buffer_binding_type = BufferBindingType::Uniform;
        } else if usage.contains(BufferUsages::STORAGE) {
            buffer_binding_type = BufferBindingType::Storage { read_only };
        } else {
            panic!("Unsupported buffer usage");
        }

        Self {
            buffer,
            visibility,
            buffer_binding_type,
            // read_only,
        }
    }
}

impl<'a> WgpuBindable<'a> for WgpuBindBuffer {
    fn bind_group_layout_entry(&self) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding: 0,
            visibility: self.visibility,
            ty: BindingType::Buffer {
                ty: self.buffer_binding_type,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None, // 只有在 BufferBindingType::Texture 时才需要此项
        }
    }

    fn binding_resource(&'a self) -> BindingResource<'a> {
        self.as_entire_binding()
    }
}
