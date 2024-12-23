use crate::math::degree_to_radian;
use crate::rendering::primitive::Transformable;
use crate::rendering::material::{DebugNormal, Dielectric, DiffuseLight, Lambertian, MaterialList};
use crate::rendering::mesh::mesh_list::TransformableMeshList;
use crate::rendering::mesh::Mesh;
use crate::rendering::primitive::sphere::Sphere;
use crate::rendering::primitive::{PrimitiveData, Quad};
use log::info;
use nalgebra::{Point3, Translation3, UnitQuaternion, Vector3};
use std::rc::Rc;

use super::camera::CameraParameters;

#[derive(Default)]
pub struct Scene {
    pub camera_parameters: CameraParameters,
    pub objects: TransformableMeshList,
    pub materials: MaterialList,
}

impl Scene {
    #[allow(unused)]
    pub fn scene_quad() -> Self {
        let mut materials = MaterialList::default();
        let debug_normal = materials.add(Box::new(DebugNormal {}));

        let mut objects = TransformableMeshList::new();

        objects.add(Quad::new(
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            debug_normal,
            false,
        ));

        let camera_parameters = CameraParameters {
            initial_position: Point3::new(0.0, 0.0, 2.0),
            initial_look_at: Point3::new(0.0, 0.0, 0.0),
            vfov: 45.0,
            up: Vector3::y_axis(),
            focus_distance: 1.0,
            defocus_angle: 0.0,
            movement_speed: 1.0,
            rotation_scale: 0.2,
        };

        Self {
            camera_parameters,
            objects,
            materials,
        }
    }

    #[allow(unused)]
    pub fn scene_primitives() -> Self {
        let mut materials = MaterialList::default();
        let debug_normal = materials.add(Box::new(DebugNormal {}));

        let mut objects = TransformableMeshList::new();

        let mut cube = TransformableMeshList::cube(Point3::origin(), 1.0, 1.0, 1.0, debug_normal, false);
        cube.rotate(UnitQuaternion::from_axis_angle(
            &Vector3::y_axis(),
            degree_to_radian(45.0),
        ));

        objects.add(cube);
        objects.add(Sphere::new(Point3::new(1.5, 0.0, 0.0), 0.5, debug_normal, false));

        let camera_parameters = CameraParameters {
            initial_position: Point3::new(0.75, 0.0, 4.0),
            initial_look_at: Point3::new(0.75, 0.0, 0.0),
            vfov: 45.0,
            up: Vector3::y_axis(),
            focus_distance: 1.0,
            defocus_angle: 0.0,
            movement_speed: 1.2,
            rotation_scale: 0.2,
        };

        Self {
            camera_parameters,
            objects,
            materials,
        }
    }

