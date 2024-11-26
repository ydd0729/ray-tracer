use crate::rendering::wgpu::buffer::WgpuBuffer;
use crate::rendering::wgpu::{IWgpuBuffer, Wgpu};
use getset::CopyGetters;
use wgpu::{Buffer, BufferAddress, BufferBindingType, BufferUsages, ShaderStages};

#[derive(CopyGetters)]
pub struct WgpuBindBuffer {
    buffer: WgpuBuffer,
    #[getset(get_copy = "pub")]
    visibility: ShaderStages,
    #[getset(get_copy = "pub")]
    binding_type: BufferBindingType,
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

        let binding_type: BufferBindingType;
        if usage.contains(BufferUsages::UNIFORM) {
            binding_type = BufferBindingType::Uniform;
        } else if usage.contains(BufferUsages::STORAGE) {
            binding_type = BufferBindingType::Storage { read_only };
        } else {
            panic!("Unsupported buffer usage");
        }

        Self {
            buffer,
            visibility,
            binding_type,
            // read_only,
        }
    }
}
