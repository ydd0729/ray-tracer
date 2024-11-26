use crate::rendering::primitive::{Primitive, PrimitiveProvider};
use nalgebra::*;

pub trait Transformable {
    fn translate(&mut self, translation: Translation3<f32>);
    fn rotate(&mut self, rotation: UnitQuaternion<f32>);
    fn scale(&mut self, scale: Scale3<f32>);
}

pub trait TransformablePrimitiveProvider: Transformable + PrimitiveProvider {}
impl<T: Transformable + PrimitiveProvider> TransformablePrimitiveProvider for T {}

pub struct TransformableCollection {
    objects: Vec<Box<dyn TransformablePrimitiveProvider>>,
}

impl Default for TransformableCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformableCollection {
    pub fn new() -> Self {
        Self { objects: Vec::new() }
    }

    pub fn add<T: TransformablePrimitiveProvider + 'static>(&mut self, object: T) {
        self.objects.push(Box::new(object));
    }

    pub fn add_vec<T: TransformablePrimitiveProvider + 'static>(&mut self, objects: Vec<T>) {
        for object in objects {
            self.add(object);
        }
    }
}

impl Transformable for TransformableCollection {
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

impl PrimitiveProvider for TransformableCollection {
    fn primitives(&self) -> Vec<Primitive> {
        self.objects.iter().flat_map(|obj| obj.primitives()).collect()
    }
}
