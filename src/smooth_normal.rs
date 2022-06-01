use std::hash::Hash;

use bevy::{
    math::Vec3A,
    prelude::{Deref, DerefMut},
    render::mesh::{Mesh, VertexAttributeValues},
    utils::HashMap,
};
use ordered_float::OrderedFloat;

/// An ordered float3 array which implements Eq and Hash
#[derive(Debug, Clone, Copy, Deref, DerefMut, Default)]
#[repr(transparent)]
struct OrderedFloat3([f32; 3]);

impl PartialEq for OrderedFloat3 {
    fn eq(&self, other: &Self) -> bool {
        OrderedFloat(self[0]).eq(&OrderedFloat(other[0]))
            && OrderedFloat(self[1]).eq(&OrderedFloat(other[1]))
            && OrderedFloat(self[2]).eq(&OrderedFloat(other[2]))
    }
}
impl Eq for OrderedFloat3 {}

impl Hash for OrderedFloat3 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        OrderedFloat(self[0]).hash(state);
        OrderedFloat(self[1]).hash(state);
        OrderedFloat(self[2]).hash(state);
    }
}

/// smooth the normals of vertex at same position
pub(crate) fn smooth_normal(mesh: &Mesh) -> VertexAttributeValues {
    let mut normals_map = HashMap::new();
    let v_positions = get_float3x3(mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap());
    let v_normals = get_float3x3(mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap());
    let mut smoothed_normals = vec![[0.; 3]; v_positions.len()];
    v_positions
        .iter()
        .zip(v_normals.iter())
        .enumerate()
        .for_each(|(index, (pos, normal))| {
            let key = OrderedFloat3(*pos);
            let entry = normals_map.entry(key).or_insert((vec![], Vec3A::ZERO));
            entry.0.push(index);
            entry.1 += Vec3A::from(*normal);
        });
    normals_map.drain().for_each(|(_, (indices, normal))| {
        let ave_normal = normal.normalize().into();
        indices.into_iter().for_each(|index| {
            smoothed_normals[index] = ave_normal;
        })
    });
    VertexAttributeValues::Float32x3(smoothed_normals)
}

#[inline(always)]
fn get_float3x3(values: &VertexAttributeValues) -> &Vec<[f32; 3]> {
    match values {
        VertexAttributeValues::Float32x3(v) => v,
        _ => panic!("Vertex Position must be a Float32x3"),
    }
}
