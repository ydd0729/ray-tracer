use crate::rendering::mesh::Mesh;
use nalgebra::*;

pub trait Transformable {
    fn translate(&mut self, translation: Translation3<f32>);
    fn rotate(&mut self, rotation: UnitQuaternion<f32>);
    fn scale(&mut self, scale: Scale3<f32>);
}

pub trait TransformableMesh: Transformable + Mesh {}
impl<T: Transformable + Mesh> TransformableMesh for T {}