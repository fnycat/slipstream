@group(0) @binding(0) var t_output: texture_storage_2d<rgba8unorm, write>;

struct VertexInput {
    @location(0) position: vec3<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(@builtin(position) coord: vec4<f32>) {
    let frag_coords = vec2<i32>(coord.xy);
    let frag_col = vec4<f32>(0.0, 1.0, 0.0, 1.0);

    textureStore(t_output, frag_coords, frag_col);
}
