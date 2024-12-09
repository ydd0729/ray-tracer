use crate::render::primitive::PrimitiveData;
use bytemuck::{Pod, Zeroable};
use std::rc::Rc;

use super::bounding_box::BoundingBox;

#[repr(C)]
#[derive(Copy, Clone, Zeroable, Pod, Debug)]
pub struct BvhNode {
    pub left_or_primitive_type: u32,
    pub right_or_primitive_id: u32,
    pub parent: u32,
    pub is_leaf: u32,
    pub bounding_box: BoundingBox,
}

pub struct BvhBuildingEntry {
    pub primitive: Rc<PrimitiveData>,
    pub primitive_type: u32,
    pub primitive_id: u32,
    pub bounding_box: BoundingBox,
}

pub fn build_bvh_tree(
    tree: &mut Vec<BvhNode>,
    primitives: &mut [BvhBuildingEntry],
    start: usize,
    end: usize,
    parent: usize,
) -> u32 {
    let len = end - start;
    if len == 0 {
        return 0;
    }

    let is_leaf = len == 1;
    let median = len / 2;
    let id = tree.len();

    let mut bounding_box = BoundingBox::empty();
    for primitive in &primitives[start..end] {
        bounding_box.merge(&primitive.bounding_box);
    }

    if is_leaf {
        tree.push(BvhNode {
            left_or_primitive_type: primitives[start + median].primitive_type,
            right_or_primitive_id: primitives[start + median].primitive_id,
            parent: parent as u32,
            is_leaf: 1,
            bounding_box,
        });
    } else {
        tree.push(BvhNode {
            left_or_primitive_type: 0,
            right_or_primitive_id: 0,
            parent: parent as u32,
            is_leaf: 0,
            bounding_box,
        });

        let longest_axis = bounding_box.longest_axis();

        let _ = &mut primitives[start..end].select_nth_unstable_by(median, |a, b| {
            let a_min = a.bounding_box.axis(longest_axis).min();
            let b_min = b.bounding_box.axis(longest_axis).min();
            a_min.total_cmp(b_min)
        });

        let id_left = build_bvh_tree(tree, primitives, start, start + median, id);
        let mut id_right = build_bvh_tree(tree, primitives, start + median, end, id);

        tree[id].left_or_primitive_type = id_left;
        if id_right == 0 {
            id_right = id_left
        }
        tree[id].right_or_primitive_id = id_right;
    }

    id as u32
}
