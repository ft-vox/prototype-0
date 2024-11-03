use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use image::GenericImageView;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct SkyUniforms {
    vp_matrix: [f32; 16],
}

pub struct SkyRenderer {
    fov: f32,
    clip_near: f32,
    clip_far: f32,
    aspect_ratio: f32,
    eye_dir: Vec3,
    projection_matrix: Mat4,
    view_matrix: Mat4,
    sky_pipeline: wgpu::RenderPipeline,
    sky_bind_group: wgpu::BindGroup,
    sky_uniform_buffer: wgpu::Buffer,
}

impl SkyRenderer {
    pub fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        fov: f32,
        clip_near: f32,
        clip_far: f32,
    ) -> Self {
        let min_alignment = device.limits().min_uniform_buffer_offset_alignment as usize;

        let sky_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Skybox Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../assets/shader_sky.wgsl"
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
            depth_stencil: None,
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

        SkyRenderer {
            fov,
            clip_near,
            clip_far,
            aspect_ratio: config.width as f32 / config.height as f32,
            eye_dir: Vec3::ZERO,
            projection_matrix: generate_projection_matrix(
                config.width as f32 / config.height as f32,
                fov,
                clip_near,
                clip_far,
            ),
            view_matrix: Mat4::ZERO,
            sky_bind_group,
            sky_pipeline,
            sky_uniform_buffer,
        }
    }
    pub fn resize(&mut self, config: &wgpu::SurfaceConfiguration) {
        self.aspect_ratio = config.width as f32 / config.height as f32;
        self.projection_matrix =
            generate_projection_matrix(self.aspect_ratio, self.fov, self.clip_near, self.clip_far);
    }

    pub fn update(&mut self, eye_dir: Vec3) {
        self.eye_dir = eye_dir;
        self.view_matrix = glam::Mat4::look_to_rh(Vec3::ZERO, eye_dir, glam::Vec3::Z);
    }

    pub fn render(
        &mut self,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        {
            let sky_view_projection_matrix = self.projection_matrix * self.view_matrix;
            queue.write_buffer(
                &self.sky_uniform_buffer,
                0,
                bytemuck::cast_slice(&[SkyUniforms {
                    vp_matrix: *sky_view_projection_matrix.as_ref(),
                }]),
            );
        }
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
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&self.sky_pipeline);
            rpass.set_bind_group(0, &self.sky_bind_group, &[]);
            rpass.draw(0..36, 0..1);
        }
    }
}

fn load_skybox_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
    let skybox_image = include_bytes!("../../assets/minecraft.png") as &[u8];

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

fn generate_projection_matrix(aspect_ratio: f32, fov: f32, near: f32, far: f32) -> glam::Mat4 {
    if aspect_ratio > 1.0 {
        let fov_x_radians = fov.to_radians();
        let fov_y_radians = 2.0 * (0.5 * fov_x_radians).tan().atan() / aspect_ratio;
        glam::Mat4::perspective_rh(fov_y_radians, aspect_ratio, near, far)
    } else {
        let fov_y_radians = fov.to_radians();
        glam::Mat4::perspective_rh(fov_y_radians, aspect_ratio, near, far)
    }
}
