use crate::rendering::wgpu::buffer::WgpuBuffer;
use crate::rendering::wgpu::{IWgpuBuffer, Wgpu};
use getset::CopyGetters;
use wgpu::{Buffer, BufferAddress, BufferUsages, IndexFormat};

#[derive(CopyGetters)]
pub struct WgpuIndexBuffer {
    buffer: WgpuBuffer,
    #[getset(get_copy = "pub")]
    len: usize,
}

impl IWgpuBuffer for WgpuIndexBuffer {
    fn buffer(&self) -> &Buffer {
        self.buffer.buffer()
    }
}

impl WgpuIndexBuffer {
    pub fn new(wgpu: &Wgpu, label: &str, len: usize) -> Self {
        let buffer = WgpuBuffer::new(
            wgpu,
            format!("{} index", label).as_str(),
            (size_of::<u32>() * len) as BufferAddress,
            BufferUsages::INDEX | BufferUsages::COPY_DST,
        );

        Self { buffer, len }
    }

    pub fn write_index(&self, wgpu: &Wgpu, data: &[u32]) {
        self.buffer.write(wgpu, 0,bytemuck::cast_slice(data));
    }

    pub const fn index_format() -> IndexFormat {
        IndexFormat::Uint32
    }
}
