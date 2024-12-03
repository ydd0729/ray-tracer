use super::Mesh;
use crate::rendering::primitive::transformable::Transformable;
use crate::rendering::primitive::{PrimitiveData, TransformableMesh};
use nalgebra::*;
use std::rc::Rc;

pub struct TransformableMeshList {
    objects: Vec<Box<dyn TransformableMesh>>,
}

impl Default for TransformableMeshList {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformableMeshList {
    pub fn new() -> Self {
        Self { objects: Vec::new() }
    }

    pub fn add<T: TransformableMesh + 'static>(&mut self, object: T) {
        self.objects.push(Box::new(object));
    }

    pub fn add_all<T: TransformableMesh + 'static>(&mut self, objects: Vec<T>) {
        for object in objects {
            self.add(object);
        }
    }
}

impl Transformable for TransformableMeshList {
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

impl Mesh for TransformableMeshList {
    fn primitives(&mut self, primitives: &mut Vec<Rc<PrimitiveData>>, important_indices: &mut Vec<u32>) {
        self.objects
            .iter_mut()
            .for_each(|object| object.primitives(primitives, important_indices));
    }
}