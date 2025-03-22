struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) distance: f32,
    @location(2) filter_tex_coord: vec2<f32>,
    @location(3) filter_color: vec4<f32>,
};

struct Uniforms {
    vp_matrix: mat4x4<f32>,
    view_position: vec4<f32>,
    fog_color: vec4<f32>,
    fog_start: f32,
    fog_end: f32,
};

@group(0)
@binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_common(
    @location(0) position: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) filter_tex_coord: vec2<f32>,
    @location(3) filter_color: vec4<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.tex_coord = tex_coord;
    result.position = uniforms.vp_matrix * position;
    result.distance = length(position.xyz - uniforms.view_position.xyz);
    result.filter_tex_coord = filter_tex_coord;
    result.filter_color = filter_color;

    return result;
}

fn lerp(t: f32, a: f32, b: f32) -> f32 {
    return a + t * (b - a);
}

fn applyTerrainFilter(
    original_color: vec4<f32>,
    filter_color: vec4<f32>,
) -> vec4<f32> {
    let r = lerp(filter_color.a, original_color.r, filter_color.r);
    let g = lerp(filter_color.a, original_color.g, filter_color.g);
    let b = lerp(filter_color.a, original_color.b, filter_color.b);
    let a = original_color.a;
    return vec4<f32>(r, g, b, a);
}

@group(0)
@binding(1)
var diffuse_color: texture_2d<f32>;

@fragment
fn fs_opaque(input: VertexOutput) -> @location(0) vec4<f32> {
    var output: vec4<f32> = textureLoad(diffuse_color, vec2<i32>(input.tex_coord * vec2<f32>(16.0, 16.0)), 0);
    var filter_color: vec4<f32> = textureLoad(diffuse_color, vec2<i32>(input.filter_tex_coord * vec2<f32>(16.0, 16.0)), 0) * input.filter_color;
    output = applyTerrainFilter(output, filter_color);
    var fog_factor: f32 = clamp((input.distance - uniforms.fog_start) / (uniforms.fog_end - uniforms.fog_start), 0.0, 1.0);
    output = mix(output, uniforms.fog_color, fog_factor);
    return output;
}

@fragment
fn fs_translucent(input: VertexOutput) -> @location(0) vec4<f32> {
    var output: vec4<f32> = textureLoad(diffuse_color, vec2<i32>(input.tex_coord * vec2<f32>(16.0, 16.0)), 0);
    var filter_color: vec4<f32> = textureLoad(diffuse_color, vec2<i32>(input.filter_tex_coord * vec2<f32>(16.0, 16.0)), 0) * input.filter_color;
    output = applyTerrainFilter(output, filter_color);
    var fog_factor: f32 = clamp((input.distance - uniforms.fog_start) / (uniforms.fog_end - uniforms.fog_start), 0.0, 1.0);
    output = mix(output, uniforms.fog_color, fog_factor);
    if (output.a == 0.0) {
        discard;
    }
    return output;
}
