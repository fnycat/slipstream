struct CameraUniformData {
    /// Size of the viewport.
    viewport: vec4f,
    pos: vec4f,
    view_proj: mat4x4f,
    inv_view_proj: mat4x4f
}

@group(0) @binding(0)
var<uniform> camera: CameraUniformData;

@group(1) @binding(0)
var<storage, read> bone_matrices: array<mat4x4f>;

struct VertexOutput {
    @builtin(position) pos: vec4f,
    @location(0) uv: vec2f
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32
) -> VertexOutput {
    let point_size = 5.0;
    var corners = array<vec2f, 4>(
        vec2(-1.0, -1.0),
        vec2( 1.0, -1.0),
        vec2(-1.0,  1.0),
        vec2( 1.0,  1.0)
    );
    let corner = corners[vertex_index];

    let world_pos = bone_matrices[instance_index][3].xyz;
    var clip = camera.view_proj * vec4f(world_pos, 1.0);

    let offset_ndc = corner * point_size / camera.viewport.xy;
    clip = vec4(clip.xy + offset_ndc * clip.w, clip.z, clip.w);

    var out: VertexOutput;
    out.pos = clip;
    out.uv = corner;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    // Draw a round dot
    if (dot(in.uv, in.uv) > 1.0) { discard; }
    return vec4f(1.0, 0.2, 0.2, 1.0);
}
