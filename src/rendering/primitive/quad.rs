use crate::rendering::material::MaterialHandle;
use crate::rendering::primitive::transformable::Transformable;
use crate::rendering::primitive::PrimitiveData;
use crate::rendering::{bounding_box::BoundingBox, mesh::Mesh};
use bytemuck::{Pod, Zeroable};
use nalgebra::{Point3, Scale3, Translation3, UnitQuaternion, Vector3};
use std::rc::Rc;

use super::Bound;

pub struct Quad {
    center: Point3<f32>,
    right: Vector3<f32>,
    up: Vector3<f32>,
    material_type: u32,
    material_id: u32,
    primitive: Option<Rc<PrimitiveData>>,
    important: bool,
}

impl Quad {
    pub fn new(
        center: Point3<f32>,
        right: Vector3<f32>,
        up: Vector3<f32>,
        material: MaterialHandle,
        important: bool,
    ) -> Self {
        Self {
            center,
            right,
            up,
            material_type: material.material_type,
            material_id: material.material_id,
            primitive: None,
            important,
        }
    }
}

impl Transformable for Quad {
    fn translate(&mut self, translation: Translation3<f32>) {
        self.center = translation * self.center;
        self.primitive = None;
    }

    fn rotate(&mut self, rotation: UnitQuaternion<f32>) {
        self.center = rotation * self.center;
        self.right = rotation * self.right;
        self.up = rotation * self.up;
        self.primitive = None;
    }

    fn scale(&mut self, scale: Scale3<f32>) {
        self.center = scale * self.center;
        self.right = scale * self.right;
        self.up = scale * self.up;
        self.primitive = None;
    }
}

impl Mesh for Quad {
    fn primitives(&mut self, primitives: &mut Vec<Rc<PrimitiveData>>, important_indices: &mut Vec<u32>) {
        if self.important {
            important_indices.push(primitives.len() as u32);
        }

        if self.primitive.is_none() {
            self.primitive = Some(Rc::new(PrimitiveData::Quad(QuadData::new(
                self.center,
                self.right,
                self.up,
                self.material_type,
                self.material_id,
            ))));
        }
        primitives.push(Rc::clone(self.primitive.as_ref().unwrap()));
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct QuadData {
    bottom_left: Point3<f32>,
    material_id: u32,
    right: Vector3<f32>,
    area: f32,
    up: Vector3<f32>,
    d: f32,
    normal: Vector3<f32>,
    material_type: u32,
    w: Vector3<f32>,
    _padding2: [u32; 1],
}

impl QuadData {
    pub fn new(
        center: Point3<f32>,
        right: Vector3<f32>,
        up: Vector3<f32>,
        material_type: u32,
        material_id: u32,
    ) -> Self {
        let bottom_left = center - right / 2.0 - up / 2.0;
        let n = right.cross(&up);
        let normal = n.normalize();
        let d = normal.dot(&bottom_left.coords);
        let w = normal / normal.dot(&n);
        let area = n.norm();

        Self {
            bottom_left,
            material_id,
            right,
            area,
            up,
            normal,
            d,
            w,
            material_type,
            _padding2: Default::default(),
        }
    }
}

impl Bound for QuadData {
    fn bounding_box(&self) -> BoundingBox {
        let box1 = BoundingBox::new_from_points(self.bottom_left, self.bottom_left + self.right + self.up);
        let box2 = BoundingBox::new_from_points(self.bottom_left + self.right, self.bottom_left + self.up);
        BoundingBox::new_from_boxes(&box1, &box2)
    }
}
