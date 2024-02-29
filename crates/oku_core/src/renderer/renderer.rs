use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use glam;
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

    // Window and Surface must have the same lifetime scope and it must be dropped after the Surface.
    window: Arc<winit::window::Window>,
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
                            render_context = create_render_context(value.window).await;
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
                        let output = render_context.as_ref().unwrap().surface.get_current_texture().unwrap();
                        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                        let mut encoder = render_context.as_ref().unwrap().device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        });

                            {
                            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                            }

                        render_context.as_ref().unwrap().queue.submit(std::iter::once(encoder.finish()));
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

async fn create_render_context(window: Arc<Window>) -> Option<RenderContext<'static>> {
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

    let render_context = RenderContext {
        surface,
        device,
        surface_config: config,
        queue,
        window,
    };

    Some(render_context)
}