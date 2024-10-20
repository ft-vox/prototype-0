use bytemuck::{Pod, Zeroable};
use ft_vox_prototype_0_map_core::Map;
use ft_vox_prototype_0_map_types::{Chunk, CHUNK_SIZE};
use ft_vox_prototype_0_util_lru_cache::LRUCache;
use glam::{Mat3, Vec3};
use image::{GenericImageView, Pixel};
use std::{borrow::Cow, collections::HashMap, rc::Rc};
use wgpu::util::DeviceExt;

mod vertex;

use vertex::{create_vertices_for_chunk, Vertex};

pub trait TerrainWorker {
    fn new(map: Map, render_distance: f32) -> Self;
    fn get_available(
        &mut self,
        chunk_coords: &[(i32, i32, i32)],
    ) -> Vec<((i32, i32, i32), Rc<Chunk>)>;
}

pub fn get_coords(distance: f32) -> Vec<(i32, i32, i32)> {
    let mut coords = Vec::new();
    let max_coord = distance.floor() as i32;
    let distance_squared = distance * distance;

    for x in -max_coord..=max_coord {
        for y in -max_coord..=max_coord {
            for z in -max_coord..=max_coord {
                let dist_sq = (x * x + y * y + z * z) as f32;
                if dist_sq <= distance_squared {
                    coords.push((x, y, z));
                }
            }
        }
    }

    coords
}

pub struct Vox<T: TerrainWorker> {
    eye: glam::Vec3,
    horizontal_rotation: f32,
    vertical_rotation: f32,
    is_paused: bool,
    projection_matrix: glam::Mat4,
    depth_buffer: wgpu::TextureView,
    chunks: HashMap<[i32; 3], Rc<Chunk>>,
    buffers: LRUCache<[i32; 3], Rc<(wgpu::Buffer, wgpu::Buffer, u32)>>,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    terrain_worker: T,
}

/// [ Speed in Minecraft ]
/// Walking speed: 4.317 blocks/second
/// Sprinting speed (Survival): 5.612 blocks/second
/// Flying speed (Creative): 10.89 blocks/second

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MoveSpeed {
    WALK,
    FLY,
}

impl MoveSpeed {
    pub const fn speed_per_sec(&self) -> f32 {
        match self {
            Self::WALK => 4.317,
            Self::FLY => 10.89,
        }
    }
}

pub const RENDER_DISTANCE: f32 = 7.0;

impl<T: TerrainWorker> Vox<T> {
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
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_buffer = draw_depth_buffer.create_view(&wgpu::TextureViewDescriptor::default());

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
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: aligned_uniform_size as wgpu::BufferAddress,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
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

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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