    #[allow(unused)]
    pub fn scene_light() -> Self {
        let mut materials = MaterialList::default();
        let lambertian_red = materials.add(Box::new(Lambertian::new(Point3::new(0.65, 0.05, 0.05))));
        info!("{:?}", lambertian_red);
        let lambertian_white = materials.add(Box::new(Lambertian::new(Point3::new(0.73, 0.73, 0.73))));
        info!("{:?}", lambertian_white);
        let lambertian_green = materials.add(Box::new(Lambertian::new(Point3::new(0.12, 0.45, 0.15))));
        info!("{:?}", lambertian_green);

        let light = materials.add(Box::new(DiffuseLight::new(Point3::new(1.0, 1.0, 1.0))));
        info!("{:?}", light);

        let mut objects = TransformableMeshList::new();

        objects.add(Quad::new(
            Point3::new(0.0, 1.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            light,
            true,
        ));

        objects.add(Quad::new(
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(-2.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 2.0),
            lambertian_green,
            false,
        ));

        let mut cube = TransformableMeshList::cube(Point3::new(0.0, 0.4, 0.0), 0.5, 0.6, 0.5, lambertian_white, false);
        cube.rotate(UnitQuaternion::from_axis_angle(
            &Vector3::y_axis(),
            degree_to_radian(15.0),
        ));

        objects.add(cube);

        let camera_parameters = CameraParameters {
            initial_position: Point3::new(0.0, 0.5, 2.0),
            initial_look_at: Point3::new(0.0, 0.5, 0.0),
            vfov: 40.0,
            up: Vector3::y_axis(),
            focus_distance: 1.0,
            defocus_angle: 0.0,
            movement_speed: 1.0,
            rotation_scale: 0.2,
        };

        Self {
            camera_parameters,
            objects,
            materials,
        }
    }

    #[allow(unused)]
    pub fn scene_light_huge() -> Self {
        let mut materials = MaterialList::default();
        let lambertian_red = materials.add(Box::new(Lambertian::new(Point3::new(0.65, 0.05, 0.05))));
        info!("{:?}", lambertian_red);
        let lambertian_white = materials.add(Box::new(Lambertian::new(Point3::new(0.73, 0.73, 0.73))));
        info!("{:?}", lambertian_white);
        let lambertian_green = materials.add(Box::new(Lambertian::new(Point3::new(0.12, 0.45, 0.15))));
        info!("{:?}", lambertian_green);

        let light = materials.add(Box::new(DiffuseLight::new(Point3::new(1.0, 1.0, 1.0))));
        info!("{:?}", light);

        let mut objects = TransformableMeshList::new();

        objects.add(Quad::new(
            Point3::new(0.0, 100.0, 0.0),
            Vector3::new(100.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 100.0),
            light,
            true,
        ));

        objects.add(Quad::new(
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(-200.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 200.0),
            lambertian_green,
            false,
        ));

        objects.add(Quad::new(
            Point3::new(0.0, 0.0, -100.0),
            Vector3::new(-200.0, 0.0, 0.0),
            Vector3::new(0.0, 200.0, 0.0),
            lambertian_red,
            false,
        ));

        let mut cube =
            TransformableMeshList::cube(Point3::new(0.0, 40.0, 0.0), 50.0, 60.0, 50.0, lambertian_white, false);
        cube.rotate(UnitQuaternion::from_axis_angle(
            &Vector3::y_axis(),
            degree_to_radian(15.0),
        ));

        objects.add(cube);

        let camera_parameters = CameraParameters {
            initial_position: Point3::new(0.0, 50.0, 200.0),
            initial_look_at: Point3::new(0.0, 50.0, 0.0),
            vfov: 40.0,
            up: Vector3::y_axis(),
            focus_distance: 1.0,
            defocus_angle: 0.0,
            movement_speed: 100.0,
            rotation_scale: 0.2,
        };

        Self {
            camera_parameters,
            objects,
            materials,
        }
    }

    #[allow(unused)]
    pub fn scene_cornell_box() -> Self {
        let mut materials = MaterialList::default();

        let lambertian_red = materials.add(Box::new(Lambertian::new(Point3::new(0.65, 0.05, 0.05))));
        let lambertian_white = materials.add(Box::new(Lambertian::new(Point3::new(0.73, 0.73, 0.73))));
        let lambertian_green = materials.add(Box::new(Lambertian::new(Point3::new(0.12, 0.45, 0.15))));
        let light = materials.add(Box::new(DiffuseLight::new(Point3::new(15.0, 15.0, 15.0))));
        let dielectric = materials.add(Box::new(Dielectric::new(1.5)));

        let mut objects = TransformableMeshList::new();

        // Light
        // objects.add(Quad::new(
        //     Point3::new(2.780, 5.540, 2.795),
        //     Vector3::new(1.300, 0.0, 0.0),
        //     Vector3::new(0.0, 0.0, 1.050),
        //     light,
        //     true,
        // ));

        // Cornell box sides
        objects.add(Quad::new(
            Point3::new(5.550, 2.775, 2.775),
            Vector3::new(0.0, 0.0, 5.550),
            Vector3::new(0.0, 5.550, 0.0),
            lambertian_green,
            false,
        ));
        objects.add(Quad::new(
            Point3::new(0.0, 2.775, 2.775),
            Vector3::new(0.0, 0.0, -5.550),
            Vector3::new(0.0, 5.550, 0.0),
            lambertian_red,
            false,
        ));
        objects.add(Quad::new(
            Point3::new(2.775, 5.550, 2.775),
            Vector3::new(5.550, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 5.550),
            lambertian_white,
            false,
        ));
        objects.add(Quad::new(
            Point3::new(2.775, 0.0, 2.775),
            Vector3::new(5.550, 0.0, 0.0),
            Vector3::new(0.0, 0.0, -5.550),
            lambertian_white,
            false,
        ));
        objects.add(Quad::new(
            Point3::new(2.775, 2.775, 5.550),
            Vector3::new(-5.550, 0.0, 0.0),
            Vector3::new(0.0, 5.550, 0.0),
            lambertian_white,
            false,
        ));

        objects.add(Sphere::new(Point3::new(1.900, 0.900, 1.900), 0.900, dielectric, true));

        // Light
        objects.add(Quad::new(
            Point3::new(2.780, 5.540, 2.795),
            Vector3::new(1.300, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.050),
            light,
            true,
        ));

        let mut cube = TransformableMeshList::cube(
            Point3::new(0.825, 1.650, 0.825),
            1.650,
            3.300,
            1.650,
            lambertian_white,
            false,
        );
        cube.rotate(UnitQuaternion::from_axis_angle(
            &Vector3::y_axis(),
            degree_to_radian(15.0),
        ));
        cube.translate(Translation3::new(2.650, 0.0, 2.950));

        objects.add(cube);

        let camera_parameters = CameraParameters {
            initial_position: Point3::new(2.780, 2.780, -8.000),
            initial_look_at: Point3::new(2.780, 2.780, 0.0),
            vfov: 40.0,
            up: Vector3::y_axis(),
            focus_distance: 1.0,
            defocus_angle: 0.0,
            movement_speed: 2.0,
            rotation_scale: 0.2,
        };

        Self {
            camera_parameters,
            objects,
            materials,
        }
    }
}

impl Mesh for Scene {
    fn primitives(&mut self, primitives: &mut Vec<Rc<PrimitiveData>>, important_indices: &mut Vec<u32>) {
        self.objects.primitives(primitives, important_indices);
    }
}
