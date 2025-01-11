use std::{borrow::Cow, sync::Arc};

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use image::GenericImageView;

use ft_vox_prototype_0_map_types::CHUNK_SIZE;

use crate::graphics::Frustum;
use crate::vertex::Vertex;
use crate::FOG_COLOR_SRGB;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct WorldUniforms {
    vp_matrix: [f32; 16],
    view_position: [f32; 4],
    fog_color: [f32; 4],
    fog_start: f32,
    fog_end: f32,
}

pub struct WorldRenderer {
    fov: f32,
    clip_near: f32,
    clip_far: f32,
    aspect_ratio: f32,
    eye: Vec3,
    eye_dir: Vec3,
    projection_matrix: Mat4,
    view_matrix: Mat4,
    frustum: Frustum,
    depth_buffer: wgpu::TextureView, // TODO: separate for other renderer(maybe)
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl WorldRenderer {
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
        let world_uniform_size = std::mem::size_of::<WorldUniforms>();
        let world_aligned_uniform_size =
            ((world_uniform_size + min_alignment - 1) / min_alignment) * min_alignment;
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
                "../../assets/shader_world.wgsl"
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
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        WorldRenderer {
            fov,
            clip_near,
            clip_far,
            aspect_ratio: config.width as f32 / config.height as f32,
            eye: Vec3::ZERO,
            eye_dir: Vec3::ZERO,
            projection_matrix: generate_projection_matrix(
                config.width as f32 / config.height as f32,
                fov,
                clip_near,
                clip_far,
            ),
            view_matrix: Mat4::ZERO,
            frustum: Frustum::new(),
            depth_buffer,
            bind_group: world_bind_group,
            uniform_buffer: world_uniform_buffer,
            pipeline: world_pipeline,
        }
    }

    pub fn resize(&mut self, config: &wgpu::SurfaceConfiguration, device: &wgpu::Device) {
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
        self.depth_buffer = depth_buffer;

        self.aspect_ratio = config.width as f32 / config.height as f32;
        self.projection_matrix =
            generate_projection_matrix(self.aspect_ratio, self.fov, self.clip_near, self.clip_far);
    }

    pub fn update(&mut self, eye: Vec3, eye_dir: Vec3) {
        self.eye = eye;
        self.eye_dir = eye_dir;
        self.view_matrix = glam::Mat4::look_to_rh(eye, eye_dir, glam::Vec3::Z);
        self.frustum
            .update(&(self.projection_matrix * self.view_matrix));
    }

    pub fn render(
        &mut self,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        buffer_data: Vec<((i32, i32), Arc<(wgpu::Buffer, wgpu::Buffer, u32)>)>,
        fog_distance: f32,
    ) {
        {
            let view_projection_matrix = self.projection_matrix * self.view_matrix;

            let fog_end = (fog_distance - 1.0) * CHUNK_SIZE as f32;
            let fog_start = fog_end * 0.8;

            queue.write_buffer(
                &self.uniform_buffer,
                0,
                bytemuck::cast_slice(&[WorldUniforms {
                    vp_matrix: *view_projection_matrix.as_ref(),
                    view_position: [self.eye.x, self.eye.y, self.eye.z, 0.0],
                    fog_color: remove_srgb_correction(FOG_COLOR_SRGB),
                    fog_start,
                    fog_end,
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
                        load: wgpu::LoadOp::Load,
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
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);

            for ((_x, _y), buffers) in buffer_data {
                let (vertex_buffer, index_buffer, index_count) = &*buffers;
                if *index_count == 0 {
                    continue;
                }
                // // TODO: fix frustum culling
                // if !self.frustum.is_sphere_in_frustum_planes(
                //     Vec3::new(
                //         x as f32 * 16.0 + 8.0,
                //         y as f32 * 16.0 + 8.0,
                //         z as f32 * 16.0 + 8.0,
                //     ),
                //     13.8564, // sqrt(8^2 + 8^2 + 8^2)
                // ) {
                //     continue;
                // }

                rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                rpass.draw_indexed(0..*index_count, 0, 0..1);
            }
        }
    }
}

fn load_texture_from_terrain_png() -> (Vec<u8>, u32, u32) {
    let img = image::load_from_memory_with_format(
        include_bytes!("../../assets/terrain.png"),
        image::ImageFormat::Png,
    )
    .expect("Failed to open image");

    let (width, height) = img.dimensions();
    let rgba = img.to_rgba8();
    (rgba.into_raw(), width, height)
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

fn remove_srgb_correction(color: [f32; 4]) -> [f32; 4] {
    let remove_srgb = |v: f32| {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };

    [
        remove_srgb(color[0]),
        remove_srgb(color[1]),
        remove_srgb(color[2]),
        color[3],
    ]
}
