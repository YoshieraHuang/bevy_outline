use bevy::{
    core_pipeline::Transparent3d,
    pbr::{
        DrawMesh, MeshPipeline, MeshPipelineKey, MeshUniform, SetMeshBindGroup,
        SetMeshViewBindGroup,
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_asset::RenderAssets,
        render_phase::{DrawFunctions, RenderPhase, SetItemPipeline, AddRenderCommand},
        render_resource::{
            PipelineCache, RenderPipelineDescriptor, SpecializedMeshPipeline,
            SpecializedMeshPipelineError, SpecializedMeshPipelines, Face,
        },
        view::ExtractedView,
        RenderApp, RenderStage,
    },
};

pub const OUTLINE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 7053223528096556000);

#[derive(Debug, Default)]
pub struct OutlinePlugin;

impl Plugin for OutlinePlugin {
    fn build(&self, app: &mut App) {
        let mut assets = app.world.resource_mut::<Assets<_>>();
        assets.set_untracked(
            OUTLINE_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("render/outline.wgsl")),
        );

        app.init_resource::<OutlineConfig>();

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Transparent3d, DrawOutline>()
                .init_resource::<OutlinePipeline>()
                .init_resource::<SpecializedMeshPipelines<OutlinePipeline>>()
                .add_system_to_stage(RenderStage::Extract, extract_outline_config)
                .add_system_to_stage(RenderStage::Extract, extract_outline)
                .add_system_to_stage(RenderStage::Queue, queue_outlines);
        }
    }
}

#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct Outline;

#[derive(Debug, Clone, Default)]
pub struct OutlineConfig {
    global: bool,
}

fn extract_outline_config(mut commands: Commands, outline_config: Res<OutlineConfig>) {
    if outline_config.is_added() || outline_config.is_changed() {
        commands.insert_resource(outline_config.into_inner().clone());
    }
}

fn extract_outline(mut commands: Commands, query: Query<Entity, With<Outline>>) {
    for entity in query.iter() {
        commands.get_or_spawn(entity).insert(Outline);
    }
}

pub struct OutlinePipeline {
    mesh_pipeline: MeshPipeline,
    shader: Handle<Shader>,
}

impl FromWorld for OutlinePipeline {
    fn from_world(render_world: &mut World) -> Self {
        Self {
            mesh_pipeline: render_world.resource::<MeshPipeline>().clone(),
            shader: OUTLINE_SHADER_HANDLE.typed(),
        }
    }
}

impl SpecializedMeshPipeline for OutlinePipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;
        descriptor.vertex.shader = self.shader.clone_weak();
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone_weak();
        descriptor.primitive.cull_mode = Some(Face::Front);
        Ok(descriptor)
    }
}

fn queue_outlines(
    opaque_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    render_meshes: Res<RenderAssets<Mesh>>,
    // outline_config: Res<OutlineConfig>,
    outline_pipeline: Res<OutlinePipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<OutlinePipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    msaa: Res<Msaa>,
    mut material_meshes: Query<(Entity, &Handle<Mesh>, &MeshUniform), With<Outline>>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
) {
    let draw_custom = opaque_3d_draw_functions
        .read()
        .get_id::<DrawOutline>()
        .unwrap();

    let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples);

    for (view, mut opaque_phase) in views.iter_mut() {
        let view_matrix = view.transform.compute_matrix();
        let view_row_2 = view_matrix.row(2);

        for (entity, mesh_handle, mesh_uniform) in material_meshes.iter_mut() {
            if let Some(mesh) = render_meshes.get(mesh_handle) {
                let key =
                    msaa_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology);
                let pipeline =
                    pipelines.specialize(&mut pipeline_cache, &outline_pipeline, key, &mesh.layout);
                let pipeline = match pipeline {
                    Ok(id) => id,
                    Err(err) => {
                        error!("{}", err);
                        return;
                    }
                };
                opaque_phase.add(Transparent3d {
                    entity,
                    pipeline,
                    draw_function: draw_custom,
                    distance: view_row_2.dot(mesh_uniform.transform.col(3)),
                })
            }
        }
    }
}

type DrawOutline = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    DrawMesh,
);
