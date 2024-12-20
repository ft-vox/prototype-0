use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3};
use image::GenericImageView;
use wgpu::util::DeviceExt;

pub struct UIRenderer {
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    transform: Mat4,
    ui_area_logical_size: Vec2,
    ui_texture_sheet_size: Vec2,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct UIUniforms {
    transform: [f32; 16],
    opacity: f32,
    _padding: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct UIVertex {
    position: [f32; 2],  // -1.0 ~ 1.0
    tex_coord: [f32; 2], // 0.0 ~ 1.0
}

pub struct UIMesh {
    pub vertices: Vec<UIVertex>,
    pub indices: Vec<u16>,
    pub index_count: u32,
}

pub struct UIMeshWGPU {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl UIRenderer {
    pub fn init(
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../assets/shader_ui.wgsl"
            ))),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("UI Uniform Buffer"),
            size: std::mem::size_of::<UIUniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let (texels, width, height) = load_texture_from_ui_png();
        let ui_texture_extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let ui_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("UI Texture"),
            size: ui_texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let ui_texture_view = ui_texture.create_view(&wgpu::TextureViewDescriptor::default());
        queue.write_texture(
            ui_texture.as_image_copy(),
            &texels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: None,
            },
            ui_texture_extent,
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("UI Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("UI Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&ui_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_ui",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<UIVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_ui",
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
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            bind_group,
            uniform_buffer,
            pipeline,
            transform: Self::calculate_transform_matrix(config.width as f32, config.height as f32),
            ui_area_logical_size: Vec2::new(1600.0, 900.0),
            ui_texture_sheet_size: Vec2::new(width as f32, height as f32),
        }
    }
    fn calculate_transform_matrix(screen_width: f32, screen_height: f32) -> Mat4 {
        let target_aspect_ratio = 16.0 / 9.0;
        let screen_aspect_ratio = screen_width / screen_height;
        let (scale_x, scale_y) = if screen_aspect_ratio > target_aspect_ratio {
            let scale = screen_height / (screen_width / target_aspect_ratio);
            (scale, 1.0)
        } else {
            let scale = screen_width / (screen_height * target_aspect_ratio);
            (1.0, scale)
        };
        Mat4::from_scale(Vec3::new(scale_x, scale_y, 1.0))
    }

    pub fn resize(&mut self, config: &wgpu::SurfaceConfiguration) {
        self.transform =
            Self::calculate_transform_matrix(config.width as f32, config.height as f32);
    }

    pub fn render(
        &self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        ui_elements: &Vec<UIMeshWGPU>,
    ) {
        let uniforms = UIUniforms {
            transform: self.transform.to_cols_array(),
            opacity: 1.0,
            _padding: [0.0; 3],
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("UI Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);

        for ui_element in ui_elements {
            render_pass.set_vertex_buffer(0, ui_element.vertex_buffer.slice(..));
            render_pass
                .set_index_buffer(ui_element.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..ui_element.index_count, 0, 0..1);
        }
    }

    fn create_ui_mesh_wgpu(device: &wgpu::Device, mesh: &UIMesh) -> UIMeshWGPU {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("UI Index Buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        UIMeshWGPU {
            vertex_buffer,
            index_buffer,
            index_count: mesh.index_count,
        }
    }

    // TODO: Separate to UI FRAMEWORK
    pub fn create_ui_mesh(
        &mut self,
        device: &wgpu::Device,
        position: Vec2,
        size: Vec2,
        texture_position: Vec2,
        texture_size: Vec2,
    ) -> UIMeshWGPU {
        let mut mesh = UIMesh {
            vertices: Vec::new(),
            indices: Vec::new(),
            index_count: 0,
        };

        fn map_position(position: Vec2, logical_size: Vec2) -> Vec2 {
            Vec2::new(
                (position.x / logical_size.x) * 2.0 - 1.0,
                (1.0 - (position.y / logical_size.y)) * 2.0 - 1.0,
            )
        }
        let mapped_position = map_position(position, self.ui_area_logical_size);
        let mapped_size = size / self.ui_area_logical_size * 2.0;
        let mapped_texture_position = texture_position / self.ui_texture_sheet_size;
        let mapped_texture_size = texture_size / self.ui_texture_sheet_size;

        mesh.vertices = vec![
            UIVertex {
                position: [mapped_position.x, mapped_position.y],
                tex_coord: [mapped_texture_position.x, mapped_texture_position.y],
            },
            UIVertex {
                position: [mapped_position.x + mapped_size.x, mapped_position.y],
                tex_coord: [
                    mapped_texture_position.x + mapped_texture_size.x,
                    mapped_texture_position.y,
                ],
            },
            UIVertex {
                position: [mapped_position.x, mapped_position.y - mapped_size.y],
                tex_coord: [
                    mapped_texture_position.x,
                    mapped_texture_position.y + mapped_texture_size.y,
                ],
            },
            UIVertex {
                position: [
                    mapped_position.x + mapped_size.x,
                    mapped_position.y - mapped_size.y,
                ],
                tex_coord: [
                    mapped_texture_position.x + mapped_texture_size.x,
                    mapped_texture_position.y + mapped_texture_size.y,
                ],
            },
        ];

        mesh.indices = vec![0, 1, 2, 1, 3, 2];
        mesh.index_count = 6;
        Self::create_ui_mesh_wgpu(device, &mesh)
    }
}

// TODO: add Texture atlas system or Texture array system

fn load_texture_from_ui_png() -> (Vec<u8>, u32, u32) {
    let img = image::load_from_memory_with_format(
        include_bytes!("../../assets/ui-sheet.png"),
        image::ImageFormat::Png,
    )
    .expect("Failed to load UI texture");

    let (width, height) = img.dimensions();
    let rgba = img.to_rgba8();
    (rgba.into_raw(), width, height)
}
