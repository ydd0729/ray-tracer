use nalgebra::UnitVector3;

// pub fn degree360(degree: f32) -> f32 {
//     degree.rem_euclid(360.0)
// }

pub fn degree_to_radian(degree: f32) -> f32 {
    degree * std::f32::consts::PI / 180f32
}

#[allow(unused)]
pub fn radian_to_degree(radian: f32) -> f32 {
    radian * 180f32 / std::f32::consts::PI
}

pub fn nearly_same_direction(a: &UnitVector3<f32>, b: &UnitVector3<f32>) -> bool {
    a.dot(b).abs() > 0.999
}
