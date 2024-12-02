use crate::rendering::aabb::AxisAlignedBoundingBox;
use crate::rendering::mesh::Mesh;
use crate::rendering::primitive::Primitive;
use nalgebra::*;
use std::rc::Rc;

pub trait Transformable {
    fn translate(&mut self, translation: Translation3<f32>);
    fn rotate(&mut self, rotation: UnitQuaternion<f32>);
    fn scale(&mut self, scale: Scale3<f32>);
}

pub trait RenderObject: Transformable + Mesh {}
impl<T: Transformable + Mesh> RenderObject for T {}

pub struct RenderObjectList {
    objects: Vec<Box<dyn RenderObject>>,
}

impl Default for RenderObjectList {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderObjectList {
    pub fn new() -> Self {
        Self { objects: Vec::new() }
    }

    pub fn add<T: RenderObject + 'static>(&mut self, object: T) {
        self.objects.push(Box::new(object));
    }

    pub fn add_all<T: RenderObject + 'static>(&mut self, objects: Vec<T>) {
        for object in objects {
            self.add(object);
        }
    }
}

impl Transformable for RenderObjectList {
    fn translate(&mut self, translation: Translation3<f32>) {
        for object in &mut self.objects {
            object.translate(translation);
        }
    }

    fn rotate(&mut self, rotation: UnitQuaternion<f32>) {
        for object in &mut self.objects {
            object.rotate(rotation);
        }
    }

    fn scale(&mut self, scale: Scale3<f32>) {
        for object in &mut self.objects {
            object.scale(scale);
        }
    }
}

impl Mesh for RenderObjectList {
    fn primitives(&mut self, primitives: &mut Vec<Rc<Primitive>>, important_indices: &mut Vec<u32>) {
        self.objects
            .iter_mut()
            .for_each(|object| object.primitives(primitives, important_indices));
    }

    fn bounding_box(&mut self, boxes: &mut Vec<Rc<AxisAlignedBoundingBox>>) {
        self.objects.iter_mut().for_each(|object| object.bounding_box(boxes));
    }
}
