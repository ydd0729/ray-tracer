use std::rc::Rc;

use super::{aabb::AxisAlignedBoundingBox, primitive::Primitive};

pub mod cube;

pub trait Mesh {
    fn primitives(&mut self, primitives: &mut Vec<Rc<Primitive>>, important_indices: &mut Vec<u32>);
    fn bounding_box(&mut self, boxes: &mut Vec<Rc<AxisAlignedBoundingBox>>);
}