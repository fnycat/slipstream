override line_width: f32 = 5.0;

struct CameraUniformData {
    /// Size of the viewport.
    viewport: vec2f,
    _padding: vec2f,
    pos: vec4f,
    view_proj: mat4x4f,
    inv_view_proj: mat4x4f
}

@group(0) @binding(0)
var<uniform> camera: CameraUniformData;

/// Contains the transformations of the bones.
@group(1) @binding(0)
var<storage, read> bone_matrices: array<mat4x4f>;
/// Contains the references to the parent bones. A value of `a` at index `i` implies that
/// bone `a` is parent of bone `i`.
@group(1) @binding(1)
var<storage, read> bone_parents: array<u32>;

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32
) -> @builtin(position) vec4f {
    let parent_index = bone_parents[instance_index];
    if parent_index == 65536 {
        return vec4(50.0, 0.0, 0.0, 1.0);
    }

    let child_world = camera.view_proj * vec4(bone_matrices[instance_index][3].xyz, 1.0);
    let parent_world = camera.view_proj * vec4(bone_matrices[parent_index][3].xyz, 1.0);

    let child = child_world.xy / camera.viewport * 0.5;
    let parent = parent_world.xy / camera.viewport * 0.5;

    let line = normalize(parent - child);
    let normal = vec2(-line.y, line.x);

    let is_child = (vertex_index >> 1u) == 1u;
    let side = select(-1.0, 1.0, (vertex_index & 1u) == 1u);
    let base = select(parent_world, child_world, is_child);

    let offset_ndc = normal * side * line_width / camera.viewport;
    let clip = vec4(base.xy + offset_ndc * base.w, base.z, base.w);

    return clip;
}

@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(1.0, 0.2, 0.2, 1.0);
}
