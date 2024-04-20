pub mod components;
pub mod elements;

pub mod renderer;
#[cfg(test)]
mod tests;
pub mod widget_id;

use crate::elements::container::Container;
use crate::elements::element::Element;
use crate::elements::layout_context::{measure_content, LayoutContext};
use crate::elements::standard_element::StandardElement;
use crate::elements::style::Unit;
use accesskit::{Node, NodeBuilder, NodeClassSet, Role, Tree, TreeUpdate};
use env_logger;
//use accesskit_winit::{ActionRequestEvent, Adapter};
use cosmic_text::{FontSystem, SwashCache};
use renderer::renderer::wgpu_integration;
use softbuffer::{Buffer, Surface};
use std::any::Any;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tiny_skia::{Pixmap, Rect, Transform};
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopBuilder};
use winit::keyboard::{Key, NamedKey};
#[cfg(not(target_arch = "wasm32"))]
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
use winit::window::{Window, WindowId};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub trait Application {
    fn view(&self) -> Element;
}

pub struct Props {
    pub data: Box<dyn Any>,
}

impl Props {
    pub fn get_data<T: 'static>(&self) -> Option<&T> {
        self.data.downcast_ref::<T>()
    }
}

pub struct OkuContext {
    render_context: Option<RenderContext>,
    application: Box<dyn Application>,
    element_tree: Option<Element>,
    should_draw: bool,
}

pub struct RenderContext {
    font_system: FontSystem,
    swash_cache: SwashCache,
    surface: Surface<Rc<Window>, Rc<Window>>,
    canvas: Pixmap,
    cursor_x: f32,
    cursor_y: f32,
    debug_draw: bool,
    window: Rc<Window>,
}

struct State {
    _focus: taffy::NodeId,
    _announcement: Option<String>,
    node_classes: NodeClassSet,
}

impl State {
    fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            _focus: taffy::NodeId::new(0),
            _announcement: None,
            node_classes: NodeClassSet::new(),
        }))
    }

    fn build_root(&mut self) -> Node {
        let mut builder = NodeBuilder::new(Role::Window);
        builder.set_name("WINDOW_TITLE");
        builder.build(&mut self.node_classes)
    }

    fn build_initial_tree(&mut self) -> TreeUpdate {
        let _root = self.build_root();
        let tree = Tree::new(accesskit::NodeId(0));
        TreeUpdate {
            nodes: vec![],
            tree: Some(tree),
            focus: accesskit::NodeId(0),
        }
    }
}

fn rgb_to_encoded_u32(r: u32, g: u32, b: u32) -> u32 {
    b | (g << 8) | (r << 16)
}
use log::info;
use log::Level;
use winit::application::ApplicationHandler;

pub fn main(application: Box<dyn Application>) {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
            info!("It works!");
        } else {
            env_logger::init();
        }
    }
    wgpu_integration();
    //async_main(application);
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum ActionRequestEvent {
    WakeUp,
}

impl ApplicationHandler<ActionRequestEvent> for OkuContext {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        create_window(self, event_loop);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::ActivationTokenDone { .. } => {}
            WindowEvent::Resized(new_size) => {
                if let Some(render_context) = &mut self.render_context {
                    let width = new_size.width;
                    let height = new_size.height;
                    render_context.surface.resize(NonZeroU32::new(width).unwrap(), NonZeroU32::new(height).unwrap()).unwrap();
                    render_context.canvas = Pixmap::new(width, height).unwrap();
                    self.should_draw = true;
                }
            }
            WindowEvent::Moved(_) => {}
            WindowEvent::CloseRequested => {
                event_loop.exit();
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
                    cfg_if::cfg_if! {
                        if #[cfg(target_arch = "wasm32")] {
                        } else {
                            if let Key::Named(NamedKey::F3) = _event.key_without_modifiers().as_ref() {
                        if let Some(render_context) = &mut self.render_context {
                            render_context.debug_draw = !render_context.debug_draw;
                            self.should_draw = true;
                        }
                    }
                        }
                    }
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
            WindowEvent::RedrawRequested => {}
        }
    }
}

fn async_main(application: Box<dyn Application>) {
    let winit_event_loop = EventLoop::<ActionRequestEvent>::with_user_event().build().unwrap();
    //let window = Rc::new(Window::default_attributes().(&winit_event_loop).unwrap());
    //window.set_title("Oku");

    let access_kit_state = State::new();
    /*let adapter = {
        let access_kit_state = Arc::clone(&access_kit_state);
        Adapter::new(
            &window,
            move || {
                let mut state = access_kit_state.lock().unwrap();
                state.build_initial_tree()
            },
            winit_event_loop.create_proxy(),
        )
    };*/

    let mut oku_context = OkuContext {
        render_context: None,
        application,
        element_tree: None,
        should_draw: false,
    };

    winit_event_loop.run_app(&mut oku_context).unwrap();
}