        // Done
        Vox {
            eye: glam::Vec3::new(0.0, -5.0, 3.0),
            horizontal_rotation: 0.0,
            vertical_rotation: 0.0,
            projection_matrix: Self::generate_projection_matrix(
                config.width as f32 / config.height as f32,
            ),
            depth_buffer,
            chunks: HashMap::new(),
            buffers: LRUCache::new(get_coords(RENDER_DISTANCE).len()),
            bind_group,
            uniform_buffer,
            pipeline,
            is_paused: false,
            terrain_worker: T::new(Map::new(42), RENDER_DISTANCE),
        }
    }

    fn generate_projection_matrix(aspect_ratio: f32) -> glam::Mat4 {
        glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_2, aspect_ratio, 0.25, 1000.0)
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    pub fn set_is_paused(&mut self, paused: bool) {
        self.is_paused = paused;
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

    pub fn tick(
        &mut self,
        delta_time: f32,
        move_direction: [f32; 3],
        move_speed: MoveSpeed,
        delta_horizontal_rotation: f32,
        delta_vertical_rotation: f32,
    ) {
        if self.is_paused {
            return;
        }

        // rotate
        {
            self.horizontal_rotation += delta_horizontal_rotation;
            self.horizontal_rotation %= 2.0 * std::f32::consts::PI;
            if self.horizontal_rotation < 0.0 {
                self.horizontal_rotation += 2.0 * std::f32::consts::PI;
            }

            self.vertical_rotation += delta_vertical_rotation;
            self.vertical_rotation = self.vertical_rotation.clamp(
                -0.4999 * std::f32::consts::PI,
                0.4999 * std::f32::consts::PI,
            );
        }

        // move
        {
            let move_direction = {
                let move_direction = Mat3::from_rotation_z(self.horizontal_rotation)
                    * Vec3::new(move_direction[0], move_direction[1], move_direction[2]);
                let move_speed = move_direction.length();
                if move_speed > 1.0 {
                    move_direction / move_speed
                } else {
                    move_direction
                }
            };
            self.eye += move_direction * move_speed.speed_per_sec() * delta_time;
        }
    }

    pub fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        const FOG_COLOR: f64 = 0.8;
        const FOG_END: f32 = (RENDER_DISTANCE - 2.9) * CHUNK_SIZE as f32;
        const FOG_START: f32 = FOG_END * 0.8;

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let dir = (glam::Mat3::from_rotation_z(self.horizontal_rotation)
                * glam::Mat3::from_rotation_x(self.vertical_rotation))
                * glam::Vec3::Y;
            let view_matrix = glam::Mat4::look_to_rh(self.eye, dir, glam::Vec3::Z);
            let mx_total = self.projection_matrix * view_matrix;
            let mx_ref: &[f32; 16] = mx_total.as_ref();
            let fog_color: [f32; 4] = [FOG_COLOR as f32, FOG_COLOR as f32, FOG_COLOR as f32, 1.0];
            let fog_start: f32 = FOG_START;
            let fog_end: f32 = FOG_END;
            let view_position: [f32; 4] = [self.eye.x, self.eye.y, self.eye.z, 0.0];
            let uniforms = Uniforms {
                transform: *mx_ref,
                view_position,
                fog_color,
                fog_start,
                fog_end,
            };
            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
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
            let (eye_x, eye_y, eye_z) = {
                let eye = (self.eye / CHUNK_SIZE as f32).floor();
                (eye.x as i32, eye.y as i32, eye.z as i32)
            };
            self.chunks.clear();
            for ((x, y, z), chunk) in self.terrain_worker.get_available(
                &get_coords(RENDER_DISTANCE + 2.0)
                    .into_iter()
                    .map(|(x, y, z)| (x + eye_x, y + eye_y, z + eye_z))
                    .collect::<Vec<_>>(),
            ) {
                self.chunks.insert([x, y, z], chunk);
            }
            for (x, y, z) in get_coords(RENDER_DISTANCE)
                .into_iter()
                .map(|(x, y, z)| (x + eye_x, y + eye_y, z + eye_z))
                .collect::<Vec<_>>()
            {
                if let Some(rc) = self.get_buffers(device, x, y, z) {
                    let (vertex_buffer, index_buffer, index_count) = &*rc;
                    if *index_count == 0 {
                        continue;
                    }
                    rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    rpass.draw_indexed(0..*index_count, 0, 0..1);
                }
            }
        }

        queue.submit(Some(encoder.finish()));
    }

    fn get_buffers(
        &mut self,
        device: &wgpu::Device,
        x: i32,
        y: i32,
        z: i32,
    ) -> Option<Rc<(wgpu::Buffer, wgpu::Buffer, u32)>> {
        if let Some(result) = self.buffers.get(&[x, y, z]) {
            Some(result)
        } else if let Some(new_value) = self.create_buffers(device, x, y, z) {
            let result = Rc::new(new_value);
            self.buffers.put([x, y, z], result.clone());
            Some(result)
        } else {
            None
        }
    }

    fn create_buffers(
        &mut self,
        device: &wgpu::Device,
        x: i32,
        y: i32,
        z: i32,
    ) -> Option<(wgpu::Buffer, wgpu::Buffer, u32)> {
        let chunk = self.chunks.get(&[x, y, z]);
        let chunk_px = self.chunks.get(&[x + 1, y, z]);
        let chunk_nx = self.chunks.get(&[x - 1, y, z]);
        let chunk_py = self.chunks.get(&[x, y + 1, z]);
        let chunk_ny = self.chunks.get(&[x, y - 1, z]);
        let chunk_pz = self.chunks.get(&[x, y, z + 1]);
        let chunk_nz = self.chunks.get(&[x, y, z - 1]);

        if chunk.is_none()
            || chunk_px.is_none()
            || chunk_nx.is_none()
            || chunk_py.is_none()
            || chunk_ny.is_none()
            || chunk_pz.is_none()
            || chunk_nz.is_none()
        {
            None
        } else {
            let chunk = chunk.unwrap();
            let chunk_px = chunk_px.unwrap();
            let chunk_nx = chunk_nx.unwrap();
            let chunk_py = chunk_py.unwrap();
            let chunk_ny = chunk_ny.unwrap();
            let chunk_pz = chunk_pz.unwrap();
            let chunk_nz = chunk_nz.unwrap();

            let (vertex_data, index_data) = create_vertices_for_chunk(
                &chunk, x, y, z, &chunk_px, &chunk_nx, &chunk_py, &chunk_ny, &chunk_pz, &chunk_nz,
            );

            let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertex_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&index_data),
                usage: wgpu::BufferUsages::INDEX,
            });

            Some((vertex_buf, index_buf, index_data.len() as u32))
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    transform: [f32; 16],
    view_position: [f32; 4],
    fog_color: [f32; 4],
    fog_start: f32,
    fog_end: f32,
}

// TODO: embed texels instead of png, remove image dependency

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
