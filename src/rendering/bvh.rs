use bytemuck::{Pod, Zeroable};

struct BoundingVolumeTree {
    date: Vec<BoundingVolumeNode>,
}

#[repr(C)]
#[derive(Copy, Clone, Zeroable, Pod)]
struct BoundingVolumeNode {}
