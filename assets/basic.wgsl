struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct Uniforms {
    transform: mat4x4<f32>,
    material: Material,
}

struct Material {
    tint_color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = uniforms.transform * vec4<f32>(input.position.xyz, 1.0);
    output.tex_coords = input.tex_coords;
    return output;
}

@group(0) @binding(1)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(2)
var s_diffuse: sampler;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let texel = textureSample(t_diffuse, s_diffuse, vertex.tex_coords);
    return texel * uniforms.material.tint_color;
}
