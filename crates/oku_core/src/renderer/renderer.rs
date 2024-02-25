use std::rc::Rc;
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
    window: winit::window::Window,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum ActionRequestEvent {
    WakeUp,
}

pub fn wgpu_integration() {
    env_logger::init();
    let winit_event_loop = EventLoop::<ActionRequestEvent>::with_user_event().build().unwrap();
    let window_attributes = Window::default_attributes().with_title("oku").with_transparent(false);
    let window = Rc::new(winit_event_loop.create_window(window_attributes).expect("Failed to create window"));


    winit_event_loop.run(move |event, event_loop_window_target| {
        event_loop_window_target.set_control_flow(ControlFlow::Wait);
        let mut should_draw = false;
    
        /*// Create the first tree
        if app.borrow_mut().element_tree.is_none() {
            should_draw = true;
        }*/
    
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
                WindowEvent::CursorMoved { .. } => {}
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
                create_window(event_loop_window_target);
            }
            Event::NewEvents(_) => {}
            Event::DeviceEvent { .. } => {}
            Event::UserEvent(_) => {}
            Event::Suspended => {
            }
            Event::AboutToWait => {}
            Event::LoopExiting => {}
            Event::MemoryWarning => {}
        };

    }).unwrap();

}

fn create_window(event_loop: &ActiveEventLoop) -> Rc<Window> {
    let window_attributes = Window::default_attributes().with_title("Oku").with_transparent(false);

    let window = Rc::new(event_loop.create_window(window_attributes).expect("Failed to create window"));
    window
}