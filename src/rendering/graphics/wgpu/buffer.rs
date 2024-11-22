use crate::rendering::vertex::Vertex;
use crate::rendering::wgpu::Wgpu;
use wgpu::*;

struct WgpuBuffer {
    pub buffer: Buffer,
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

    pub fn slice(&self) -> BufferSlice {
        self.buffer.slice(..)
    }

    pub fn write(&self, wgpu: &Wgpu, data: &[u8]) {
        wgpu.queue.write_buffer(&self.buffer, 0, data);
    }
}

pub struct WgpuVertexBuffer {
    buffer: WgpuBuffer,
    len: usize,
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

    pub fn write(&self, wgpu: &Wgpu, data: &[Vertex]) {
        self.buffer.write(wgpu, bytemuck::cast_slice(data));
    }

    pub fn slice(&self) -> BufferSlice {
        self.buffer.slice()
    }

    pub fn layout(&self) -> VertexBufferLayout {
        Self::VERTEX_BUFFER_LAYOUT
    }

    pub fn len(&self) -> usize {
        self.len
    }

    const VERTEX_ATTRIBUTES: [VertexAttribute; 3] = vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4];

    const VERTEX_BUFFER_LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: size_of::<Vertex>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &Self::VERTEX_ATTRIBUTES,
    };
}

pub struct WgpuIndexBuffer {
    buffer: WgpuBuffer,
    len: usize,
}

impl WgpuIndexBuffer {
    pub fn new(wgpu: &Wgpu, label: &str, index: &[u16]) -> Self {
        let len = index.len();
        let buffer = WgpuBuffer::new(
            wgpu,
            format!("{} index", label).as_str(),
            size_of_val(index) as BufferAddress,
            BufferUsages::INDEX | BufferUsages::COPY_DST,
        );

        Self { buffer, len }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn slice(&self) -> BufferSlice {
        self.buffer.slice()
    }

    pub fn write(&self, wgpu: &Wgpu, data: &[u16]) {
        self.buffer.write(wgpu, bytemuck::cast_slice(data));
    }

    pub const fn index_format() -> IndexFormat {
        IndexFormat::Uint16
    }
}
