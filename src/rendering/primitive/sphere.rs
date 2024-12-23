use crate::rendering::bounding_box::BoundingBox;
use crate::rendering::material::MaterialHandle;
use crate::rendering::mesh::Mesh;
use crate::rendering::primitive::{PrimitiveData, Transformable};
use bytemuck::{Pod, Zeroable};
use nalgebra::{Point3, Scale3, Translation3, UnitQuaternion, Vector3};
use std::rc::Rc;

use super::Bound;

pub struct Sphere {
    center: Point3<f32>,
    radius: f32,
    material_id: u32,
    material_type: u32,
    primitive: Option<Rc<PrimitiveData>>,
    important: bool,
}

impl Sphere {
    pub fn new(center: Point3<f32>, radius: f32, material: MaterialHandle, important: bool) -> Self {
        Self {
            center,
            radius,
            material_type: material.material_type,
            material_id: material.material_id,
            primitive: None,
            important,
        }
    }
}

impl Transformable for Sphere {
    fn translate(&mut self, translation: Translation3<f32>) {
        self.center = translation * self.center;
        self.primitive = None;
    }

    fn rotate(&mut self, _rotation: UnitQuaternion<f32>) {
        todo!()
    }

    fn scale(&mut self, _scale: Scale3<f32>) {
        todo!()
    }
}

impl Mesh for Sphere {
    fn primitives(&mut self, primitives: &mut Vec<Rc<PrimitiveData>>, important_indices: &mut Vec<u32>) {
        if self.important {
            important_indices.push(primitives.len() as u32);
        }

        if self.primitive.is_none() {
            self.primitive = Some(Rc::new(PrimitiveData::Sphere(SphereData::new(
                self.center,
                self.radius,
                self.material_type,
                self.material_id,
            ))));
        }
        primitives.push(Rc::clone(self.primitive.as_ref().unwrap()));
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SphereData {
    center: Point3<f32>,
    radius: f32,
    primitive_type: u32,
    primitive_id: u32,
    _padding: [u32; 2],
}

impl SphereData {
    pub fn new(center: Point3<f32>, radius: f32, primitive_type: u32, primitive_id: u32) -> Self {
        Self {
            center,
            radius,
            primitive_type,
            primitive_id,
            _padding: [0; 2],
        }
    }
}

impl Bound for SphereData {
    fn bounding_box(&self) -> BoundingBox {
        let r_vec = Vector3::new(self.radius, self.radius, self.radius);
        BoundingBox::new_from_points(self.center - r_vec, self.center + r_vec)
    }
}
