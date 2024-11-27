use crate::rendering::primitive::quad::Quad;
use crate::rendering::primitive::transformable::{Transformable, TransformableCollection};
use nalgebra::*;

impl TransformableCollection {
    pub fn cube(center: Point3<f32>, x_extent: f32, y_extent: f32, z_extent: f32) -> Self {
        let half_x_extent = x_extent / 2.0;
        let half_y_extent = y_extent / 2.0;
        let half_z_extent = z_extent / 2.0;

        let mut front = Quad::new(center, Vector3::x() * x_extent, Vector3::y() * y_extent);
        front.translate(Translation3::new(0.0, 0.0, half_z_extent));

        let mut back = Quad::new(center, -Vector3::x() * x_extent, Vector3::y() * y_extent);
        back.translate(Translation3::new(0.0, 0.0, -half_z_extent));

        let mut left = Quad::new(center, -Vector3::z() * z_extent, Vector3::y() * y_extent);
        left.translate(Translation3::new(half_x_extent, 0.0, 0.0));

        let mut right = Quad::new(center, Vector3::z() * z_extent, Vector3::y() * y_extent);
        right.translate(Translation3::new(-half_x_extent, 0.0, 0.0));

        let mut up = Quad::new(center, -Vector3::x() * x_extent, Vector3::z() * z_extent);
        up.translate(Translation3::new(0.0, half_y_extent, 0.0));

        let mut bottom = Quad::new(center, Vector3::x() * x_extent, Vector3::z() * z_extent);
        bottom.translate(Translation3::new(0.0, -half_y_extent, 0.0));

        let mut cube = TransformableCollection::new();
        cube.add_vec(vec![front, back, left, right, up, bottom]);

        cube
    }
}