use crate::math::degree_to_radian;
use crate::rendering::aabb::AxisAlignedBoundingBox;
use crate::rendering::material::{DebugNormal, Material};
use crate::rendering::primitive::sphere::Sphere;
use crate::rendering::primitive::{Primitive, PrimitiveProvider, Quad, Transformable, TransformableCollection};
use nalgebra::{Point3, UnitQuaternion, Vector3};
use std::rc::Rc;

#[derive(Default)]
pub struct Scene {
    pub camera_initial_position: Point3<f32>,
    pub camera_initial_look_at: Point3<f32>,
    pub camera_initial_movement_speed: f32,
    pub camera_initial_rotation_scale: Vector3<f32>,
    pub objects: TransformableCollection,
}

impl Scene {
    #[allow(unused)]
    pub fn scene_quad() -> Self {
        let mut objects = TransformableCollection::new();

        let debug_normal = Rc::new(DebugNormal {});

        objects.add(Quad::new(
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            debug_normal,
        ));

        Self {
            camera_initial_position: Point3::new(0.0, 0.0, 2.0),
            camera_initial_look_at: Point3::new(0.0, 0.0, 0.0),
            camera_initial_movement_speed: 1.0,
            camera_initial_rotation_scale: Vector3::new(0.2, 0.1, 0.0),
            objects,
        }
    }

    pub fn scene_primitives() -> Self {
        let mut objects = TransformableCollection::new();

        let debug_normal: Rc<dyn Material> = Rc::new(DebugNormal {});

        let mut cube = TransformableCollection::cube(Point3::origin(), 1.0, 1.0, 1.0, Rc::clone(&debug_normal));
        cube.rotate(UnitQuaternion::from_axis_angle(
            &Vector3::y_axis(),
            degree_to_radian(45.0),
        ));

        objects.add(cube);

        objects.add(Sphere::new(Point3::new(1.5, 0.0, 0.0), 0.5, Rc::clone(&debug_normal)));

        Self {
            camera_initial_position: Point3::new(0.75, 0.0, 4.0),
            camera_initial_look_at: Point3::new(0.75, 0.0, 0.0),
            camera_initial_movement_speed: 1.2,
            camera_initial_rotation_scale: Vector3::new(0.2, 0.1, 0.0),
            objects,
        }
    }
}

impl PrimitiveProvider for Scene {
    fn primitives(&mut self, primitives: &mut Vec<Rc<Primitive>>) {
        self.objects.primitives(primitives);
    }

    fn bounding_box(&mut self, boxes: &mut Vec<Rc<AxisAlignedBoundingBox>>) {
        self.objects.bounding_box(boxes);
    }
}
