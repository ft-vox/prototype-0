use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use image::{GenericImageView, Pixel};
use std::{borrow::Cow, rc::Rc};

use crate::vertex::Vertex;
use crate::RENDER_DISTANCE;
use ft_vox_prototype_0_map_types::CHUNK_SIZE;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    transform: [f32; 16],
    view_position: [f32; 4],
    fog_color: [f32; 4],
    fog_start: f32,
    fog_end: f32,
}

pub struct VoxGraphicsWrapper {
    world_projection_matrix: Mat4,
    world_depth_buffer: wgpu::TextureView,
    world_bind_group: wgpu::BindGroup,
    world_uniform_buffer: wgpu::Buffer,
    world_pipeline: wgpu::RenderPipeline,
}

impl VoxGraphicsWrapper {
    pub fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let min_alignment = device.limits().min_uniform_buffer_offset_alignment as usize;
        let uniform_size = std::mem::size_of::<Uniforms>();
        let aligned_uniform_size =
            ((uniform_size + min_alignment - 1) / min_alignment) * min_alignment;

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
        let world_depth_buffer =
            draw_depth_buffer.create_view(&wgpu::TextureViewDescriptor::default());

        // Create pipeline layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(aligned_uniform_size as u64),
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
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create the texture
        let (texels, width, height) = load_texture_from_terrain_png();
        let texture_extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        queue.write_texture(
            texture.as_image_copy(),
            &texels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: None,
            },
            texture_extent,
        );

        // Create other resources
        let world_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: aligned_uniform_size as wgpu::BufferAddress,
            mapped_at_creation: false,
        });

        let world_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: world_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
            ],
            label: None,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../assets/shader_fog.wgsl"
            ))),
        });

        let vertex_size = std::mem::size_of::<Vertex>();

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
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
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffers,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(config.view_formats[0].into())],
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

        VoxGraphicsWrapper {
            world_projection_matrix: Self::generate_projection_matrix(
                config.width as f32 / config.height as f32,
            ),
            world_depth_buffer,
            world_bind_group,
            world_uniform_buffer,
            world_pipeline,
        }
    }

    fn generate_projection_matrix(aspect_ratio: f32) -> glam::Mat4 {
        let fov_x_radians = 80.0_f32.to_radians();

        glam::Mat4::perspective_rh(
            2.0 * (0.5 * fov_x_radians).tan() / aspect_ratio,
            aspect_ratio,
            0.25,
            1000.0,
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
        self.world_depth_buffer = depth_buffer;

        self.world_projection_matrix =
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
        const FOG_COLOR: f64 = 0.8;
        const FOG_END: f32 = (RENDER_DISTANCE - 2.0) * CHUNK_SIZE as f32;
        const FOG_START: f32 = FOG_END * 0.8;

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let dir = (glam::Mat3::from_rotation_z(horizontal_rotation)
                * glam::Mat3::from_rotation_x(vertical_rotation))
                * glam::Vec3::Y;
            let view_matrix = glam::Mat4::look_to_rh(eye, dir, glam::Vec3::Z);
            let mx_total = self.world_projection_matrix * view_matrix;
            let mx_ref: &[f32; 16] = mx_total.as_ref();
            let fog_color: [f32; 4] = [FOG_COLOR as f32, FOG_COLOR as f32, FOG_COLOR as f32, 1.0];
            let fog_start: f32 = FOG_START;
            let fog_end: f32 = FOG_END;
            let view_position: [f32; 4] = [eye.x, eye.y, eye.z, 0.0];
            let uniforms = Uniforms {
                transform: *mx_ref,
                view_position,
                fog_color,
                fog_start,
                fog_end,
            };
            queue.write_buffer(
                &self.world_uniform_buffer,
                0,
                bytemuck::cast_slice(&[uniforms]),
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
                            r: FOG_COLOR,
                            g: FOG_COLOR,
                            b: FOG_COLOR,
                            a: FOG_COLOR,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.world_depth_buffer,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&self.world_pipeline);
            rpass.set_bind_group(0, &self.world_bind_group, &[]);

            for (_, _, _, buffers) in buffer_data {
                let (vertex_buffer, index_buffer, index_count) = &*buffers;
                if *index_count == 0 {
                    continue;
                }
                rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                rpass.draw_indexed(0..*index_count, 0, 0..1);
            }
        }
        queue.submit(Some(encoder.finish()));
    }
}

const TERRAIN_PNG: &[u8] = include_bytes!("../assets/terrain.png");

pub fn load_texture_from_terrain_png() -> (Vec<u8>, u32, u32) {
    let img = image::load_from_memory_with_format(TERRAIN_PNG, image::ImageFormat::Png)
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
