use crate::engine::renderer::color::Color;
use crate::engine::renderer::renderer::{Rectangle, Renderer};
use glam;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, GenericImageView, ImageEncoder};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::{CompositeAlphaMode, PresentMode};
use winit::window::Window;

pub struct WgpuRenderer<'a> {
    device: wgpu::Device,
    queue: wgpu::Queue,

    surface: wgpu::Surface<'a>,
    surface_clear_color: Color,
    surface_config: wgpu::SurfaceConfiguration,

    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    rectangle_render_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    rectangle_batch: Vec<RectangleBatch>,
    textures: HashMap<String, Texture>,
    rectangle_default_white_texture: Texture,
}

struct RectangleBatch {
    texture_path: Option<String>,
    rectangle_vertices: Vec<Vertex>,
    rectangle_indices: Vec<u32>,
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn from_bytes(device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[u8], label: &str) -> Option<Self> {
        let img = image::load_from_memory(bytes);
        Self::from_image(device, queue, &img.unwrap(), Some(label))
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Option<Self> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Some(Self {
            texture,
            view,
            sampler,
        })
    }
}

async fn request_adapter(instance: wgpu::Instance, surface: &wgpu::Surface<'_>) -> wgpu::Adapter {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        })
        .await
        .expect("Failed to request an adapter, cannot request GPU access without an adapter.");
    adapter
}

async fn request_device_and_queue(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: wgpu::Label::from("oku_wgpu_renderer"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            },
            None, // Trace path
        )
        .await
        .expect("Failed to request a GPU!");
    (device, queue)
}

fn create_surface_config(
    surface: &wgpu::Surface<'_>,
    width: u32,
    height: u32,
    _device: &wgpu::Device,
    adapter: &wgpu::Adapter,
) -> wgpu::SurfaceConfiguration {
    let surface_caps = surface.get_capabilities(adapter);

    // Require that we use a surface with a srgb format.
    surface_caps.formats.iter().copied().find(|f| f.is_srgb()).expect("Failed to find a SRGB surface!");

    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Rgba8Unorm,
        width,
        height,
        present_mode: PresentMode::Fifo,
        desired_maximum_frame_latency: 0,
        alpha_mode: CompositeAlphaMode::Auto,
        view_formats: vec![],
    }
}

fn bind_group_from_2d_texture(
    device: &wgpu::Device,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    texture: &Texture,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            },
        ],
        label: Some("oku_bind_group"),
    })
}

fn generate_default_white_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> Texture {
    let white_pixel = image::Rgba([255, 255, 255, 255]);
    let image_buffer = image::ImageBuffer::from_pixel(1, 1, white_pixel);

    let mut cursor = Cursor::new(Vec::new());
    let encoder = PngEncoder::new(&mut cursor);

    // Encode the image as a PNG and write the image bytes into our cursor.
    encoder
        .write_image(image_buffer.as_raw(), 1, 1, ExtendedColorType::Rgba8)
        .expect("Failed to create a default texture");

    Texture::from_bytes(device, queue, cursor.get_mut().as_slice(), "1x1_white_texture")
        .expect("Failed to create the default texture")
}

impl<'a> WgpuRenderer<'a> {
    pub(crate) async fn new(window: Arc<Window>) -> WgpuRenderer<'a> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12 | wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = request_adapter(instance, &surface).await;
        let (device, queue) = request_device_and_queue(&adapter).await;

        let surface_size = window.inner_size();
        let surface_config =
            create_surface_config(&surface, surface_size.width, surface_size.height, &device, &adapter);
        surface.configure(&device, &surface_config);

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
            z_near: 0.0,
            z_far: 100.0,
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
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Vertex::description()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
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

