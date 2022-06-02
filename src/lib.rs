#![doc = include_str!("../README.md")]

mod prepare;
mod smooth_normal;
mod window_size;

use bevy::{
    core_pipeline::Opaque3d,
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemParamItem,
    },
    pbr::{
        DrawMesh, MeshPipeline, MeshPipelineKey, MeshUniform, SetMeshBindGroup,
        SetMeshViewBindGroup,
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{MeshVertexAttribute, MeshVertexBufferLayout},
        render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssets},
        render_component::ExtractComponentPlugin,
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            std140::{AsStd140, Std140},
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState,
            BufferBindingType, BufferDescriptor, BufferInitDescriptor, BufferSize, CompareFunction,
            DepthBiasState, DepthStencilState, Face, FragmentState, FrontFace, MultisampleState,
            PipelineCache, PolygonMode, PrimitiveState, RenderPipelineDescriptor, ShaderStages,
            SpecializedMeshPipeline, SpecializedMeshPipelineError, SpecializedMeshPipelines,
            StencilFaceState, StencilState, TextureFormat, VertexState,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
        view::ExtractedView,
        RenderApp, RenderStage,
    },
};
use wgpu_types::{BufferUsages, ColorTargetState, ColorWrites, VertexFormat};
use window_size::{DoubleReciprocalWindowSizeUniform, SetWindowSizeBindGroup};

use crate::{
    prepare::prepare_outline_mesh,
    window_size::{
        extract_window_size, prepare_window_size, queue_window_size_bind_group,
        DoubleReciprocalWindowSizeMeta,
    },
};

macro_rules! load_internal_asset {
    ($app: ident, $handle: ident, $path_str: expr, $loader: expr) => {{
        let mut assets = $app.world.resource_mut::<bevy::asset::Assets<_>>();
        assets.set_untracked($handle, ($loader)(include_str!($path_str)));
    }};
}

pub const OUTLINE_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 7053223528096556000);

pub const ATTRIBUTE_OUTLINE_NORMAL: MeshVertexAttribute =
    MeshVertexAttribute::new("OutlineNormal", 9885409170, VertexFormat::Float32x3);

#[derive(Debug, Default)]
pub struct OutlinePlugin;

impl Plugin for OutlinePlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            OUTLINE_SHADER_HANDLE,
            "render/outline.wgsl",
            Shader::from_wgsl
        );

        let render_device = app.world.resource::<RenderDevice>();
        let buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("window size uniform buffer"),
            size: DoubleReciprocalWindowSizeUniform::std140_size_static() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        app.add_asset::<OutlineMaterial>()
            .add_plugin(ExtractComponentPlugin::<Handle<OutlineMaterial>>::default())
            .add_plugin(RenderAssetPlugin::<OutlineMaterial>::default())
            .add_system_to_stage(CoreStage::PostUpdate, prepare_outline_mesh);

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Opaque3d, DrawOutlines>()
                .insert_resource(DoubleReciprocalWindowSizeMeta {
                    buffer,
                    bind_group: None,
                })
                .init_resource::<OutlinePipeline>()
                .init_resource::<SpecializedMeshPipelines<OutlinePipeline>>()
                .add_system_to_stage(RenderStage::Extract, extract_window_size)
                .add_system_to_stage(RenderStage::Prepare, prepare_window_size)
                .add_system_to_stage(RenderStage::Queue, queue_outlines)
                .add_system_to_stage(RenderStage::Queue, queue_window_size_bind_group);
        }
    }
}

#[derive(TypeUuid, Clone)]
#[uuid = "f31fac68-fd87-44db-a4c5-eed0bcbb96cd"]
pub struct OutlineMaterial {
    pub width: f32,
    pub color: Color,
}

#[derive(AsStd140)]
struct OutlineMaterialUniform {
    width: f32,
    color: Vec4,
}

pub struct GpuOutlineMaterial {
    bind_group: BindGroup,
}

