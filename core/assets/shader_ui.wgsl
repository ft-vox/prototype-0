struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) tex_layer: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) @interpolate(flat) tex_layer: u32,
};

struct Uniforms {
    transform: mat4x4<f32>,
    opacity: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var t_diffuse: texture_2d_array<f32>;
@group(0) @binding(2) var s_diffuse: sampler;

@vertex
fn vs_ui(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = in.tex_coord;
    out.tex_layer = in.tex_layer;
    out.clip_position = uniforms.transform * vec4<f32>(in.position, 0.0, 1.0);
    return out;
}

@fragment
fn fs_ui(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_diffuse, s_diffuse, in.tex_coord, in.tex_layer);
    return vec4<f32>(color.rgb, color.a * uniforms.opacity);
}