fn event_loop(app: &mut OkuContext, event: Event<ActionRequestEvent>, event_loop_window_target: &ActiveEventLoop) {
    event_loop_window_target.set_control_flow(ControlFlow::Wait);

    /*// Create the first tree
    if app.borrow_mut().element_tree.is_none() {
        should_draw = true;
    }*/

    if !app.should_draw || app.render_context.is_none() {
        return;
    }

    let render_context = app.render_context.as_mut().unwrap();
    let width = render_context.canvas.width() as f32;
    let height = render_context.canvas.height() as f32;

    let new_view = app.application.view();
    app.element_tree = Some(new_view);

    let mut window_element = Container::new();

    let mut root = app.element_tree.clone().unwrap();

    window_element = window_element.width(Unit::Px(width));
    let computed_style = &root.computed_style_mut();

    // The root element should be 100% window width if the width is not already set.
    if computed_style.width.is_auto() {
        root.computed_style_mut().width = Unit::Px(width);
    }

    window_element = window_element.add_child(root);
    let mut window_element = Element::Container(window_element);

    layout(width, height, render_context, &mut window_element);
    draw(width, height, render_context, &mut window_element);

    app.element_tree = Some(window_element);
}

fn draw(window_width: f32, window_height: f32, render_context: &mut RenderContext, root_element: &mut Element) {
    if window_height == 0.0 || window_width == 0.0 {
        return;
    }

    render_context.canvas.fill(tiny_skia::Color::WHITE);

    root_element.draw(render_context);
    if render_context.debug_draw {
        let mut paint = tiny_skia::Paint::default();
        paint.set_color_rgba8(255, 90, 24, 255);
        render_context.canvas.fill_rect(Rect::from_xywh(window_width / 2.0 - 1.0, 0.0, 2.0, window_height).unwrap(), &paint, Transform::identity(), None);
        root_element.debug_draw(render_context);
    }

    // Fill Framebuffer with pixels from tiny-skia.
    let buffer = copy_skia_buffer_to_softbuffer(window_width, window_height, render_context);

    buffer.present().unwrap();
}

fn layout(_window_width: f32, _window_height: f32, render_context: &mut RenderContext, root_element: &mut Element) {
    let mut taffy_tree: taffy::TaffyTree<LayoutContext> = taffy::TaffyTree::new();
    let root_node = root_element.compute_layout(&mut taffy_tree, &mut render_context.font_system);

    taffy_tree
        .compute_layout_with_measure(root_node, taffy::Size::max_content(), |known_dimensions, available_space, _node_id, node_context| measure_content(known_dimensions, available_space, node_context, &mut render_context.font_system))
        .unwrap();

    root_element.finalize_layout(&mut taffy_tree, root_node, 0.0, 0.0);
}

fn copy_skia_buffer_to_softbuffer(width: f32, height: f32, render_context: &mut RenderContext) -> Buffer<Rc<Window>, Rc<Window>> {
    let mut buffer = render_context.surface.buffer_mut().unwrap();
    for y in 0..height as u32 {
        for x in 0..width as u32 {
            let index = y as usize * width as usize + x as usize;
            let current_pixel = render_context.canvas.pixels()[index];

            let red = current_pixel.red() as u32;
            let green = current_pixel.green() as u32;
            let blue = current_pixel.blue() as u32;

            buffer[index] = rgb_to_encoded_u32(red, green, blue);
        }
    }
    buffer
}

fn create_window(oku_context: &mut OkuContext, event_loop: &ActiveEventLoop) {
    let window_attributes = Window::default_attributes().with_title("Oku").with_transparent(false);

    let window = Rc::new(event_loop.create_window(window_attributes).expect("Failed to create window"));

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        web_sys::window().unwrap().document().unwrap().body().unwrap().append_child(&window.canvas().unwrap()).unwrap();
    }

    info!("width: {}", window.inner_size().width);
    info!("height: {}", window.inner_size().height);
    let context = softbuffer::Context::new(window.clone()).unwrap();
    let surface: Surface<Rc<Window>, Rc<Window>> = Surface::new(&context, window.clone()).unwrap();
    let pixmap = Pixmap::new(100, 100).unwrap();

    oku_context.render_context = Some(RenderContext {
        font_system: FontSystem::new_with_fonts([cosmic_text::fontdb::Source::Binary(Arc::new(include_bytes!("../../../fonts/FiraSans-Regular.ttf").as_slice()))]),
        swash_cache: SwashCache::new(),
        surface,
        canvas: pixmap,
        cursor_x: 0.0,
        cursor_y: 0.0,
        debug_draw: false,
        window: window.clone(),
    });
}
