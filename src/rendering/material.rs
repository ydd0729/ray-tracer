#[repr(u32)]
#[derive(Copy, Clone)]
enum MaterialType {
    DebugNormal,
}

pub trait Material {
    fn material_type(&self) -> u32;
    fn material_id(&self) -> u32;
}

pub struct DebugNormal {}

impl Material for DebugNormal {
    fn material_type(&self) -> u32 {
        MaterialType::DebugNormal as u32
    }

    fn material_id(&self) -> u32 {
        0
    }
}
