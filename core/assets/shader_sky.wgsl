@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32
) -> @builtin(position) vec4<f32> {
    var skybox_vertices = array<vec3<f32>, 36>(
        vec3<f32>(-1.0, -1.0,  1.0), // Front face
        vec3<f32>( 1.0, -1.0,  1.0),
        vec3<f32>( 1.0,  1.0,  1.0),
        vec3<f32>(-1.0, -1.0,  1.0),
        vec3<f32>( 1.0,  1.0,  1.0),
        vec3<f32>(-1.0,  1.0,  1.0),

        vec3<f32>( 1.0, -1.0, -1.0), // Back face
        vec3<f32>(-1.0, -1.0, -1.0),
        vec3<f32>(-1.0,  1.0, -1.0),
        vec3<f32>( 1.0, -1.0, -1.0),
        vec3<f32>(-1.0,  1.0, -1.0),
        vec3<f32>( 1.0,  1.0, -1.0),

        vec3<f32>(-1.0, -1.0, -1.0), // Left face
        vec3<f32>(-1.0, -1.0,  1.0),
        vec3<f32>(-1.0,  1.0,  1.0),
        vec3<f32>(-1.0, -1.0, -1.0),
        vec3<f32>(-1.0,  1.0,  1.0),
        vec3<f32>(-1.0,  1.0, -1.0),

        vec3<f32>( 1.0, -1.0,  1.0), // Right face
        vec3<f32>( 1.0, -1.0, -1.0),
        vec3<f32>( 1.0,  1.0, -1.0),
        vec3<f32>( 1.0, -1.0,  1.0),
        vec3<f32>( 1.0,  1.0, -1.0),
        vec3<f32>( 1.0,  1.0,  1.0),

        vec3<f32>(-1.0,  1.0,  1.0), // Top face
        vec3<f32>( 1.0,  1.0,  1.0),
        vec3<f32>( 1.0,  1.0, -1.0),
        vec3<f32>(-1.0,  1.0,  1.0),
        vec3<f32>( 1.0,  1.0, -1.0),
        vec3<f32>(-1.0,  1.0, -1.0),

        vec3<f32>(-1.0, -1.0, -1.0), // Bottom face
        vec3<f32>( 1.0, -1.0, -1.0),
        vec3<f32>( 1.0, -1.0,  1.0),
        vec3<f32>(-1.0, -1.0, -1.0),
        vec3<f32>( 1.0, -1.0,  1.0),
        vec3<f32>(-1.0, -1.0,  1.0)
    );

    let vertex_position = skybox_vertices[vertex_index];
    return vec4<f32>(vertex_position, 1.0);
}

@group(0) @binding(0) var<uniform> transform: mat4x4<f32>;
@group(0) @binding(1) var skybox_texture: texture_cube<f32>;
@group(0) @binding(2) var skybox_sampler: sampler;

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    let coord = (frag_coord.xy / frag_coord.w) * 2.0 - 1.0;
    let view_direction = (transform * vec4<f32>(coord, -1.0, 0.0)).xyz;
    return textureSample(skybox_texture, skybox_sampler, normalize(view_direction));
}