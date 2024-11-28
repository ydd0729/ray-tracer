pub mod cube;
pub mod quad;
mod sphere;
pub mod transformable;

pub use quad::*;
pub use transformable::*;

use bytemuck::{Pod, Zeroable};
use nalgebra::{Point3, Vector3};

pub trait PrimitiveProvider {
    fn primitives(&self) -> Vec<Primitive>;
}

#[derive(Debug)]
pub enum Primitive {
    Quad(PrimitiveQuad),
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PrimitiveQuad {
    bottom_left: Point3<f32>,
    material_id: u32,
    right: Vector3<f32>,
    area: f32,
    up: Vector3<f32>,
    d: f32,
    normal: Vector3<f32>,
    _padding1: [u32; 1],
    w: Vector3<f32>,
    _padding2: [u32; 1],
}

impl PrimitiveQuad {
    pub fn new(center: Point3<f32>, right: Vector3<f32>, up: Vector3<f32>) -> Self {
        let bottom_left = center - right / 2.0 - up / 2.0;
        let normal = right.cross(&up).normalize();
        let d = normal.dot(&bottom_left.coords);
        let w = normal / normal.dot(&normal);
        let area = normal.norm_squared();

        Self {
            bottom_left,
            material_id: 0,
            right,
            area,
            up,
            normal,
            d,
            w,
            _padding1: Default::default(),
            _padding2: Default::default(),
        }
    }
}
