pub mod cube;
pub mod quad;
pub mod sphere;
pub mod transformable;

pub use quad::*;
use std::rc::Rc;
pub use transformable::*;

use crate::rendering::aabb::AxisAlignedBoundingBox;
use crate::rendering::primitive::sphere::SphereData;
use bytemuck::{Pod, Zeroable};

pub trait PrimitiveProvider {
    fn primitives(&mut self, primitives: &mut Vec<Rc<Primitive>>);
    fn bounding_box(&mut self, boxes: &mut Vec<Rc<AxisAlignedBoundingBox>>);
}

#[derive(Debug, Copy, Clone)]
pub enum Primitive {
    Quad(QuadData),
    Sphere(SphereData),
}

impl From<Primitive> for u32 {
    fn from(value: Primitive) -> Self {
        match value {
            Primitive::Quad(_) => 0,
            Primitive::Sphere(_) => 1,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct PrimitiveData {
    pub primitive_type: u32,
    pub primitive_id: u32,
}
