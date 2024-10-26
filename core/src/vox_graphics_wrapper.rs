use bytemuck::{Pod, Zeroable};

use glam::{Mat4, Vec3};
use image::{GenericImageView, Pixel};

use std::{borrow::Cow, rc::Rc};

use crate::vertex::Vertex;
use crate::{FOG_COLOR, FOG_END, FOG_START, FOV};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct WorldUniforms {
    vp_matrix: [f32; 16],
    view_position: [f32; 4],
    fog_color: [f32; 4],
    fog_start: f32,
    fog_end: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct SkyUniforms {
    vp_matrix: [f32; 16],
}

pub struct VoxGraphicsWrapper {
    projection_matrix: Mat4,
    depth_buffer: wgpu::TextureView,
    world_bind_group: wgpu::BindGroup,
    world_uniform_buffer: wgpu::Buffer,
    world_pipeline: wgpu::RenderPipeline,
    sky_pipeline: wgpu::RenderPipeline,
    sky_bind_group: wgpu::BindGroup,
    sky_uniform_buffer: wgpu::Buffer,
}

impl VoxGraphicsWrapper {
    pub fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        //////////////////////////////
        /////////// Common ///////////
        //////////////////////////////

        let min_alignment = device.limits().min_uniform_buffer_offset_alignment as usize;

        let texture_extent = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let draw_depth_buffer = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Buffer"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_buffer = draw_depth_buffer.create_view(&wgpu::TextureViewDescriptor::default());

        /////////////////////////////
        /////////// World ///////////
        /////////////////////////////

        let world_uniform_size = std::mem::size_of::<WorldUniforms>();
        let world_aligned_uniform_size =
            ((world_uniform_size + min_alignment - 1) / min_alignment) * min_alignment;

        let world_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                world_aligned_uniform_size as u64,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: (false),
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&world_bind_group_layout],
            push_constant_ranges: &[],
        });

        let (texels, width, height) = load_texture_from_terrain_png();
        let terrain_texture_extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let terrain_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: terrain_texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let terrain_texture_view =
            terrain_texture.create_view(&wgpu::TextureViewDescriptor::default());
        queue.write_texture(
            terrain_texture.as_image_copy(),
            &texels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: None,
            },
            terrain_texture_extent,
        );

        let world_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: world_aligned_uniform_size as wgpu::BufferAddress,
            mapped_at_creation: false,
        });

        let world_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &world_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: world_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&terrain_texture_view),
                },
            ],
            label: None,
        });

        let world_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../assets/shader_world.wgsl"
            ))),
        });

        let world_vertex_size = std::mem::size_of::<Vertex>();

        let world_vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: world_vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 4 * 4,
                    shader_location: 1,
                },
            ],
        }];

        let world_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &world_shader,
                entry_point: "vs_world",
                buffers: &world_vertex_buffers,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &world_shader,
                entry_point: "fs_world",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.view_formats[0],
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::OVER,
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        /////////////////////////////
        ///////////  Sky  ///////////
        /////////////////////////////

        let sky_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Skybox Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../assets/shader_sky.wgsl"
            ))),
        });

        let sky_uniform_size = std::mem::size_of::<SkyUniforms>();
        let sky_aligned_uniform_size =
            ((sky_uniform_size + min_alignment - 1) / min_alignment) * min_alignment;

        let sky_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sky Uniform Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: sky_aligned_uniform_size as wgpu::BufferAddress,
            mapped_at_creation: false,
        });

        let sky_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                sky_aligned_uniform_size as u64,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::Cube,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let sky_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Skybox Pipeline Layout"),
            bind_group_layouts: &[&sky_bind_group_layout],
            push_constant_ranges: &[],
        });

        let sky_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Skybox Pipeline"),
            layout: Some(&sky_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &sky_shader,
                entry_point: "vs_sky",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &sky_shader,
                entry_point: "fs_sky",
                targets: &[Some(config.view_formats[0].into())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let skybox_texture = load_skybox_texture(device, queue);
        let skybox_texture_view = skybox_texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Skybox Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let sky_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &sky_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: sky_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&skybox_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

        VoxGraphicsWrapper {
            projection_matrix: Self::generate_projection_matrix(
                config.width as f32 / config.height as f32,
            ),
            depth_buffer,
            world_bind_group,
            world_uniform_buffer,
            world_pipeline,
            sky_bind_group,
            sky_pipeline,
            sky_uniform_buffer,
        }
    }

    fn generate_projection_matrix(aspect_ratio: f32) -> glam::Mat4 {
        let fov_x_radians = FOV.to_radians();

        glam::Mat4::perspective_rh(
            2.0 * (0.5 * fov_x_radians).tan() / aspect_ratio,
            aspect_ratio,
            0.25,
            368.0,
        )
    }

    pub fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        let texture_extent = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let draw_depth_buffer = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Buffer"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_buffer = draw_depth_buffer.create_view(&wgpu::TextureViewDescriptor::default());
        self.depth_buffer = depth_buffer;

        self.projection_matrix =
            Self::generate_projection_matrix(config.width as f32 / config.height as f32);
    }

    pub fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        eye: Vec3,
        horizontal_rotation: f32,
        vertical_rotation: f32,
        buffer_data: Vec<(i32, i32, i32, Rc<(wgpu::Buffer, wgpu::Buffer, u32)>)>,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let dir = (glam::Mat3::from_rotation_z(horizontal_rotation)
            * glam::Mat3::from_rotation_x(vertical_rotation))
            * glam::Vec3::Y;

        let world_view_matrix = glam::Mat4::look_to_rh(eye, dir, glam::Vec3::Z);
        let world_view_projection_matrix = self.projection_matrix * world_view_matrix;
        let frustum_planes = extract_frustum_planes(&world_view_projection_matrix);

        queue.write_buffer(
            &self.world_uniform_buffer,
            0,
            bytemuck::cast_slice(&[WorldUniforms {
                vp_matrix: *world_view_projection_matrix.as_ref(),
                view_position: [eye.x, eye.y, eye.z, 0.0],
                fog_color: FOG_COLOR,
                fog_start: FOG_START,
                fog_end: FOG_END,
            }]),
        );

        let sky_view_matrix = glam::Mat4::look_to_rh(Vec3::ZERO, dir, glam::Vec3::Z);
        let sky_view_projection_matrix = self.projection_matrix * sky_view_matrix;
        queue.write_buffer(
            &self.sky_uniform_buffer,
            0,
            bytemuck::cast_slice(&[SkyUniforms {
                vp_matrix: *sky_view_projection_matrix.as_ref(),
            }]),
        );

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_buffer,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&self.sky_pipeline);
            rpass.set_bind_group(0, &self.sky_bind_group, &[]);
            rpass.draw(0..36, 0..1);

            rpass.set_pipeline(&self.world_pipeline);
            rpass.set_bind_group(0, &self.world_bind_group, &[]);

            let mut skip_frustum = 0;
            let mut skip_zero_index = 0;
            let mut drawed = 0;
            for (x, y, z, buffers) in buffer_data {
                if !is_sphere_in_frustum_planes(
                    &frustum_planes,
                    Vec3::new(
                        x as f32 * 16.0 + 8.0,
                        y as f32 * 16.0 + 8.0,
                        z as f32 * 16.0 + 8.0,
                    ),
                    13.8564, // sqrt(8^2 + 8^2 + 8^2)
                ) {
                    skip_frustum += 1;
                    continue;
                }

                let (vertex_buffer, index_buffer, index_count) = &*buffers;
                if *index_count == 0 {
                    skip_zero_index += 1;
                    continue;
                }
                rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                rpass.draw_indexed(0..*index_count, 0, 0..1);
                drawed += 1;
            }
            println!(
                "skip_frustum: {}, skip_zero_index: {}, drawed: {}",
                skip_frustum, skip_zero_index, drawed
            );
        }
        queue.submit(Some(encoder.finish()));
    }
}

