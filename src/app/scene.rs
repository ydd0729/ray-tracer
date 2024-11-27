use crate::rendering::primitive::{Primitive, PrimitiveProvider, Quad, TransformableCollection};
use nalgebra::{Point3, Vector3};

#[derive(Default)]
pub struct Scene {
    pub camera_initial_position: Point3<f32>,
    pub camera_initial_look_at: Point3<f32>,
    pub objects: TransformableCollection,
}

impl Scene {
    pub fn scene_quad() -> Self {
        let mut objects = TransformableCollection::new();

        objects.add(Quad::new(
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        ));

        Self {
            camera_initial_position: Point3::new(0.0, 0.0, 2.0),
            camera_initial_look_at: Point3::new(0.0, 0.0, 0.0),
            objects,
        }
    }
}

impl PrimitiveProvider for Scene {
    fn primitives(&self) -> Vec<Primitive> {
        self.objects.primitives()
    }
}
