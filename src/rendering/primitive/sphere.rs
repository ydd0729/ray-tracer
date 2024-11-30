use crate::rendering::aabb::AxisAlignedBoundingBox;
use crate::rendering::material::Material;
use crate::rendering::primitive::{Primitive, PrimitiveProvider, Transformable};
use bytemuck::{Pod, Zeroable};
use nalgebra::{Point3, Scale3, Translation3, UnitQuaternion, Vector3};
use std::rc::Rc;

pub struct Sphere {
    center: Point3<f32>,
    radius: f32,
    material: Rc<dyn Material>,
    primitive: Option<Rc<Primitive>>,
    bounding_box: Option<Rc<AxisAlignedBoundingBox>>,
}

impl Sphere {
    pub fn new(center: Point3<f32>, radius: f32, material: Rc<dyn Material>) -> Self {
        Self {
            center,
            radius,
            material,
            primitive: None,
            bounding_box: None,
        }
    }
}

impl Transformable for Sphere {
    fn translate(&mut self, translation: Translation3<f32>) {
        self.center = translation * self.center;
        self.primitive = None;
        self.bounding_box = None;
    }

    fn rotate(&mut self, rotation: UnitQuaternion<f32>) {
        todo!()
    }

    fn scale(&mut self, scale: Scale3<f32>) {
        todo!()
    }
}

impl PrimitiveProvider for Sphere {
    fn primitives(&mut self, primitives: &mut Vec<Rc<Primitive>>) {
        if self.primitive.is_none() {
            self.primitive = Some(Rc::new(Primitive::Sphere(SphereData::new(
                self.center,
                self.radius,
                self.material.material_type(),
                self.material.material_id(),
            ))));
        }
        primitives.push(Rc::clone(self.primitive.as_ref().unwrap()));
    }

    fn bounding_box(&mut self, boxes: &mut Vec<Rc<AxisAlignedBoundingBox>>) {
        if self.bounding_box.is_none() {
            let r_vec = Vector3::new(self.radius, self.radius, self.radius);

            self.bounding_box = Some(Rc::new(AxisAlignedBoundingBox::new_from_points(
                self.center - r_vec,
                self.center + r_vec,
            )))
        }
        boxes.push(self.bounding_box.as_ref().unwrap().clone());
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
