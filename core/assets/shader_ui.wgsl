struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

struct Uniforms {
    transform: mat4x4<f32>,
    opacity: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var ui_texture: texture_2d<f32>;

@group(0) @binding(2)
var ui_sampler: sampler;

@vertex
fn vs_ui(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var tex_coords = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0)
    );

    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>( 1.0, -1.0)
    );

    var output: VertexOutput;
    output.position = uniforms.transform * vec4<f32>(positions[vertex_index], 0.0, 1.0);
    output.tex_coord = tex_coords[vertex_index];
    return output;
}

@fragment
fn fs_ui(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(ui_texture, ui_sampler, input.tex_coord);
    color.a *= uniforms.opacity;
    return color;
}
