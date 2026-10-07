struct CameraUniformData {
    /// Size of the viewport.
    viewport_size: vec2f,
    view_proj: mat4x4f,
    inv_view_proj: mat4x4f
}

struct Material {
    uv_count: u32
}

@group(0) @binding(0)
var<uniform> camera: CameraUniformData;

/// Contains the static transformations, i.e. the transformations applied
/// to the character when no animations are playing. These are required to
/// make the character render properly, as the vertex positions are in bone space.
@group(1) @binding(0)
var<storage, read> bone_transformations: array<mat4x4f>;

/// Stores the UV coordinates of the model. These are stored in a separate buffer because there can
/// be up to 8 of them.
@group(1) @binding(1)
var<storage, read> uvs: array<vec2f>;

@group(1) @binding(2)
var<uniform> material: Material;

// Texture bindings
@group(2) @binding(0) var tex0: texture_2d<f32>;
@group(2) @binding(1) var tex1: texture_2d<f32>;
@group(2) @binding(2) var tex2: texture_2d<f32>;
@group(2) @binding(3) var tex3: texture_2d<f32>;
@group(2) @binding(4) var tex4: texture_2d<f32>;
@group(2) @binding(5) var tex5: texture_2d<f32>;
@group(2) @binding(6) var tex6: texture_2d<f32>;
@group(2) @binding(7) var tex7: texture_2d<f32>;

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) color0: vec4f,
    /// Indices of the bones that this vertex is connected to.
    @location(3) bone_indices: vec4u,
    /// Weights for the indices above.
    @location(4) bone_weights: vec4f
}

struct VertexOutput {
    @builtin(position) vertex: vec4f,
    @location(0) normal: vec3f,
    @location(1) color0: vec4f,
    @location(2) @interpolate(flat) bone_index: u32
}

fn get_uv(vertex_index: u32, channel: u32) -> vec2f {
    return uvs[vertex_index * material.uv_count + channel];
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var pos = vec4f(0.0);
    var nrm = vec3f(0.0);

    for (var i = 0u; i < 1u; i += 1) {
        let weight = input.bone_weights[i];
        if (weight > 0.0) {
            let transform = bone_transformations[input.bone_indices[i]];
            pos += weight * (transform * vec4f(input.position, 1.0));
            nrm += weight * (mat3x3f(transform[0].xyz, transform[1].xyz, transform[2].xyz) * input.normal);
        }
    }

    var output: VertexOutput;
    output.vertex = camera.view_proj * pos;
    output.normal = normalize(nrm);
    output.color0 = input.color0;
    output.bone_index = input.bone_indices[0];
    return output;
}

fn linear_to_srgb(color: vec4f) -> vec4f {
    let rgb = 1.055 * pow(color.rgb, vec3f(1.0 / 2.4)) - 0.055;
    return vec4f(rgb, color.a);
}

fn compute_diffuse(normal: vec3f, color: vec4f) -> vec4f {
    let sunDirection = vec3f(0.0, 100.0, 100.0);
    let dot = dot(normal, sunDirection);

    return color * dot + vec4f(0.2);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4f {
    // let color = compute_diffuse(input.normal, input.color0);
    // let color = mix(input.color0, diffuse, 0.5);
    let color = input.color0;

    // let color = compute_diffuse(input.normal, input.color0);
    // let color = compute_diffuse(input.normal, vec4f(input.normal, 1.0));

    // Convert the linear colours to SRGB.
    // Without this, the colours will look very washed out in the editor.
    let srgb = linear_to_srgb(color);
    return srgb;
}
