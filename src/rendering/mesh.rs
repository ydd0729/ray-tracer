use std::rc::Rc;

use super::primitive::PrimitiveData;

pub mod cube;
pub mod mesh_list;

pub trait Mesh {
    fn primitives(&mut self, primitives: &mut Vec<Rc<PrimitiveData>>, important_indices: &mut Vec<u32>);
}
