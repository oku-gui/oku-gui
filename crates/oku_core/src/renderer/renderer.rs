use std::rc::Rc;
use std::sync::Arc;
use glam;
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopBuilder};
use winit::keyboard::{Key, NamedKey};
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
use winit::window::Window;

struct LogicalSize<P> {
    width: P,
    height: P,
}

struct PhysicalSize<P> {
    width: P,
    height: P,
}

trait Surface {
    fn size(&self) -> PhysicalSize<u32>;
}

trait Renderer {
    // fn quad_pipeline() -> QuadPipeline;
}

enum Device {
    WgpuDevice(wgpu::Device),
    Software,
}

struct RenderContext {
    surface: Box<dyn Surface>,
    renderer: Box<dyn Renderer>,
    device: Device,

    // Window and Surface must have the same lifetime scope and it must be dropped after the Surface.
    window: Arc<winit::window::Window>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum ActionRequestEvent {
    WakeUp,
}

pub fn wgpu_integration() {
    env_logger::init();
    let mut winit_event_loop = EventLoop::<ActionRequestEvent>::with_user_event().build().unwrap();
    let window_attributes = Window::default_attributes().with_title("oku").with_transparent(false);
    let window = Rc::new(winit_event_loop.create_window(window_attributes).expect("Failed to create window"));


    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut render_context: Option<RenderContext> = None;


    async fn async_operation(mut rx: tokio::sync::mpsc::Receiver<String>) {
        loop {
            tokio::select! {
            value = rx.recv() => {
                    if(value.is_some()) {
                        println!("Received event: {:?}", value.unwrap());
                    }
            }
            }
        }
    }

    let (tx, rx) = tokio::sync::mpsc::channel::<String>(32);
    rt.spawn(async_operation(rx));

    winit_event_loop.run(|event: Event<ActionRequestEvent>, event_loop_window_target: &ActiveEventLoop| {
        event_loop_window_target.set_control_flow(ControlFlow::Wait);

        let mut should_draw = false;

        match event {
            Event::WindowEvent { window_id, event } => match event {
                WindowEvent::ActivationTokenDone { .. } => {}
                WindowEvent::Resized(size) => {
                }
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
                    if _event.state == ElementState::Pressed {
                    }
                }
                WindowEvent::ModifiersChanged(_) => {}
                WindowEvent::Ime(_) => {}
                WindowEvent::CursorMoved { .. } => {
                    rt.block_on(async {
                        tx.send(String::from("Hi")).await;
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
                WindowEvent::RedrawRequested => {
                    should_draw = true;
                }
            },
            Event::Resumed => {
               /* async {
                    create_render_context(event_loop_window_target);
                };*/
            }
            Event::NewEvents(_) => {}
            Event::DeviceEvent { .. } => {}
            Event::UserEvent(_) => {}
            Event::Suspended => {
            }
            Event::AboutToWait => {}
            Event::LoopExiting => {}
            Event::MemoryWarning => {}
    }

    }).unwrap();
}

async fn create_render_context(event_loop: &ActiveEventLoop) -> Option<RenderContext> {
    let window_attributes = Window::default_attributes().with_title("Oku").with_transparent(false);

    let window = Arc::new(event_loop.create_window(window_attributes).expect("Failed to create window"));

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let surface = unsafe { instance.create_surface(window.clone()) }.unwrap();
    let adapter = instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        },
    ).await;

   /* let (device, queue) = adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: wgpu::Label::from("oku_wgpu_renderer"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        },
        None, // Trace path
    ).unwrap();*/


   /* let mut render_context = RenderContext {
        surface: Box::new(()),
        renderer: Box::new(()),
        device: Device::WgpuDevice(),
        window: window,
    };*/

    None
    // Some(render_context)
}