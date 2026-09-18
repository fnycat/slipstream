struct VertexInput {
    @location(0) position: vec2<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    let frag_coords = vec2<i32>(coord.xy);
    let frag_col = vec4<f32>(vec2<f32>(frag_coords) / vec2<f32>(652.0, 539.0), 0.0, 1.0);

    return frag_col;
}
