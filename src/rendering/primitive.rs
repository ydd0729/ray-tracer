pub mod quad;
pub mod sphere;
pub mod transformable;

pub use quad::*;
pub use transformable::*;

use crate::rendering::primitive::sphere::SphereData;
use bytemuck::{Pod, Zeroable};

use super::bounding_box::BoundingBox;

pub trait Bound {
    fn bounding_box(&self) -> BoundingBox;
}

#[derive(Debug, Copy, Clone)]
pub enum PrimitiveData {
    Quad(QuadData),
    Sphere(SphereData),
}

impl From<PrimitiveData> for u32 {
    fn from(value: PrimitiveData) -> Self {
        match value {
            PrimitiveData::Quad(_) => 0,
            PrimitiveData::Sphere(_) => 1,
        }
    }
}

impl Bound for PrimitiveData {
    fn bounding_box(&self) -> BoundingBox {
        match self {
            PrimitiveData::Quad(quad_data) => quad_data.bounding_box(),
            PrimitiveData::Sphere(sphere_data) => sphere_data.bounding_box(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct PrimitiveIndex {
    pub primitive_type: u32,
    pub primitive_id: u32,
}
