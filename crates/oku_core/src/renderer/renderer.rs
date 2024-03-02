use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use glam;
use wgpu::util::DeviceExt;
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopBuilder};
use winit::keyboard::{Key, NamedKey};
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
use winit::window::{Window, WindowId};

struct LogicalSize<P> {
    width: P,
    height: P,
}

struct PhysicalSize<P> {
    width: P,
    height: P,
}

/*trait Surface {
    fn size(&self) -> PhysicalSize<u32>;
}*/

trait Renderer {
    // fn quad_pipeline() -> QuadPipeline;
}

/*enum Device {
    WgpuDevice(wgpu::Device),
    Software,
}*/

struct RenderContext<'a> {
    /*  surface: Box<dyn Surface>,
      renderer: Box<dyn Renderer>,*/
    surface: wgpu::Surface<'a>,
    surface_config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    oku_bind_group: wgpu::BindGroup,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    // Window and Surface must have the same lifetime scope and it must be dropped after the Surface.
    window: Arc<winit::window::Window>,
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [250.0, 250.0, 0.0], color: [0.5, 0.0, 0.2], tex_coords: [0.0, 0.0] },
    Vertex { position: [250.0, 500.0, 0.0], color: [0.5, 0.0, 0.2], tex_coords: [0.0, 1.0] },
    Vertex { position: [500.0, 250.0, 0.0], color: [0.5, 0.0, 0.5], tex_coords: [1.0, 0.0] },
    Vertex { position: [500.0, 500.0, 0.0], color: [0.5, 0.0, 0.5], tex_coords: [1.0, 1.0] },
];

const INDICES: &[u16] = &[
    0, 1, 2,
    2, 1, 3
];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
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
        return  proj * view;
    }
}

impl Vertex {

    fn description() -> wgpu::VertexBufferLayout<'static> {
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
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ]
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum ActionRequestEvent {
    WakeUp,
}

struct Snapshot {
    event: winit::event::Event<ActionRequestEvent>,
    window: Arc<winit::window::Window>,
}

