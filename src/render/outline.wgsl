#import bevy_pbr::mesh_types
#import bevy_pbr::mesh_view_bindings

struct Vertex {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
};

[[group(1), binding(0)]]
var<uniform> mesh: Mesh;

struct OutlineMat {
    width: f32;
    color: vec4<f32>;
};

[[group(2), binding(0)]]
var<uniform> outline_mat: OutlineMat;

struct DoubleReciprocalWindowSize {
    size: vec2<f32>;
};

[[group(3), binding(0)]]
var<uniform> window_size: DoubleReciprocalWindowSize;

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> VertexOutput {
    let mvp = view.view_proj * mesh.model;
    let clip_position = mvp * vec4<f32>(vertex.position, 1.0);
    let clip_normal = mvp * vec4<f32>(vertex.normal, 0.0);
    let extrude_offset = normalize(clip_normal.xy) * outline_mat.width * clip_position.w * window_size.size;
    var out: VertexOutput;
    out.clip_position = vec4<f32>(clip_position.xy + extrude_offset, clip_position.zw);
    return out;
}

[[stage(fragment)]]
fn fragment() -> [[location(0)]] vec4<f32> {
    return outline_mat.color;
}