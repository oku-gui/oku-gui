pub mod application;
pub mod components;
pub mod elements;
mod widget_id;

use crate::application::Application;
use crate::elements::element::Element;
use cosmic_text::{FontSystem, SwashCache};
use log::info;
//use softbuffer::Surface;
use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;
use std::{thread, time};
use tiny_skia::Pixmap;
use tokio::sync::mpsc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use wgpu::{Device, Queue, RenderPipeline, Surface, SurfaceConfiguration};

const WAIT_TIME: time::Duration = time::Duration::from_millis(100);

struct App<'a> {
    window: Option<Arc<Window>>,
    wgpu_instance: wgpu::Instance,
    renderer: Option<RenderState<'a>>,
}
struct RenderState<'a> {
    surface: Surface<'a>,
    device: Device,
    render_pipeline: RenderPipeline,
    queue: Queue,
    config: SurfaceConfiguration,
}

pub struct RenderContext {
    font_system: FontSystem,
    swash_cache: SwashCache,
    surface: softbuffer::Surface<Rc<Window>, Rc<Window>>,
    canvas: Pixmap,
    cursor_x: f32,
    cursor_y: f32,
    debug_draw: bool,
    window: Rc<Window>,
}

struct ControlFlowDemo {
    id: u64,
    rt: tokio::runtime::Runtime,
    request_redraw: bool,
    wait_cancelled: bool,
    close_requested: bool,
    window: Option<Arc<Window>>,
    app_to_winit_rx: mpsc::Receiver<(u64, Message)>,
    winit_to_app_tx: mpsc::Sender<(u64, Message)>,
}

#[derive(Debug, Clone)]
enum Message {
    RequestRedraw,
    Close,
    None,
    Resume(Arc<Window>),
    Resize(PhysicalSize<u32>),
}

pub fn oku_main(application: Box<dyn Application>) {
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Failed to create runtime");

    let event_loop = EventLoop::new().unwrap();

    let (winit_to_app_tx, winit_to_app_rx) = mpsc::channel::<(u64, Message)>(100);
    let (app_to_winit_tx, app_to_winit_rx) = mpsc::channel::<(u64, Message)>(100);

    rt.spawn(async move {
        async_main(winit_to_app_rx, app_to_winit_tx).await;
    });

    let mut app = ControlFlowDemo {
        id: 0,
        rt,
        request_redraw: false,
        wait_cancelled: false,
        close_requested: false,
        window: None,
        app_to_winit_rx,
        winit_to_app_tx,
    };

    event_loop.run_app(&mut app).expect("run_app failed");
}

impl ApplicationHandler for ControlFlowDemo {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        //println!("new_events: {cause:?}");