        let default_texture = generate_default_white_texture(&device, &queue);

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
            texture_bind_group_layout,
            rectangle_batch: vec![],
            textures: HashMap::new(),
            rectangle_default_white_texture: default_texture,
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
        self.surface_config.width = width as u32;
        self.surface_config.height = height as u32;
        self.surface.configure(&self.device, &self.surface_config);
        self.camera = Camera {
            width,
            height,
            z_near: 0.0,
            z_far: 100.0,
        };

        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform.view_proj]));
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.surface_clear_color = color;
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

        // Append to the current batch or create a new one.
        let current_batch =
            if !self.rectangle_batch.is_empty() && self.rectangle_batch.last().unwrap().texture_path.is_none() {
                self.rectangle_batch.last_mut().unwrap()
            } else {
                self.rectangle_batch.push(RectangleBatch {
                    texture_path: None,
                    rectangle_vertices: vec![],
                    rectangle_indices: vec![],
                });
                self.rectangle_batch.last_mut().unwrap()
            };

        current_batch.rectangle_vertices.append(&mut vec![
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

        let next_starting_index: u32 = (current_batch.rectangle_indices.len() / 6) as u32 * 4;
        current_batch.rectangle_indices.append(&mut vec![
            next_starting_index,
            next_starting_index + 1,
            next_starting_index + 2,
            next_starting_index + 2,
            next_starting_index + 1,
            next_starting_index + 3,
        ]);
    }

    fn draw_image(&mut self, rectangle: Rectangle, path: &str) {
        let x = rectangle.x;
        let y = rectangle.y;
        let width = rectangle.width;
        let height = rectangle.height;

        let top_left = [x, y, 0.0];
        let bottom_left = [x, y + height, 0.0];
        let top_right = [x + width, y, 0.0];
        let bottom_right = [x + width, y + height, 0.0];

        let color = [255.0, 255.0, 255.0, 255.0];

        // For now, always create a new batch when rendering an image
        let current_batch = {
            self.rectangle_batch.push(RectangleBatch {
                texture_path: Some(path.to_string()),
                rectangle_vertices: vec![],
                rectangle_indices: vec![],
            });
            self.rectangle_batch.last_mut().unwrap()
        };

        current_batch.rectangle_vertices.append(&mut vec![
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

        let next_starting_index: u32 = (current_batch.rectangle_indices.len() / 6) as u32 * 4;
        current_batch.rectangle_indices.append(&mut vec![
            next_starting_index,
            next_starting_index + 1,
            next_starting_index + 2,
            next_starting_index + 2,
            next_starting_index + 1,
            next_starting_index + 3,
        ]);
    }

    fn submit(&mut self) {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let output = self.surface.get_current_texture().unwrap();
        let texture_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut vertex_buffers: Vec<wgpu::Buffer> = vec![];
        let mut index_buffers: Vec<wgpu::Buffer> = vec![];
        let mut bind_groups: Vec<wgpu::BindGroup> = vec![];

        {
            for batch in self.rectangle_batch.iter_mut() {
                // Get the batch texture or use the default white texture if we cannot find the batch texture.
                let texture = if let Some(texture_path) = batch.texture_path.clone() {
                    if let Some(texture) = self.textures.get(&texture_path) {
                        texture
                    } else {
                        // If we were given an image path, but it isn't in our texture cache then try to load the image from the filesystem.
                        // Fallback to the default texture if that fails.
                        let mut rectangle_texture: &Texture = &self.rectangle_default_white_texture;
                        let image_reader = image::io::Reader::open(texture_path.clone());
                        if image_reader.is_ok() {
                            let image_reader = image_reader.unwrap();
                            let decoded_image = image_reader.decode();

                            if decoded_image.is_ok() {
                                let decoded_image = decoded_image.unwrap();
                                let texture = Texture::from_image(&self.device, &self.queue, &decoded_image, None);
                                if let Some(texture) = texture {
                                    self.textures.insert(texture_path.clone(), texture);
                                    rectangle_texture = self.textures.get(&texture_path.clone()).unwrap();
                                }
                            }
                        }

                        rectangle_texture
                    }
                } else {
                    &self.rectangle_default_white_texture
                };

                let oku_bind_group = bind_group_from_2d_texture(&self.device, &self.texture_bind_group_layout, texture);

                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&batch.rectangle_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&batch.rectangle_indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

                vertex_buffers.push(vertex_buffer);
                index_buffers.push(index_buffer);
                bind_groups.push(oku_bind_group);
            }
        }

        //let r = ((self.surface_clear_color.r / 255.0 + 0.055) / 1.055).powf(2.4);
        //let g = ((self.surface_clear_color.g / 255.0 + 0.055) / 1.055).powf(2.4);
        //let b = ((self.surface_clear_color.b / 255.0 + 0.055) / 1.055).powf(2.4);
        let r = self.surface_clear_color.r / 255.0;
        let g = self.surface_clear_color.g / 255.0;
        let b = self.surface_clear_color.b / 255.0;

        {
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

            for (index, batch) in self.rectangle_batch.iter_mut().enumerate() {
                {
                    _render_pass.set_pipeline(&self.rectangle_render_pipeline);

                    _render_pass.set_bind_group(0, bind_groups.get(index).unwrap(), &[]);
                    _render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
                    _render_pass.set_vertex_buffer(0, vertex_buffers.get(index).unwrap().slice(..));
                    _render_pass
                        .set_index_buffer(index_buffers.get(index).unwrap().slice(..), wgpu::IndexFormat::Uint32);
                    _render_pass.draw_indexed(0..(batch.rectangle_indices.len() as u32), 0, 0..1);
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.rectangle_batch.clear();
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
    z_near: f32,
    z_far: f32,
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
        let proj = glam::Mat4::orthographic_lh(0.0, self.width, self.height, 0.0, self.z_near, self.z_far);
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
