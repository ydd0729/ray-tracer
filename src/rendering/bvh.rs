use bytemuck::{Pod, Zeroable};

#[allow(unused)]
struct BoundingVolumeTree {
    date: Vec<BoundingVolumeNode>,
}

#[repr(C)]
#[derive(Copy, Clone, Zeroable, Pod)]
struct BoundingVolumeNode {}
