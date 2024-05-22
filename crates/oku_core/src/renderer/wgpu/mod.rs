use crate::renderer::color::Color;
use crate::renderer::renderer::{Rectangle, Renderer};
use glam;
use std::mem::size_of;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::{BlendState, CompositeAlphaMode, PresentMode};
use winit::window::Window;

pub struct WgpuRenderer<'a> {
    device: wgpu::Device,
    surface: wgpu::Surface<'a>,
    surface_clear_color: Color,
    surface_config: wgpu::SurfaceConfiguration,
    queue: wgpu::Queue,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    rectangle_render_pipeline: wgpu::RenderPipeline,
    rectangle_bind_group: wgpu::BindGroup,
    rectangle_vertices: Vec<Vertex>,
    rectangle_indices: Vec<u32>,
}

impl<'a> WgpuRenderer<'a> {
    pub(crate) async fn new(window: Arc<Window>) -> WgpuRenderer<'a> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12 | wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: wgpu::Label::from("oku_wgpu_renderer"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(surface_caps.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Rgba8Unorm,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 0,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let oku_image_bytes = include_bytes!("oku.png");
        let oku_image = image::load_from_memory(oku_image_bytes).unwrap();
        let oku_image_rgba = oku_image.to_rgba8();

        let texture_size = wgpu::Extent3d {
            width: oku_image.width(),
            height: oku_image.height(),
            depth_or_array_layers: 1,
        };

        let oku_image_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("oku_image_texture"),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &oku_image_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &oku_image_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * oku_image.width()),
                rows_per_image: Some(oku_image.height()),
            },
            texture_size,
        );

        let oku_image_texture_view = oku_image_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let oku_image_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let camera = Camera {
            width: window.inner_size().width as f32,
            height: window.inner_size().height as f32,
            znear: 0.0,
            zfar: 100.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let oku_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&oku_image_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&oku_image_sampler),
                },
            ],
            label: Some("oku_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[Vertex::description()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        WgpuRenderer {
            device,
            surface,
            surface_clear_color: Color::new_from_rgba_u8(255, 255, 255, 255),
            surface_config,
            queue,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            rectangle_render_pipeline: render_pipeline,
            rectangle_bind_group: oku_bind_group,
            rectangle_vertices: vec![],
            rectangle_indices: vec![],
        }
    }
}

impl Renderer for WgpuRenderer<'_> {
    fn surface_width(&self) -> f32 {
        self.surface_config.width as f32
    }

    fn surface_height(&self) -> f32 {
        self.surface_config.height as f32
    }

    fn present_surface(&mut self) {
        todo!()
    }

    fn resize_surface(&mut self, width: f32, height: f32) {
        //println!("{}, {}", width, height);
        self.surface_config.width = width as u32;
        self.surface_config.height = height as u32;
        self.surface.configure(&self.device, &self.surface_config);
        self.camera = Camera {
            width,
            height,
            znear: 0.0,
            zfar: 100.0,
        };

        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform.view_proj]));
    }

    fn draw_rect(&mut self, rectangle: Rectangle, fill_color: Color) {
        let x = rectangle.x;
        let y = rectangle.y;
        let width = rectangle.width;
        let height = rectangle.height;

        let top_left = [x, y, 0.0];
        let bottom_left = [x, y + height, 0.0];
        let top_right = [x + width, y, 0.0];
        let bottom_right = [x + width, y + height, 0.0];

        let color = [fill_color.r, fill_color.g, fill_color.b, fill_color.a];

        self.rectangle_vertices.append(&mut vec![
            Vertex {
                position: top_left,
                color,
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: bottom_left,
                color,
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: top_right,
                color,
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: bottom_right,
                color,
                tex_coords: [1.0, 1.0],
            },
        ]);

        let next_starting_index: u32 = (self.rectangle_indices.len() / 6) as u32 * 4;
        self.rectangle_indices.append(&mut vec![next_starting_index + 0, next_starting_index + 1, next_starting_index + 2, next_starting_index + 2, next_starting_index + 1, next_starting_index + 3]);
    }

    fn submit(&mut self) {
        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.rectangle_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&self.rectangle_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let output = self.surface.get_current_texture().unwrap();
        let texture_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let width = output.texture.width() as f32;
        let height = output.texture.height() as f32;

        {
            //let r = ((self.surface_clear_color.r / 255.0 + 0.055) / 1.055).powf(2.4);
            //let g = ((self.surface_clear_color.g / 255.0 + 0.055) / 1.055).powf(2.4);
            //let b = ((self.surface_clear_color.b / 255.0 + 0.055) / 1.055).powf(2.4);
            let r = self.surface_clear_color.r / 255.0;
            let g = self.surface_clear_color.g / 255.0;
            let b = self.surface_clear_color.b / 255.0;

            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: r as f64,
                            g: g as f64,
                            b: b as f64,
                            a: self.surface_clear_color.a as f64 / 255.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            _render_pass.set_pipeline(&self.rectangle_render_pipeline);
            _render_pass.set_bind_group(0, &self.rectangle_bind_group, &[]);
            _render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            _render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            _render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            _render_pass.draw_indexed(0..(self.rectangle_indices.len() as u32), 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.rectangle_indices.clear();
        self.rectangle_vertices.clear();
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.surface_clear_color = color;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
    tex_coords: [f32; 2],
}

struct Camera {
    width: f32,
    height: f32,
    znear: f32,
    zfar: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().to_cols_array_2d();
    }
}

impl Camera {
    fn build_view_projection_matrix(&self) -> glam::Mat4 {
        let view = glam::Mat4::IDENTITY;
        let proj = glam::Mat4::orthographic_lh(0.0, self.width, self.height, 0.0, self.znear, self.zfar);
        proj * view
    }
}

impl Vertex {
    fn description<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
