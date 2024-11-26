use lazy_static::lazy_static;
use nalgebra::{UnitVector3, Vector3};

lazy_static! {
    pub static ref UNIT_X: UnitVector3<f32> = UnitVector3::new_unchecked(Vector3::x());
    pub static ref UNIT_Y: UnitVector3<f32> = UnitVector3::new_unchecked(Vector3::y());
    pub static ref UNIT_UP: UnitVector3<f32> = UnitVector3::new_unchecked(Vector3::y());
    pub static ref UNIT_Z: UnitVector3<f32> = UnitVector3::new_unchecked(Vector3::z());
    pub static ref NEG_UNIT_Z: UnitVector3<f32> = UnitVector3::new_unchecked(-Vector3::z());
}

// pub fn degree360(degree: f32) -> f32 {
//     degree.rem_euclid(360.0)
// }

pub fn degree_to_radian(degree: f32) -> f32 {
    degree * std::f32::consts::PI / 180f32
}

pub fn radian_to_degree(radian: f32) -> f32 {
    radian * 180f32 / std::f32::consts::PI
}

pub fn nearly_same_direction(a: &UnitVector3<f32>, b: &UnitVector3<f32>) -> bool {
    a.dot(&b).abs() > 0.999
}
