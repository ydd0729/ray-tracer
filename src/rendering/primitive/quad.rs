use crate::rendering::primitive::transformable::Transformable;
use crate::rendering::primitive::{Primitive, PrimitiveProvider, PrimitiveQuad};
use nalgebra::{Point3, Scale3, Translation3, UnitQuaternion, Vector3};

pub struct Quad {
    center: Point3<f32>,
    right: Vector3<f32>,
    up: Vector3<f32>,
}

impl Quad {
    pub(crate) fn new(center: Point3<f32>, right: Vector3<f32>, up: Vector3<f32>) -> Self {
        Self { center, right, up }
    }
}

impl Transformable for Quad {
    fn translate(&mut self, translation: Translation3<f32>) {
        self.center = translation * self.center;
    }

    fn rotate(&mut self, rotation: UnitQuaternion<f32>) {
        self.center = rotation * self.center;
        self.right = rotation * self.right;
        self.up = rotation * self.up;
    }

    fn scale(&mut self, scale: Scale3<f32>) {
        self.center = scale * self.center;
        self.right = scale * self.right;
        self.up = scale * self.up;
    }
}

impl PrimitiveProvider for Quad {
    fn primitives(&self) -> Vec<Primitive> {
        vec![Primitive::Quad(PrimitiveQuad::new(self.center, self.right, self.up))]
    }
}
