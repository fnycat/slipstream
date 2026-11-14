override point_size: f32 = 250.0;

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

struct VertexOutput {
    @builtin(position) clip: vec4f,
    @location(0) local: vec2f
}

// Generates a small triangle around a point.
@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
    @location(0) position: vec3f
) -> VertexOutput {
    // Depending on the vertex index, we offset slightly to create a triangle with `position`
    // at its center.
    let dir = 3.0 * point_size / sqrt(2.0);
    let offsets = array<vec3f, 3>(
        vec3f(3.0 * point_size, 0.0, 0.0),
        vec3f(-dir, 0.0, dir),
        vec3f(-dir, 0.0, -dir)
    );

    let offset = offsets[vi];
    let corner = position + offset + vec3f(0.0, 200.0, 0.0); // raise the points slightly off the ground to avoid clipping

    var out: VertexOutput;
    out.clip = camera.view_proj * vec4f(corner, 1.0);
    out.local = offset.xz;
    return out;
}

/// Turns the triangle into a circle.
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    if (length(in.local) > point_size) {
        discard;
    }

    return vec4f(0.0, 1.0, 1.0, 1.0);
}
