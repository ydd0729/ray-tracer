use bytemuck::*;
use nalgebra::*;
use wgpu::{vertex_attr_array, VertexAttribute};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, Default)]
pub struct Vertex {
    pub position: Point4<f32>,
    // pub color: Point4<f32>,
    // pub normal: Vector4<f32>,
    // pub tex_coords: Point2<f32>,
}

impl Vertex {
    pub fn with_position(mut self, x: f32, y: f32, z: f32, w: f32) -> Self {
        self.position = Point4::new(x, y, z, w);
        self
    }

    // pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
    //     self.color = Point4::new(r, g, b, a);
    //     self
    // }

    // pub fn with_tex_coords(mut self, u: f32, v: f32) -> Self {
    //     self.tex_coords = Point2::new(u, v);
    //     self
    // }

    pub fn size() -> usize {
        const SIZE: usize = size_of::<Vertex>();
        SIZE
    }

    pub const fn wgpu_vertex_attributes() -> &'static [VertexAttribute] {
        const VERTEX_ATTRIBUTES: [VertexAttribute; 1] =
            vertex_attr_array![0 => Float32x4 /*, 1 => Float32x4, 2 => Float32x4, 3 => Float32x2*/];

        &VERTEX_ATTRIBUTES
    }
}
