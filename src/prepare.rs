use bevy::{
    prelude::{Assets, Component, Handle, Mesh, Query, ResMut, With},
    render::mesh::VertexAttributeValues,
};

use crate::{smooth_normal::smooth_normal, OutlineMaterial, ATTRIBUTE_OUTLINE_NORMAL};

#[derive(Component, Clone)]
pub struct OutlineNormals(pub VertexAttributeValues);

pub fn prepare_outline_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    outline_without_normals: Query<&Handle<Mesh>, With<Handle<OutlineMaterial>>>,
) {
    for mesh_handle in outline_without_normals.iter() {
        if let Some(mesh) = meshes.get_mut(mesh_handle) {
            // Don't have outline normal, just compute it.
            if !mesh.contains_attribute(ATTRIBUTE_OUTLINE_NORMAL) {
                mesh.insert_attribute(ATTRIBUTE_OUTLINE_NORMAL, smooth_normal(mesh));
            }
        }
    }
}
