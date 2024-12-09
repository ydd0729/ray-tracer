pub mod bind_buffer;
pub mod index_buffer;
pub mod vertex_buffer;

pub use bind_buffer::*;
pub use index_buffer::*;
pub use vertex_buffer::*;

use crate::render::wgpu::Wgpu;
use getset::Getters;
use wgpu::*;

pub trait IWgpuBuffer {
    fn buffer(&self) -> &Buffer;

    fn as_entire_binding(&self) -> BindingResource {
        self.buffer().as_entire_binding()
    }

    fn slice(&self) -> BufferSlice {
        self.buffer().slice(..)
    }

    fn write(&self, wgpu: &Wgpu, offset: usize, data: &[u8]) {
        wgpu.queue.write_buffer(self.buffer(), offset as BufferAddress, data);
    }
}

#[derive(Getters)]
struct WgpuBuffer {
    buffer: Buffer,
}

impl IWgpuBuffer for WgpuBuffer {
    fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}

impl WgpuBuffer {
    pub fn new(wgpu: &Wgpu, label: &str, size: BufferAddress, usage: BufferUsages) -> Self {
        let buffer = wgpu.device.create_buffer(&BufferDescriptor {
            label: Some(format!("{} buffer", label).as_str()),
            size,
            usage,
            mapped_at_creation: false,
        });

        Self { buffer }
    }
}