pub fn wgpu_integration() {
    env_logger::init();
    let mut winit_event_loop = EventLoop::<ActionRequestEvent>::with_user_event().build().unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();

    async fn async_operation(mut rx: tokio::sync::mpsc::Receiver<Snapshot>) {
        let mut render_context: Option<RenderContext> = None;
        let mut should_draw = false;

        loop {
            tokio::select! {
            value = rx.recv() => {
                    if(value.is_none()) {
                        return;
                    }
                    let mut value = value.unwrap();

                    match value.event {
                        Event::Resumed => {
                            render_context = RenderContext::new(value.window).await;
                            should_draw = true;
                        },
                        Event::WindowEvent { window_id, event } => match event {
                            WindowEvent::Resized(size) => {
                                  if size.width > 0 && size.height > 0 {
                                    let mut render_context = render_context.as_mut().unwrap();
                                    render_context.surface_config.width = size.width;
                                    render_context.surface_config.height = size.height;
                                    render_context.surface.configure(&render_context.device, &render_context.surface_config);
                                }
                            }
                            _ => {
                                should_draw = true;
                            },
                        }
                        _ => {},
                    }

                    if should_draw {
                        let render_context = render_context.as_ref().unwrap();
                        let output = render_context.surface.get_current_texture().unwrap();
                        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                        let mut encoder = render_context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        });

                            {
                            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("Render Pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color {
                                            r: 0.1,
                                            g: 0.2,
                                            b: 0.3,
                                            a: 1.0,
                                        }),
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                occlusion_query_set: None,
                                timestamp_writes: None,
                            });
                                _render_pass.set_pipeline(&render_context.render_pipeline);
                                _render_pass.set_bind_group(0, &render_context.oku_bind_group, &[]);
                                _render_pass.set_bind_group(1, &render_context.camera_bind_group, &[]);
                                _render_pass.set_vertex_buffer(0, render_context.vertex_buffer.slice(..));
                                _render_pass.set_index_buffer(render_context.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                                _render_pass.draw_indexed(0..(INDICES.len() as u32), 0, 0..1);
                            }

                        render_context.queue.submit(std::iter::once(encoder.finish()));
                        output.present();
                        should_draw = false;
                    }

            }
            }
        }
    }

    let (tx, rx) = tokio::sync::mpsc::channel::<Snapshot>(1);
    rt.spawn(async_operation(rx));

    let mut windows: HashMap<WindowId, Arc<Window>> = HashMap::new();
    let mut current_window: Option<Arc<Window>> = None;

    winit_event_loop.run(|event: Event<ActionRequestEvent>, event_loop_window_target: &ActiveEventLoop| {
        event_loop_window_target.set_control_flow(ControlFlow::Wait);

        let clone_event = event.clone();
        let current_window_check_event = event.clone();
        match current_window_check_event {
            Event::WindowEvent { window_id, event } => {
                let current_window_value = windows.get(&window_id);
                if current_window_value.is_some() {
                    current_window = Some(windows.get(&window_id).unwrap().clone());
                }
            }
            _ => {}
        }

        match event {
            Event::WindowEvent { window_id, event } => match event {
                WindowEvent::ActivationTokenDone { .. } => {}
                WindowEvent::Moved(_) => {}
                WindowEvent::CloseRequested => {
                    event_loop_window_target.exit();
                }
                WindowEvent::Destroyed => {}
                WindowEvent::DroppedFile(_) => {}
                WindowEvent::HoveredFile(_) => {}
                WindowEvent::HoveredFileCancelled => {}
                WindowEvent::Focused(_) => {}
                WindowEvent::KeyboardInput {
                    device_id: _device_id,
                    event: _event,
                    is_synthetic: _is_synthetic,
                } => {
                    if _event.state == ElementState::Pressed {}
                }
                WindowEvent::ModifiersChanged(_) => {}
                WindowEvent::Ime(_) => {}
                WindowEvent::Resized(size) => {
                    rt.block_on(async {
                        tx.send(Snapshot {
                            event: clone_event,
                            window: current_window.clone().unwrap(),
                        }).await.expect("TODO: panic message");
                    })
                }
                WindowEvent::CursorMoved { .. } | WindowEvent::RedrawRequested => {
                    rt.block_on(async {
                        tx.send(Snapshot {
                            event: clone_event,
                            window: current_window.clone().unwrap(),
                        }).await.expect("TODO: panic message");
                    })
                }
                WindowEvent::CursorEntered { .. } => {}
                WindowEvent::CursorLeft { .. } => {}
                WindowEvent::MouseWheel { .. } => {}
                WindowEvent::MouseInput { .. } => {}
                WindowEvent::PinchGesture { .. } => {}
                WindowEvent::DoubleTapGesture { .. } => {}
                WindowEvent::RotationGesture { .. } => {}
                WindowEvent::TouchpadPressure { .. } => {}
                WindowEvent::AxisMotion { .. } => {}
                WindowEvent::Touch(_) => {}
                WindowEvent::ScaleFactorChanged { .. } => {}
                WindowEvent::ThemeChanged(_) => {}
                WindowEvent::Occluded(_) => {}
            },
            Event::Resumed => {
                let window_attributes = Window::default_attributes().with_title("oku").with_transparent(false);
                let window = event_loop_window_target.create_window(window_attributes).expect("failed to create window");
                let window_id = window.id();
                windows.insert(window_id, Arc::new(window));

                current_window = Some(windows.get(&window_id).unwrap().clone());

                rt.block_on(async {
                    tx.send(Snapshot {
                        event: clone_event,
                        window: current_window.clone().unwrap(),
                    }).await.expect("TODO: panic message");
                })
            }
            Event::NewEvents(_) => {}
            Event::DeviceEvent { .. } => {}
            Event::UserEvent(_) => {}
            Event::Suspended => {}
            Event::AboutToWait => {}
            Event::LoopExiting => {}
            Event::MemoryWarning => {}
        }
    }).unwrap();
}

impl RenderContext<'_> {
    async fn new(window: Arc<Window>) -> Option<RenderContext<'static>> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: wgpu::Label::from("oku_wgpu_renderer"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None, // Trace path
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            // Filter for SRGB compatible surfaces.
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 0,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);


        let oku_image_bytes = include_bytes!("oku.png");
        let oku_image = image::load_from_memory(oku_image_bytes).unwrap();
        let oku_image_rgba = oku_image.to_rgba8();

        use image::GenericImageView;

        let texture_size = wgpu::Extent3d {
            width: oku_image.width(),
            height: oku_image.height(),
            depth_or_array_layers: 1,
        };

        let oku_image_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("oku_image_texture"),
                view_formats: &[],
            }
        );

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

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        let oku_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&oku_image_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&oku_image_sampler),
                    }
                ],
                label: Some("oku_bind_group"),
            }
        );

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
                buffers: &[
                    Vertex::description()
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
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
        });

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let render_context = RenderContext {
            surface,
            device,
            surface_config: config,
            queue,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            oku_bind_group,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            window,
        };

        Some(render_context)
    }
}