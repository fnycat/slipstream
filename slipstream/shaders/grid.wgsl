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

const GRID_SPACING: f32 = 1.0;
const GRID_COLOR: vec4f = vec4f(0.5);

/// Generates 2D coordinates [-1.0, 1.0] from index 0, 1, 2
fn unproject_point(x: f32, y: f32, z: f32, inv_vp: mat4x4f) -> vec3f {
    let unprojected = inv_vp * vec4f(x, y, z, 1.0);
    return unprojected.xyz / unprojected.w;
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    // Convert vertex index to NDC points such that the triangle covers the entire screen.
    // Index 0 converts to vec2(-1.0, -1.0).
    // Index 1 converts to vec2(3.0, -1.0)
    // Index 2 converts to vec2(-1.0, 3.0)
    let x = f32(vertex_index & 1) * 4.0 - 1.0;
    let y = f32(vertex_index >> 1) * 4.0 - 1.0;

    return vec4f(x, y, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) fragCoord: vec4f) -> @location(0) vec4f {
    let uv = fragCoord.xy / camera.viewport;

    let grid_size = 5.0;

    let x = step(0.5, fract(uv.x * 10.0 * camera.viewport.x / camera.viewport.y));
    let y = step(0.5, fract(uv.y * 10.0));

    let a = vec3f(0.3);
    let b = vec3f(0.2);

    return vec4f(mix(a, b, f32(x != y)), 1.0);
}
