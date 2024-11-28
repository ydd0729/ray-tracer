use crate::math::degree_to_radian;
use crate::rendering::primitive::{Primitive, PrimitiveProvider, Quad, Transformable, TransformableCollection};
use nalgebra::{Point3, UnitQuaternion, Vector3};

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

    pub fn scene_cube() -> Self {
        let mut objects = TransformableCollection::new();

        let mut cube = TransformableCollection::cube(Point3::origin(), 1.0, 1.0, 1.0);
        cube.rotate(UnitQuaternion::from_axis_angle(
            &Vector3::y_axis(),
            degree_to_radian(45.0),
        ));

        objects.add(cube);

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