impl RenderAsset for OutlineMaterial {
    type ExtractedAsset = OutlineMaterial;
    type PreparedAsset = GpuOutlineMaterial;
    type Param = (SRes<RenderDevice>, SRes<OutlinePipeline>);

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let uniform = OutlineMaterialUniform {
            width: extracted_asset.width,
            color: extracted_asset.color.as_linear_rgba_f32().into(),
        };

        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: None,
            contents: uniform.as_std140().as_bytes(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipeline.material_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        Ok(GpuOutlineMaterial { bind_group })
    }
}

pub struct OutlinePipeline {
    pub mesh_layout: BindGroupLayout,
    pub view_layout: BindGroupLayout,
    pub material_layout: BindGroupLayout,
    pub window_size_layout: BindGroupLayout,
}

impl FromWorld for OutlinePipeline {
    fn from_world(render_world: &mut World) -> Self {
        let mesh_pipeline = render_world.resource::<MeshPipeline>();
        let render_device = render_world.resource::<RenderDevice>();
        let mesh_binding = BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: true,
                min_binding_size: BufferSize::new(MeshUniform::std140_size_static() as u64),
            },
            count: None,
        };

        let mesh_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[mesh_binding],
            label: Some("mesh_layout"),
        });

        let view_layout = mesh_pipeline.view_layout.clone();

        let material_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("material layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(
                        OutlineMaterialUniform::std140_size_static() as u64
                    ),
                },
                count: None,
            }],
        });

        let window_size_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("window size layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(
                            DoubleReciprocalWindowSizeUniform::std140_size_static() as u64,
                        ),
                    },
                    count: None,
                }],
            });

        Self {
            mesh_layout,
            view_layout,
            material_layout,
            window_size_layout,
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
        let vertex_attributes = vec![
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            ATTRIBUTE_OUTLINE_NORMAL.at_shader_location(1),
        ];

        let vertex_buffer_layout = layout.get_layout(&vertex_attributes)?;

        let bind_group_layout = vec![
            self.view_layout.clone(),
            self.mesh_layout.clone(),
            self.material_layout.clone(),
            self.window_size_layout.clone(),
        ];

        Ok(RenderPipelineDescriptor {
            vertex: VertexState {
                shader: OUTLINE_SHADER_HANDLE.typed::<Shader>(),
                entry_point: "vertex".into(),
                shader_defs: vec![],
                buffers: vec![vertex_buffer_layout],
            },
            fragment: Some(FragmentState {
                shader: OUTLINE_SHADER_HANDLE.typed::<Shader>(),
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            layout: Some(bind_group_layout),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Front),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: key.primitive_topology(),
                strip_index_format: None,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Greater,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("outline_mesh_pipeline".into()),
        })
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
fn queue_outlines(
    opaque_3d_draw_functions: Res<DrawFunctions<Opaque3d>>,
    render_meshes: Res<RenderAssets<Mesh>>,
    outline_pipeline: Res<OutlinePipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<OutlinePipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    msaa: Res<Msaa>,
    material_meshes: Query<(Entity, &Handle<Mesh>, &MeshUniform), With<Handle<OutlineMaterial>>>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Opaque3d>)>,
) {
    let draw_function = opaque_3d_draw_functions
        .read()
        .get_id::<DrawOutlines>()
        .unwrap();

    let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples);

    for (view, mut opaque_phase) in views.iter_mut() {
        let view_matrix = view.transform.compute_matrix();
        let view_row_2 = view_matrix.row(2);

        for (entity, mesh_handle, mesh_uniform) in material_meshes.iter() {
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
                opaque_phase.add(Opaque3d {
                    entity,
                    pipeline,
                    draw_function,
                    distance: view_row_2.dot(mesh_uniform.transform.col(3)),
                })
            }
        }
    }
}

type DrawOutlines = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetOutlineMaterialBindGroup<2>,
    SetWindowSizeBindGroup<3>,
    DrawMesh,
);

pub struct SetOutlineMaterialBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetOutlineMaterialBindGroup<I> {
    type Param = (
        SRes<RenderAssets<OutlineMaterial>>,
        SQuery<Read<Handle<OutlineMaterial>>>,
    );
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (materials, query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let material_handle = query.get(item).unwrap();
        let material = materials.into_inner().get(material_handle).unwrap();
        pass.set_bind_group(I, &material.bind_group, &[]);
        RenderCommandResult::Success
    }
}
