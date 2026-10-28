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

const STANDARD_TYPE: u32 = 0;
const KEY_TYPE: u32 = 1;
const LAP_TYPE: u32 = 2;

struct VertexOutput {
    @builtin(position) clip: vec4f,
    @location(0) @interpolate(flat) ty: u32
}

// Generates a full box from the two corners.
@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
    @location(0) left: vec2f,
    @location(1) right: vec2f,
    @location(2) respawn_y: f32,
    @location(3) ty: u32
) -> VertexOutput {
    let base = select(left, right, (vi >> 1u) == 1u);
    let side = f32(vi & 1u);

    let dir = right - left;
    let len = max(length(dir), 1e-6);
    let normal = vec2(-dir.y, dir.x) / len;

    let world = base + normal * f32(vi & 1u) * 2.0 - 1.0;
    let bound_y = respawn_y - 500.0 + 4000.0 * side;

    var out: VertexOutput;
    out.ty = ty;
    out.clip = camera.view_proj * vec4f(world.x, bound_y, world.y, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let red = select(0.0, 1.0, in.ty == STANDARD_TYPE);
    let green = select(0.0, 1.0, in.ty == KEY_TYPE);
    let blue = select(0.0, 1.0, in.ty == LAP_TYPE);

    return vec4f(red, green, blue, 0.25);
}
