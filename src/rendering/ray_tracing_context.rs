use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Zeroable, Pod)]
pub struct RayTracingContext {
    pub width: u32,
    pub height: u32,
}
