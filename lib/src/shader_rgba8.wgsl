struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@group(0)
@binding(0)
var<uniform> vp_matrix: mat4x4<f32>;

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.tex_coord = tex_coord;
    result.position = vp_matrix * position;
    return result;
}

@group(0)
@binding(1)
var diffuse_color: texture_2d<f32>;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var output: vec4<f32>;

    let tex_color: vec4<f32> = textureLoad(diffuse_color, vec2<i32>(input.tex_coord * vec2<f32>(16.0, 16.0)), 0);

    let edgeThreshold: f32 = 0.01; // 테두리 두께

    if (input.tex_coord.x < edgeThreshold || input.tex_coord.x > 1.0 - edgeThreshold ||
        input.tex_coord.y < edgeThreshold || input.tex_coord.y > 1.0 - edgeThreshold) {
        output = vec4<f32>(0.0, 0.7, 0.0, 1.0);
    } else {
        output = tex_color;
    }
    return output;
}