use crate::map::*;
use crate::texture::*;
use crate::vertex::*;
use bytemuck::{Pod, Zeroable};
use std::{borrow::Cow, collections::BTreeMap, f32::consts, mem, rc::Rc};
use wgpu::util::DeviceExt;

const REGIONS: [[i32; 3]; 8] = [
    [-1, -1, -1],
    [-1, -1, 0],
    [-1, 0, -1],
    [-1, 0, 0],
    [0, -1, -1],
    [0, -1, 0],
    [0, 0, -1],
    [0, 0, 0],
];

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    transform: [[f32; 4]; 4], // This represents a single transformation matrix.
}

pub struct Vox {
    pub eye: glam::Vec3,
    pub horizontal_rotation: f32,
    pub vertical_rotation: f32,
    projection_matrix: glam::Mat4,
    depth_buffer: wgpu::TextureView,
    map: Map,
    chunks: BTreeMap<[i32; 3], Rc<Chunk>>,
    buffers: BTreeMap<[i32; 3], Rc<(wgpu::Buffer, wgpu::Buffer, u32)>>,
    bind_group: wgpu::BindGroup,
    uniform_vp_buffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl Vox {
    fn get_chunk(&mut self, x: i32, y: i32, z: i32) -> Rc<Chunk> {
        if !self.chunks.contains_key(&[x, y, z]) {
            let result = Rc::new(self.map.get_chunk(x, y, z));
            self.chunks.insert([x, y, z], result);
        }
        Rc::clone(self.chunks.get(&[x, y, z]).unwrap())
    }

    fn get_buffers(
        &mut self,
        device: &wgpu::Device,
        x: i32,
        y: i32,
        z: i32,
    ) -> Rc<(wgpu::Buffer, wgpu::Buffer, u32)> {
        if !self.buffers.contains_key(&[x, y, z]) {
            let result = Rc::new(self.create_buffers(device, x, y, z));
            self.buffers.insert([x, y, z], result);
        }
        Rc::clone(self.buffers.get(&[x, y, z]).unwrap())
    }

    fn create_buffers(
        &mut self,
        device: &wgpu::Device,
        x: i32,
        y: i32,
        z: i32,
    ) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let chunk = self.get_chunk(x, y, z);
        let chunk_px = self.get_chunk(x + 1, y, z);
        let chunk_nx = self.get_chunk(x - 1, y, z);
        let chunk_py = self.get_chunk(x, y + 1, z);
        let chunk_ny = self.get_chunk(x, y - 1, z);
        let chunk_pz = self.get_chunk(x, y, z + 1);
        let chunk_nz = self.get_chunk(x, y, z - 1);

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

        return (vertex_buf, index_buf, index_data.len() as u32);
    }

    fn generate_projection_matrix(aspect_ratio: f32) -> glam::Mat4 {
        glam::Mat4::perspective_rh(consts::FRAC_PI_4, aspect_ratio, 1.0, 1000.0)
    }

    pub fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
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

        let map = Map::new(42);
        let chunk_x = 0;
        let chunk_y = 0;
        let chunk_z = 0;
        let chunk = map.get_chunk(chunk_x, chunk_y, chunk_z);
        let chunk_px = map.get_chunk(chunk_x + 1, chunk_y, chunk_z);
        let chunk_nx = map.get_chunk(chunk_x - 1, chunk_y, chunk_z);
        let chunk_py = map.get_chunk(chunk_x, chunk_y + 1, chunk_z);
        let chunk_ny = map.get_chunk(chunk_x, chunk_y - 1, chunk_z);
        let chunk_pz = map.get_chunk(chunk_x, chunk_y, chunk_z + 1);
        let chunk_nz = map.get_chunk(chunk_x, chunk_y, chunk_z - 1);
        let (vertex_data, index_data) = create_vertices_for_chunk(
            &chunk, chunk_x, chunk_y, chunk_z, &chunk_px, &chunk_nx, &chunk_py, &chunk_ny,
            &chunk_pz, &chunk_nz,
        );

        // Create pipeline layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
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
        let size: u32 = 16;
        let texels = load_texture_from_png("terrain.png", 32, 48, 16, 16);
        let texture_extent = wgpu::Extent3d {
            width: size,
            height: size,
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
                bytes_per_row: Some(size * 4),
                rows_per_image: None,
            },
            texture_extent,
        );

        let min_alignment = device.limits().min_uniform_buffer_offset_alignment as usize;
        let uniform_size = std::mem::size_of::<Uniforms>();
        let aligned_uniform_size =
            ((uniform_size + min_alignment - 1) / min_alignment) * min_alignment;

        // Create other resources
        let uniform_vp_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform VP Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: aligned_uniform_size as wgpu::BufferAddress,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_vp_buffer.as_entire_binding(),
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
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader_rgba8.wgsl"))),
        });

        let vertex_size = mem::size_of::<Vertex>();

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
            map,
            chunks: BTreeMap::new(),
            buffers: BTreeMap::new(),
            bind_group,
            uniform_vp_buffer,
            pipeline,
        }
    }

    pub fn update(&mut self, _event: winit::event::WindowEvent) {
        //empty
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

    pub fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let dir = (glam::Mat3::from_rotation_z(self.horizontal_rotation)
                * glam::Mat3::from_rotation_x(self.vertical_rotation))
                * glam::Vec3::Y;
            let view_matrix = glam::Mat4::look_to_rh(self.eye, dir, glam::Vec3::Z);
            let mx_total = self.projection_matrix * view_matrix;
            let mx_ref: &[f32; 16] = mx_total.as_ref();
            queue.write_buffer(&self.uniform_vp_buffer, 0, bytemuck::cast_slice(mx_ref));
        }

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.5,
                            g: 0.5,
                            b: 0.5,
                            a: 1.0,
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
            for [x, y, z] in REGIONS {
                let buffers = self.get_buffers(device, x, y, z);
                rpass.push_debug_group("Prepare data for draw.");
                rpass.set_pipeline(&self.pipeline);
                rpass.set_index_buffer(buffers.1.slice(..), wgpu::IndexFormat::Uint16);
                rpass.set_vertex_buffer(0, buffers.0.slice(..));
                rpass.pop_debug_group();
                rpass.insert_debug_marker("Draw!");
                rpass.set_bind_group(0, &self.bind_group, &[]);
                rpass.draw_indexed(0..buffers.2, 0, 0..1);
            }
        }

        queue.submit(Some(encoder.finish()));
    }
}