use crate::math::{degree_to_radian, nearly_same_direction};
use getset::{CopyGetters, Getters};
use nalgebra::{Point3, UnitQuaternion, UnitVector3, Vector2, Vector3};
use std::f32::consts::PI;

#[derive(Getters, CopyGetters)]
pub struct Camera {
    #[getset(get = "pub")]
    position: Point3<f32>, // Point camera is looking from
    #[getset(get = "pub")]
    rotation: UnitQuaternion<f32>,

    #[getset(get_copy = "pub")]
    vfov: f32, // Vertical view angle (field of view)
    up: UnitVector3<f32>, // Camera-relative "up" direction

    #[getset(get_copy = "pub")]
    focus_distance: f32, // Distance from camera look-from point to plane of perfect focus
    #[getset(get_copy = "pub")]
    defocus_angle: f32, // Variation angle of rays through each pixel

    movement_speed: f32,
    rotation_scale: Vector3<f32>,

    // Camera frame basis vectors
    #[getset(get = "pub")]
    u: UnitVector3<f32>, // 相机朝向的右侧
    #[getset(get = "pub")]
    v: UnitVector3<f32>, // 相机朝向的上方
    #[getset(get = "pub")]
    w: UnitVector3<f32>, // 相机朝向的后方
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Default::default(),
            rotation: Default::default(),
            vfov: 0.0,
            up: Vector3::y_axis(),
            focus_distance: 0.0,
            defocus_angle: 0.0,
            movement_speed: 0.0,
            rotation_scale: Default::default(),
            u: Vector3::x_axis(),
            v: Vector3::y_axis(),
            w: Vector3::z_axis(),
        }
    }
}

pub struct CameraParameter {
    pub position: Point3<f32>,
    pub look_at: Point3<f32>,
    pub vfov: f32,
    pub up: UnitVector3<f32>,
    pub focus_distance: f32,
    pub defocus_angle: f32,
    pub movement_speed: f32,
    pub rotation_scale: Vector3<f32>,
}

impl Default for CameraParameter {
    fn default() -> Self {
        Self {
            position: Default::default(),
            look_at: Default::default(),
            vfov: 0.0,
            up: Vector3::y_axis(),
            focus_distance: 0.0,
            defocus_angle: 0.0,
            movement_speed: 0.0,
            rotation_scale: Default::default(),
        }
    }
}

impl Camera {
    pub fn new(parameter: &CameraParameter) -> Self {
        let rotation = UnitQuaternion::rotation_between(&Vector3::z_axis(), &(parameter.position - parameter.look_at))
            // rotation_between 在两个方向共线且方向相反时会返回 None ，因为此时的旋转不唯一
            .unwrap_or(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI));
        let mut camera = Camera {
            position: parameter.position,
            rotation,
            vfov: parameter.vfov,
            up: parameter.up,
            focus_distance: parameter.focus_distance,
            defocus_angle: parameter.defocus_angle,
            movement_speed: parameter.movement_speed,
            rotation_scale: parameter.rotation_scale,
            u: Vector3::x_axis(),
            v: Vector3::y_axis(),
            w: Vector3::z_axis(),
        };

        camera.update_camera_frame();
        camera
    }

    pub fn translate(&mut self, delta_time: Vector3<f32>) {
        let movement_distance = delta_time * self.movement_speed;
        self.position +=
            self.u.scale(movement_distance.x) + self.v.scale(movement_distance.y) - self.w.scale(movement_distance.z);
    }

    pub fn rotate(&mut self, delta: &Vector2<f32>) {
        // 避免相机朝向与 up 方向重叠，否则将无法计算出 camera frame ，但尽可能让相机可旋转

        let mut rotation_changed = false;
        // 向左旋转
        rotation_changed |= self.try_rotate(&UnitQuaternion::from_axis_angle(
            &self.v,
            degree_to_radian(delta.x * self.rotation_scale.x),
        ));

        // 向上旋转
        rotation_changed |= self.try_rotate(&UnitQuaternion::from_axis_angle(
            &self.u,
            degree_to_radian(delta.y * self.rotation_scale.y),
        ));

        if rotation_changed {
            self.update_camera_frame();
        }
    }

    fn try_rotate(&mut self, rotation: &UnitQuaternion<f32>) -> bool {
        let new_rotation = rotation * self.rotation;
        if !self.nearly_up(&UnitVector3::new_unchecked(
            new_rotation.transform_vector(&-Vector3::z_axis()),
        )) {
            self.rotation = new_rotation;
            return true;
        }
        false
    }

    fn nearly_up(&self, unit_vector: &UnitVector3<f32>) -> bool {
        nearly_same_direction(&self.up, &unit_vector)
    }

    fn update_camera_frame(&mut self) {
        self.w = self.rotation * Vector3::z_axis();
        self.u = UnitVector3::new_normalize(self.up.cross(&self.w));
        self.v = UnitVector3::new_normalize(self.w.cross(&self.u));
    }
}