        self.wait_cancelled = match cause {
            StartCause::WaitCancelled { .. } => true,
            _ => false,
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        info!("resumed");
        let window_attributes = Window::default_attributes().with_title("Press 1, 2, 3 to change control flow mode. Press R to toggle redraw requests.");
        info!("resumed 4");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        info!("resumed 2");
        self.window = Some(window.clone());

        let id = self.id;
        info!("resumed 3");
        self.rt.block_on(async {
            self.winit_to_app_tx.send((id, Message::Resume(window.clone()))).await.expect("send failed");
            if let Some((id, Message::None)) = self.app_to_winit_rx.recv().await {
                println!("Resume Done: {}", id);
            }

            // web code
            info!("send message");
            let x = self.winit_to_app_tx.send((id, Message::Resume(window.clone())));
        });
        self.id += 1;
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        //println!("{event:?}");

        match event {
            WindowEvent::CloseRequested => {
                let id = self.id;
                self.rt.block_on(async {
                    self.winit_to_app_tx.send((id, Message::Close)).await.expect("send failed");
                    if let Some((id, Message::None)) = self.app_to_winit_rx.recv().await {
                        println!("Close Done: {}", id);
                    }
                });
                self.id += 1;
                self.close_requested = true;
            }
            WindowEvent::Resized(new_size) => {
                let id = self.id;
                self.rt.block_on(async {
                    self.winit_to_app_tx.send((id, Message::Resize(new_size))).await.expect("send failed");
                    if let Some((id, Message::None)) = self.app_to_winit_rx.recv().await {
                        println!("Resize Done: {}", id);
                    }
                });
                self.id += 1;
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    logical_key: key,
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => match key.as_ref() {
                Key::Character("r") => {
                    self.request_redraw = !self.request_redraw;
                    println!("\nrequest_redraw: {}\n", self.request_redraw);
                }
                Key::Named(NamedKey::Escape) => {
                    self.close_requested = true;
                }
                _ => (),
            },
            WindowEvent::RedrawRequested => {
                self.rt.block_on(async {
                    let id = self.id;
                    self.winit_to_app_tx.send((id, Message::RequestRedraw)).await.expect("send failed");
                    if let Some((id, Message::None)) = self.app_to_winit_rx.recv().await {
                        println!("Redraw Done: {}", id);
                    }
                });

                let window = self.window.as_ref().unwrap();
                window.pre_present_notify();
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.request_redraw && !self.wait_cancelled && !self.close_requested {
            self.window.as_ref().unwrap().request_redraw();
        }

        if !self.wait_cancelled {
            event_loop.set_control_flow(ControlFlow::WaitUntil(time::Instant::now() + WAIT_TIME));
        }

        if self.close_requested {
            event_loop.exit();
        }
    }
}

async fn async_main(mut rx: mpsc::Receiver<(u64, Message)>, mut tx: mpsc::Sender<(u64, Message)>) {
    let mut app = App {
        window: None,
        wgpu_instance: wgpu::Instance::default(),
        renderer: None,
    };

    loop {
        if let Some((id, msg)) = rx.recv().await {
            match msg {
                Message::RequestRedraw => {
                    println!("request_redraw");

                    let renderer = app.renderer.as_ref().unwrap();

                    let frame = renderer.surface.get_current_texture().expect("Failed to acquire next swap chain texture");
                    let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                    {
                        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: None,
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });
                        render_pass.set_pipeline(&renderer.render_pipeline);
                        render_pass.draw(0..3, 0..1);
                    }

                    renderer.queue.submit(Some(encoder.finish()));
                    frame.present();

                    tx.send((id, Message::None)).await.expect("send failed");
                }
                Message::Close => {
                    println!("close");
                    tx.send((id, Message::None)).await.expect("send failed");
                    break;
                }
                Message::None => {
                    println!("none");
                }
                Message::Resume(window) => {
                    println!("Resumed");

                    let size = window.inner_size();

                    let surface = app.wgpu_instance.create_surface(window.clone()).unwrap();

                    let adapter = app
                        .wgpu_instance
                        .request_adapter(&wgpu::RequestAdapterOptions {
                            power_preference: wgpu::PowerPreference::default(),
                            force_fallback_adapter: false,
                            // Request an adapter which can render to our surface
                            compatible_surface: Some(&surface),
                        })
                        .await
                        .expect("Failed to find an appropriate adapter");

                    // Create the logical device and command queue
                    let (device, queue) = adapter
                        .request_device(
                            &wgpu::DeviceDescriptor {
                                label: None,
                                required_features: wgpu::Features::empty(),
                                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                                required_limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
                            },
                            None,
                        )
                        .await
                        .expect("Failed to create device");

                    // Load the shaders from disk
                    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: None,
                        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/shader.wgsl"))),
                    });

                    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &[],
                        push_constant_ranges: &[],
                    });

                    let swapchain_capabilities = surface.get_capabilities(&adapter);
                    let swapchain_format = swapchain_capabilities.formats[0];

                    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: None,
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: "vs_main",
                            buffers: &[],
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: "fs_main",
                            targets: &[Some(swapchain_format.into())],
                        }),
                        primitive: wgpu::PrimitiveState::default(),
                        depth_stencil: None,
                        multisample: wgpu::MultisampleState::default(),
                        multiview: None,
                    });

                    let config = surface.get_default_config(&adapter, size.width, size.height).unwrap();
                    surface.configure(&device, &config);

                    app.window = Some(window.clone());
                    app.renderer = Some(RenderState {
                        surface,
                        device,
                        render_pipeline,
                        queue,
                        config,
                    });

                    tx.send((id, Message::None)).await.expect("send failed");
                }
                Message::Resize(new_size) => {
                    // Reconfigure the surface with the new size

                    let renderer = app.renderer.as_mut().unwrap();

                    renderer.config.width = new_size.width.max(1);
                    renderer.config.height = new_size.height.max(1);
                    renderer.surface.configure(&renderer.device, &renderer.config);

                    // On macOS the window needs to be redrawn manually after resizing
                    app.window.as_ref().unwrap().request_redraw();

                    tx.send((id, Message::None)).await.expect("send failed");
                }
            }
        }

        println!("Message processed");
    }
}