fn load_texture_from_terrain_png() -> (Vec<u8>, u32, u32) {
    let img = image::load_from_memory_with_format(
        include_bytes!("../assets/terrain.png"),
        image::ImageFormat::Png,
    )
    .expect("Failed to open image");

    let (width, height) = img.dimensions();

    let mut texels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let channels = pixel.channels();
            texels.extend_from_slice(channels);
        }
    }
    (texels, width, height)
}

fn load_skybox_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
    let skybox_image = include_bytes!("../assets/minecraft.png") as &[u8];

    let img = image::load_from_memory(skybox_image).expect("Failed to open image");
    let img = img.to_rgba8();
    let (full_width, full_height) = img.dimensions();

    let face_width = full_width / 4;
    let face_height = full_height / 3;

    let mut face_data = Vec::with_capacity(6);

    #[rustfmt::skip]
    let face_coords = [
        (2, 1),
        (0, 1),
        (1, 0),
        (1, 2),
        (1, 1),
        (3, 1),
    ];

    for &(x, y) in &face_coords {
        let sub_image = img.view(x * face_width, y * face_height, face_width, face_height);
        let sub_image = sub_image.to_image();
        face_data.push(sub_image.into_raw());
    }

    let size = wgpu::Extent3d {
        width: face_width,
        height: face_height,
        depth_or_array_layers: 6,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("Skybox Texture"),
        view_formats: &[],
    });

    for (i, face) in face_data.iter().enumerate() {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: i as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            face,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * face_width),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: face_width,
                height: face_height,
                depth_or_array_layers: 1,
            },
        );
    }

    texture
}

