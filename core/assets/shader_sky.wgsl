struct Uniforms {
    vp_matrix: mat4x4<f32>,
};

@group(0)
@binding(0)
var<uniform> uniforms: Uniforms;

@group(0)
@binding(1)
var skybox_texture: texture_cube<f32>;

@group(0)
@binding(2)
var skybox_sampler: sampler;

struct VertexOutput {
    @builtin(position)  position: vec4<f32>,
    @location(0) tex_coords: vec3<f32>,
};

@vertex
fn vs_sky(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {

    var positions = array<vec3<f32>, 8>(
        vec3<f32>(-1.0, -1.0,  1.0),
        vec3<f32>( 1.0, -1.0,  1.0),
        vec3<f32>(-1.0,  1.0,  1.0),
        vec3<f32>( 1.0,  1.0,  1.0),
        vec3<f32>(-1.0, -1.0, -1.0),
        vec3<f32>( 1.0, -1.0, -1.0),
        vec3<f32>(-1.0,  1.0, -1.0),
        vec3<f32>( 1.0,  1.0, -1.0)
    );

    var indices = array<u32, 36>(
        0, 2, 1, 1, 2, 3, // 위
        4, 5, 6, 5, 7, 6, // 아래
        0, 4, 2, 2, 4, 6, // 
        1, 3, 5, 5, 3, 7, // 
        2, 6, 3, 3, 6, 7, // 
        0, 1, 4, 1, 5, 4  // 
    );

    let index = indices[vertex_index];
    let vertex_pos = positions[index];

    var output: VertexOutput;
    output.tex_coords = vec3<f32>(vertex_pos.x, vertex_pos.z, vertex_pos.y);

    let projection_only = mat4x4<f32>(uniforms.vp_matrix);
    output.position = projection_only * vec4<f32>(vertex_pos, 1.0);
    return output;
}

@fragment
fn fs_sky(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(skybox_texture, skybox_sampler, input.tex_coords);
    return color;
}
