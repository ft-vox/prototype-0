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
    texture_sheet_count: u32,
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
    tex_layer: u32,      // texture array layer index
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
    pub base_vertices: Vec<UIVertex>, // Store original vertices for transformation
}

#[derive(Clone, Copy)]
pub struct UITransform {
    pub position: Vec2,
    pub size: Vec2,
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

        let (texels_array, width, height, layer_count) = load_texture_sheets();
        let ui_texture_extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: layer_count,
        };
        let ui_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("UI Texture Array"),
            size: ui_texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let ui_texture_view = ui_texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        // Write each texture layer
        for (layer_index, texels) in texels_array.iter().enumerate() {
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &ui_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer_index as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                texels,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
        }

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
                        view_dimension: wgpu::TextureViewDimension::D2Array,
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
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32,
                            offset: 16,
                            shader_location: 2,
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
            texture_sheet_count: layer_count,
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
        ui_elements: &[(UIMeshWGPU, UITransform)],
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

        for (mesh, transform) in ui_elements {
            // Update vertex positions based on transform
            let mut transformed_vertices = mesh.base_vertices.clone();
            for vertex in &mut transformed_vertices {
                let mapped_position =
                    self.map_position(transform.position, self.ui_area_logical_size);
                let mapped_size = transform.size / self.ui_area_logical_size * 2.0;

                // Calculate relative position within the UI element
                let relative_x = (vertex.position[0] + 1.0) / 2.0;
                let relative_y = (-vertex.position[1] + 1.0) / 2.0;

                // Apply new transform
                vertex.position[0] = mapped_position.x + relative_x * mapped_size.x;
                vertex.position[1] = mapped_position.y - relative_y * mapped_size.y;
            }

            // Update vertex buffer with new positions
            queue.write_buffer(
                &mesh.vertex_buffer,
                0,
                bytemuck::cast_slice(&transformed_vertices),
            );

            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
    }

    fn map_position(&self, position: Vec2, logical_size: Vec2) -> Vec2 {
        Vec2::new(
            (position.x / logical_size.x) * 2.0 - 1.0,
            (1.0 - (position.y / logical_size.y)) * 2.0 - 1.0,
        )
    }

    // TODO: Separate to UI FRAMEWORK
    pub fn create_ui_mesh(
        &mut self,
        device: &wgpu::Device,
        position: Vec2,
        size: Vec2,
        texture_position: Vec2,
        texture_size: Vec2,
        texture_layer: u32,
    ) -> (UIMeshWGPU, UITransform) {
        let mut mesh = UIMesh {
            vertices: Vec::new(),
            indices: Vec::new(),
            index_count: 0,
        };

        let mapped_texture_position = texture_position / self.ui_texture_sheet_size;
        let mapped_texture_size = texture_size / self.ui_texture_sheet_size;

        let base_vertices = vec![
            UIVertex {
                position: [-1.0, 1.0],
                tex_coord: [mapped_texture_position.x, mapped_texture_position.y],
                tex_layer: texture_layer,
            },
            UIVertex {
                position: [1.0, 1.0],
                tex_coord: [
                    mapped_texture_position.x + mapped_texture_size.x,
                    mapped_texture_position.y,
                ],
                tex_layer: texture_layer,
            },
            UIVertex {
                position: [-1.0, -1.0],
                tex_coord: [
                    mapped_texture_position.x,
                    mapped_texture_position.y + mapped_texture_size.y,
                ],
                tex_layer: texture_layer,
            },
            UIVertex {
                position: [1.0, -1.0],
                tex_coord: [
                    mapped_texture_position.x + mapped_texture_size.x,
                    mapped_texture_position.y + mapped_texture_size.y,
                ],
                tex_layer: texture_layer,
            },
        ];

        mesh.vertices = base_vertices.clone();
        mesh.indices = vec![0, 1, 2, 1, 3, 2];
        mesh.index_count = 6;

        let mesh_wgpu = UIMeshWGPU {
            vertex_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("UI Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }),
            index_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("UI Index Buffer"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            }),
            index_count: mesh.index_count,
            base_vertices,
        };

        (mesh_wgpu, UITransform { position, size })
    }
}

fn load_texture_sheets() -> (Vec<Vec<u8>>, u32, u32, u32) {
    use image::ImageFormat;

    const TEXTURE_SHEETS: [&[u8]; 2] = [
        include_bytes!("../../assets/ui-sheet-0000.png"),
        include_bytes!("../../assets/ui-sheet-0001.png"),
    ];

    let mut texels_array = Vec::new();
    let mut width = 0;
    let mut height = 0;

    for (index, &data) in TEXTURE_SHEETS.iter().enumerate() {
        let img = image::load_from_memory_with_format(data, ImageFormat::Png)
            .unwrap_or_else(|_| panic!("Failed to load texture sheet. index: {}", index));

        let (img_width, img_height) = img.dimensions();

        if texels_array.is_empty() {
            width = img_width;
            height = img_height;
        } else if width != img_width || height != img_height {
            panic!(
                "All UI texture sheets must have the same dimensions. Mismatch at index {}",
                index
            );
        }

        texels_array.push(img.to_rgba8().into_raw());
    }

    let layer_count = texels_array.len() as u32;

    (texels_array, width, height, layer_count)
}
