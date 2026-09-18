struct CameraUniformData {
    viewport_size: vec4<f32>,
    proj_matrix: mat4x4<f32>
}

@group(0) @binding(0) var<uniform> camera: CameraUniformData;

struct VertexInput {
    @location(0) position: vec3<f32>
}

struct VertexOutput {
    @builtin(position) vertex: vec4<f32>,
    @location(0) original: vec3<f32>
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.vertex = camera.proj_matrix * vec4<f32>(model.position, 1.0);
    out.original = model.position;
    return out;
}

fn linear_to_srgb(color: vec3<f32>) -> vec3<f32> {
    return 1.055 * pow(input.original, vec3<f32>(1.0 / 2.4)) - 0.055;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Convert the linear colours to SRGB.
    // Without this, the colours will look very washed out in the editor.
    let srgb = linear_to_srgb(input.original);
    return vec4<f32>(srgb, 1.0);
}
