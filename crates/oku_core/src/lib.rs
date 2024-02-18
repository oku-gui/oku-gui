pub mod components;
pub mod elements;
pub mod tests;
pub mod widget_id;

use cosmic_text::{FontSystem, SwashCache};
use std::any::Any;
use std::cell::RefCell;

use crate::elements::color::Color;
use crate::elements::container::Container;
use crate::elements::element::Element;
use crate::elements::layout_context::{measure_content, LayoutContext};
use crate::elements::style::{AlignItems, FlexDirection, JustifyContent, Unit};
use crate::elements::text::Text;
use crate::elements::trees::assign_tree_new_ids;
use crate::widget_id::create_unique_widget_id;
use accesskit::{Node, NodeBuilder, NodeClassSet, Role, Tree, TreeUpdate};
use accesskit_winit::{ActionRequestEvent, Adapter};
use softbuffer::{Buffer, Surface};
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tiny_skia::{Pixmap, Rect, Transform};
use winit::event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopWindowTarget};
use winit::keyboard::{Key, NamedKey};
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
use winit::window::{Window, WindowBuilder};

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

pub struct RenderContext {
    font_system: FontSystem,
    swash_cache: SwashCache,
    surface: Surface<Rc<Window>, Rc<Window>>,
    canvas: Pixmap,
    cursor_x: f32,
    cursor_y: f32,
    debug_draw: bool,

    application: Box<dyn Application>,
    element_tree: Option<Element>,
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
        let result = TreeUpdate {
            nodes: vec![],
            tree: Some(tree),
            focus: accesskit::NodeId(0),
        };
        result
    }
}

fn rgb_to_encoded_u32(r: u32, g: u32, b: u32) -> u32 {
    b | (g << 8) | (r << 16)
}

pub fn main(application: Box<dyn Application>) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async_main(application))
}

async fn async_main(application: Box<dyn Application>) {
    let winit_event_loop: EventLoop<ActionRequestEvent> = EventLoopBuilder::with_user_event().build().unwrap();
    let window = Rc::new(WindowBuilder::new().build(&winit_event_loop).unwrap());
    window.set_title("Oku");

    let context = softbuffer::Context::new(window.clone()).unwrap();
    let surface: Surface<Rc<Window>, Rc<Window>> = Surface::new(&context, window.clone()).unwrap();
    let pixmap = Pixmap::new(100, 100).unwrap();

    let app = Rc::new(RefCell::new(RenderContext {
        font_system: FontSystem::new(),
        swash_cache: SwashCache::new(),
        surface,
        canvas: pixmap,

        cursor_x: 0.0,
        cursor_y: 0.0,
        debug_draw: false,
        application,
        element_tree: None,
    }));

    let access_kit_state = State::new();
    let adapter = {
        let access_kit_state = Arc::clone(&access_kit_state);
        Adapter::new(
            &window,
            move || {
                let mut state = access_kit_state.lock().unwrap();
                state.build_initial_tree()
            },
            winit_event_loop.create_proxy(),
        )
    };

    winit_event_loop
        .run(move |event, event_loop_window_target| {
            event_loop(&window, &adapter, app.clone(), event, event_loop_window_target);
        })
        .unwrap();
}

fn event_loop(window: &Window, adapter: &Adapter, app: Rc<RefCell<RenderContext>>, event: Event<ActionRequestEvent>, event_loop_window_target: &EventLoopWindowTarget<ActionRequestEvent>) {
    event_loop_window_target.set_control_flow(ControlFlow::Wait);
    let mut should_draw = false;

    // Create the first tree
    if app.borrow_mut().element_tree.is_none() {
        should_draw = true;
    }

    match event {
        Event::WindowEvent { event, window_id } => {
            let app: &mut RenderContext = &mut app.borrow_mut();
            if window_id != window.id() {
                return;
            }

            adapter.process_event(&window, &event);
            match event {
                WindowEvent::RedrawRequested => {
                    should_draw = true;
                }
                WindowEvent::CloseRequested => {
                    event_loop_window_target.exit();
                }
                WindowEvent::CursorMoved { position, .. } => {
                    app.cursor_x = position.x as f32;
                    app.cursor_y = position.y as f32;
                }
                WindowEvent::KeyboardInput {
                    device_id: _device_id,
                    event: _event,
                    is_synthetic: _is_synthetic,
                } => {
                    if _event.state == ElementState::Pressed {
                        match _event.key_without_modifiers().as_ref() {
                            Key::Named(NamedKey::F3) => {
                                // Toggle Debug Draw
                                app.debug_draw = !app.debug_draw;
                                // Redraw
                                should_draw = true;
                            }
                            _ => {}
                        }
                    }
                }
                WindowEvent::MouseInput {
                    device_id: _device_id,
                    state: _state,
                    button,
                } => {}
                _ => {}
            }

            if should_draw {
                let mut element = app.application.view();

                let mut window_element = Container::new();

                let mut root = app.element_tree.clone().unwrap();

                window_element = window_element.width(Unit::Px(window.inner_size().width as f32));
                //window_element = window_element.height(Unit::Px(window.inner_size().height as f32));
                let computed_style = &root.computed_style_mut();

                // The root element should be 100% window width if the width is not already set.
                if computed_style.width.is_auto() {
                    root.computed_style_mut().width = Unit::Px(window.inner_size().width as f32);
                }

                window_element = window_element.add_child(root);
                let mut window_element = Element::Container(window_element);

                layout(window.inner_size().width as f32, window.inner_size().height as f32, app, &mut window_element);
                draw(window.inner_size().width as f32, window.inner_size().height as f32, app, &mut window_element);

                app.element_tree = Some(window_element);
            }
        }
        _ => {}
    }
}

fn draw(window_width: f32, window_height: f32, render_context: &mut RenderContext, root_element: &mut Element) {
    if window_height == 0.0 || window_width == 0.0 {
        return;
    }

    render_context.surface.resize(NonZeroU32::new(window_width as u32).unwrap(), NonZeroU32::new(window_height as u32).unwrap()).unwrap();
    render_context.canvas = Pixmap::new(window_width as u32, window_height as u32).unwrap();
    render_context.canvas.fill(tiny_skia::Color::from_rgba8(255, 255, 255, 255));

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
