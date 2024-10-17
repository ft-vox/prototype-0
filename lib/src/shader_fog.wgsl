struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
    @location(1) distance: f32,
};

@group(0)
@binding(0)
var<uniform> vp_matrix: mat4x4<f32>;

@group(0)
@binding(2)
var<uniform> fog_data: vec4<f32>;

@group(0)
@binding(3)
var<uniform> view_position: vec4<f32>;

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) tex_coord: vec2<f32>
) -> VertexOutput {
    var result: VertexOutput;
    result.tex_coord = tex_coord;
    result.position = vp_matrix * position;
    result.distance = length(position.xyz - view_position.xyz);

    return result;
}

@group(0)
@binding(1)
var diffuse_color: texture_2d<f32>;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var output: vec4<f32>;

    output = textureLoad(diffuse_color, vec2<i32>(input.tex_coord * vec2<f32>(16.0, 16.0)), 0);

    let fog_start: f32 = fog_data.x;
    let fog_end: f32 = fog_data.y;
    let fog_color: f32 = fog_data.z;

    let fog_factor: f32 = clamp((fog_end - input.distance) / (fog_end - fog_start), 0.0, 1.0);

    output = mix(vec4<f32>(fog_color, fog_color, fog_color, 1.0), output, fog_factor);
    return output;
}