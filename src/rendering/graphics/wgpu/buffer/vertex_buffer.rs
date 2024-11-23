use crate::rendering::wgpu::buffer::WgpuBuffer;
use crate::rendering::wgpu::{IWgpuBuffer, Wgpu};
use crate::rendering::Vertex;
use getset::{CopyGetters, Getters};
use wgpu::{
    vertex_attr_array, Buffer, BufferAddress, BufferUsages, VertexAttribute, VertexBufferLayout,
    VertexStepMode,
};

#[derive(Getters, CopyGetters)]
pub struct WgpuVertexBuffer {
    buffer: WgpuBuffer,
    #[getset(get_copy = "pub")]
    len: usize,
}

impl IWgpuBuffer for WgpuVertexBuffer {
    fn buffer(&self) -> &Buffer {
        self.buffer.buffer()
    }
}

impl WgpuVertexBuffer {
    pub fn new(wgpu: &Wgpu, label: &str, len: usize) -> Self {
        let buffer = WgpuBuffer::new(
            wgpu,
            format!("{} vertex", label).as_str(),
            (Vertex::size() * len) as BufferAddress,
            BufferUsages::VERTEX | BufferUsages::COPY_DST,
        );

        Self { buffer, len }
    }

    pub fn write_vertex(&self, wgpu: &Wgpu, data: &[Vertex]) {
        self.buffer.write(wgpu, bytemuck::cast_slice(data));
    }

    pub fn layout(&self) -> VertexBufferLayout {
        Self::VERTEX_BUFFER_LAYOUT
    }

    const VERTEX_ATTRIBUTES: [VertexAttribute; 4] =
        vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x2];

    const VERTEX_BUFFER_LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: size_of::<Vertex>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &Self::VERTEX_ATTRIBUTES,
    };
}