struct FrustumPlane {
    normal: Vec3,
    distance: f32,
}

impl FrustumPlane {
    fn normalize(&mut self) {
        let length = self.normal.length();
        self.normal /= length;
        self.distance /= length;
    }
}

fn extract_frustum_planes(view_projectioon: &Mat4) -> [FrustumPlane; 6] {
    let matrix_array = view_projectioon.to_cols_array_2d();
    let mut planes = [
        FrustumPlane {
            normal: Vec3::new(
                matrix_array[0][3] + matrix_array[0][0],
                matrix_array[1][3] + matrix_array[1][0],
                matrix_array[2][3] + matrix_array[2][0],
            ),
            distance: matrix_array[3][3] + matrix_array[3][0],
        }, // Left
        FrustumPlane {
            normal: Vec3::new(
                matrix_array[0][3] - matrix_array[0][0],
                matrix_array[1][3] - matrix_array[1][0],
                matrix_array[2][3] - matrix_array[2][0],
            ),
            distance: matrix_array[3][3] - matrix_array[3][0],
        }, // Right
        FrustumPlane {
            normal: Vec3::new(
                matrix_array[0][3] + matrix_array[0][1],
                matrix_array[1][3] + matrix_array[1][1],
                matrix_array[2][3] + matrix_array[2][1],
            ),
            distance: matrix_array[3][3] + matrix_array[3][1],
        }, // Near
        FrustumPlane {
            normal: Vec3::new(
                matrix_array[0][3] - matrix_array[0][1],
                matrix_array[1][3] - matrix_array[1][1],
                matrix_array[2][3] - matrix_array[2][1],
            ),
            distance: matrix_array[3][3] - matrix_array[3][1],
        }, // Far
        FrustumPlane {
            normal: Vec3::new(
                matrix_array[0][3] + matrix_array[0][2],
                matrix_array[1][3] + matrix_array[1][2],
                matrix_array[2][3] + matrix_array[2][2],
            ),
            distance: matrix_array[3][3] + matrix_array[3][2],
        }, // Bottom
        FrustumPlane {
            normal: Vec3::new(
                matrix_array[0][3] - matrix_array[0][2],
                matrix_array[1][3] - matrix_array[1][2],
                matrix_array[2][3] - matrix_array[2][2],
            ),
            distance: matrix_array[3][3] - matrix_array[3][2],
        }, // Top
    ];

    for plane in planes.iter_mut() {
        plane.normalize();
    }

    planes
}

fn is_sphere_in_frustum_planes(
    frustum_planes: &[FrustumPlane; 6],
    center: Vec3,
    radius: f32,
) -> bool {
    for plane in frustum_planes {
        let distance = plane.normal.dot(center) + plane.distance;
        if distance < -radius {
            return false;
        }
    }
    true
}
