pub mod application;
pub mod components;
pub mod elements;
//mod lib2;
pub mod renderer;
mod widget_id;

pub mod events;
pub mod reactive;
#[cfg(test)]
mod tests;

use crate::components::component::{ComponentSpecification, ComponentOrElement, ViewFn};
use cosmic_text::{FontSystem, SwashCache};
use slotmap::{DefaultKey, SlotMap};
use std::any::type_name_of_val;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;
use std::time;
use taffy::NodeId;
use tokio::sync::mpsc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceId, ElementState, KeyEvent, MouseButton, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};
use crate::components::props::Props;

use crate::elements::container::Container;
use crate::elements::element::{Element, StandardElementClone};
use crate::elements::layout_context::{measure_content, LayoutContext};
use crate::elements::style::{Style, Unit};
use crate::renderer::color::Color;
use crate::renderer::renderer::Renderer;
use crate::renderer::softbuffer::SoftwareRenderer;
use crate::renderer::wgpu::WgpuRenderer;
const WAIT_TIME: time::Duration = time::Duration::from_millis(100);

struct App {
    app: ComponentSpecification,
    window: Option<Arc<Window>>,
    renderer: Option<Box<dyn Renderer + Send>>,
    renderer_context: Option<RenderContext>,
    element_tree: Option<Box<dyn Element>>,
    component_tree: Option<Box<ComponentTreeNode>>,
    mouse_position: (f32, f32),
}

pub struct RenderContext {
    font_system: FontSystem,
    swash_cache: SwashCache,
}

struct OkuState {
    id: u64,
    rt: tokio::runtime::Runtime,
    request_redraw: bool,
    wait_cancelled: bool,
    close_requested: bool,
    window: Option<Arc<Window>>,
    app_to_winit_rx: mpsc::Receiver<(u64, InternalMessage)>,
    winit_to_app_tx: mpsc::Sender<(u64, bool, InternalMessage)>,
    oku_options: OkuOptions,
}

#[derive(Copy, Clone, Debug)]
struct MouseInput {
    device_id: DeviceId,
    state: ElementState,
    button: MouseButton,
}

#[derive(Copy, Clone, Debug)]
struct MouseMoved {
    device_id: DeviceId,
    position: (f64, f64),
}

enum InternalMessage {
    RequestRedraw,
    Close,
    Confirmation,
    Resume(Arc<Window>, Option<Box<dyn Renderer + Send>>),
    Resize(PhysicalSize<u32>),
    MouseInput(MouseInput),
    MouseMoved(MouseMoved),
}

#[derive(Default)]
pub struct OkuOptions {
    pub renderer: RendererType,
}

#[derive(Default)]
pub enum RendererType {
    Software,
    #[default]
    Wgpu,
}

pub fn oku_main(application: ComponentSpecification) {
    oku_main_with_options(application, None)
}

struct ComponentTreeNode {
    key: Option<String>,
    tag: String,
    children: Vec<ComponentTreeNode>,
    id: u64
}

pub fn oku_main_with_options(application: ComponentSpecification, options: Option<OkuOptions>) {
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Failed to create runtime");

    let event_loop = EventLoop::new().expect("Failed to create winit event loop");

    let (winit_to_app_tx, winit_to_app_rx) = mpsc::channel::<(u64, bool, InternalMessage)>(100);
    let (app_to_winit_tx, app_to_winit_rx) = mpsc::channel::<(u64, InternalMessage)>(100);

    rt.spawn(async move {
        async_main(application, winit_to_app_rx, app_to_winit_tx).await;
    });

    let mut app = OkuState {
        id: 0,
        rt,
        request_redraw: false,
        wait_cancelled: false,
        close_requested: false,
        window: None,
        app_to_winit_rx,
        winit_to_app_tx,
        oku_options: options.unwrap_or_default(),
    };

    event_loop.run_app(&mut app).expect("run_app failed");
}

impl ApplicationHandler for OkuState {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        self.wait_cancelled = matches!(cause, StartCause::WaitCancelled { .. })
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("oku");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());

        let renderer: Box<dyn Renderer + Send> = match self.oku_options.renderer {
            RendererType::Software => Box::new(SoftwareRenderer::new(window.clone())) as Box<dyn Renderer + Send>,
            RendererType::Wgpu => Box::new(self.rt.block_on(async { WgpuRenderer::new(window.clone()).await })),
        };

        self.send_message(InternalMessage::Resume(window, Some(renderer)), true);
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.send_message(InternalMessage::Close, true);
                self.close_requested = true;
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                let mouse_event = MouseInput {
                    device_id,
                    state,
                    button,
                };
                if let MouseButton::Left = button {
                    self.request_redraw = true;

                    if state == ElementState::Pressed {
                        self.send_message(InternalMessage::MouseInput(mouse_event), true);
                    }
                }
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                self.send_message(
                    InternalMessage::MouseMoved(MouseMoved {
                        device_id,
                        position: (position.x, position.y),
                    }),
                    true,
                );
            }
            WindowEvent::Resized(new_size) => {
                self.send_message(InternalMessage::Resize(new_size), true);
            }
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    logical_key: key,
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => {
                if let Key::Named(NamedKey::Escape) = key.as_ref() {
                    self.close_requested = true;
                }
            }
            WindowEvent::RedrawRequested => {
                self.send_message(InternalMessage::RequestRedraw, true);
                self.window.clone().unwrap().pre_present_notify();
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.request_redraw && !self.wait_cancelled && !self.close_requested {
            //self.window.as_ref().unwrap().request_redraw();
        }

        if !self.wait_cancelled {
            event_loop.set_control_flow(ControlFlow::WaitUntil(time::Instant::now() + WAIT_TIME));
        }

        if self.close_requested {
            event_loop.exit();
        }
    }
}

impl OkuState {
    fn send_message(&mut self, message: InternalMessage, wait_for_response: bool) {
        let id = self.id;
        self.rt.block_on(async {
            self.winit_to_app_tx.send((id, wait_for_response, message)).await.expect("send failed");
            if wait_for_response {
                if let Some((id, InternalMessage::Confirmation)) = self.app_to_winit_rx.recv().await {
                    assert_eq!(id, self.id, "Expected response message with id {}", self.id);
                } else {
                    panic!("Expected response message");
                }
            }
        });
        self.id += 1;
    }
}

async fn send_response(id: u64, wait_for_response: bool, tx: &mpsc::Sender<(u64, InternalMessage)>) {
    if wait_for_response {
        tx.send((id, InternalMessage::Confirmation)).await.expect("send failed");
    }
}
use crate::events::ClickMessage;
use crate::reactive::reactive::RUNTIME;
use crate::widget_id::{create_unique_widget_id, reset_unique_widget_id};

struct UnsafeElement {
    element: *mut dyn Element,
}

#[derive(Clone)]
struct TreeVisitorNode {
    component_specification: Rc<RefCell<ComponentSpecification>>,
    parent: *mut dyn Element,
    parent_component_node: *mut ComponentTreeNode,
    old_component_node: Option<*mut ComponentTreeNode>,
}

impl ComponentTreeNode {
    pub fn print_tree(&self) {
        unsafe {
            let mut elements: Vec<(*const ComponentTreeNode, usize, bool)> = vec![(self, 0, true)];
            while let Some((element, indent, is_last)) = elements.pop() {
                let mut prefix = String::new();
                for _ in 0..indent {
                    prefix.push_str("  ");
                }
                if is_last {
                    prefix.push_str("└─");
                } else {
                    prefix.push_str("├─");
                }
                println!("{} , Tag: {}, Id: {}", prefix, (*element).tag, (*element).id);
                let children = &(*element).children;
                for (i, child) in children.iter().enumerate().rev() {
                    let is_last = i == children.len() - 1;
                    elements.push((child, indent + 1, is_last));
                }
            }
        }
    }
}

// This function constructs the render tree from the component specification.
// The function is safe despite using multiple shared mutable references, because the references are only used to traverse the tree.
fn construct_render_tree_from_user_tree(component_specification: ComponentSpecification, root: &mut Box<dyn Element>, old_component_tree: Option<&Box<ComponentTreeNode>>) -> ComponentTreeNode {
    unsafe {

        let mut component_tree = ComponentTreeNode {
            key: None,
            tag: "root".to_string(),
            children: vec![],
            id: 0
        };

        println!("-----------------------------------");
        let old_component_tree_as_ptr = old_component_tree.map(|old_root| old_root.as_ref() as *const ComponentTreeNode as *mut ComponentTreeNode);

        if let Some(comp) = old_component_tree_as_ptr {
            println!("Old component tree is Some");
            (*comp).print_tree();
        } else {
            println!("Old component tree is None");
        }
        
        let component_root: *mut ComponentTreeNode = &mut component_tree as *mut ComponentTreeNode;
        // A component can output only 1 subtree, but the subtree may have an unknown amount of variants.
        // The subtree variant is determined by the state, much like a function. f(s) = ... where f(s) = Subtree produced and s = State
        // The algorithm is as follows:
        // 1. Determine if the currently visited child is an element or a component.
        // 2. If the child is an element: Add the children of the element to the list of elements to visit.
        // 3. If the child is a component: Produce the subtree with the inputted state and add the parent of the subtree to the to visit list.
        
        let root_spec = ComponentSpecification {
            component: ComponentOrElement::Element(root.clone()),
            key: None,
            props: None,
            children: vec![component_specification],
        };
        let mut to_visit: Vec<TreeVisitorNode> = vec![TreeVisitorNode {
            component_specification: Rc::new(RefCell::new(root_spec)),
            parent: root.as_mut(),
            parent_component_node: component_root,
            old_component_node: old_component_tree_as_ptr,
        }];

        while let Some(tree_node) = to_visit.pop() {
            let key = tree_node.component_specification.borrow().key.clone();
            let children = tree_node.component_specification.borrow().children.clone();
            let props = tree_node.component_specification.borrow().props.clone();

            let has_previous_node = tree_node.old_component_node.is_some();

            match &mut tree_node.component_specification.borrow_mut().component {
                ComponentOrElement::Element(element) => {
                    let mut element = element.clone();

                    let element_ptr = &mut *element as *mut dyn Element;
                    tree_node.parent.as_mut().unwrap().children_mut().push(element);

                    let mut olds: Vec<*mut ComponentTreeNode> = vec![];
                    if has_previous_node {
                        for child in (*tree_node.old_component_node.unwrap()).children.iter_mut() {
                            olds.push(child as *mut ComponentTreeNode);
                        }
                    }

                    let mut news: Vec<TreeVisitorNode> = vec![];
                    let mut component_index: i64 = -1;
                    for (index, child) in children.into_iter().enumerate() {
                        let old_node = if matches!(child.component, ComponentOrElement::ComponentSpec(_, _, _)) {
                            component_index += 1;
                            olds.get(component_index as usize).copied()
                        } else {
                            tree_node.old_component_node
                        };

                        news.push(TreeVisitorNode {
                            component_specification: Rc::new(RefCell::new(child)),
                            parent: element_ptr,
                            parent_component_node: tree_node.parent_component_node,
                            old_component_node: old_node,
                        });
                    }
                    to_visit.extend(news.into_iter().rev());
                }
                ComponentOrElement::ComponentSpec(component_spec, component_tag, type_id) => {

                    // Find the old root node if it exists and use its id if the tag matches the current tag.
                    let old_node_tag: Option<String> = if has_previous_node {
                        Some((*tree_node.old_component_node.unwrap()).tag.clone())
                    } else {
                        None
                    };
                    println!("Diffing: Old: {:?}, New: {}", old_node_tag, *component_tag);
                    let id: u64 = if old_node_tag.is_some() && *component_tag == old_node_tag.unwrap() {
                        (*tree_node.old_component_node.unwrap()).id
                    } else {
                        create_unique_widget_id()
                    };

                    let new_component_node = ComponentTreeNode {
                        key,
                        tag: component_tag.clone(),
                        children: vec![],
                        id,
                    };

                    tree_node.parent_component_node.as_mut().unwrap().children.push(new_component_node);
                    let new_component_pointer: *mut ComponentTreeNode = (*tree_node.parent_component_node).children.last_mut().unwrap();

                    let next_component_spec = Rc::new(RefCell::new(component_spec(props, children, id)));
                    to_visit.push(TreeVisitorNode {
                        component_specification: next_component_spec,
                        parent: tree_node.parent,
                        parent_component_node: new_component_pointer,
                        old_component_node: tree_node.old_component_node,
                    });
                }
            };
        }
        println!("Component tree:");
        component_tree.print_tree();
        component_tree
    }
}

async fn async_main(application: ComponentSpecification, mut rx: mpsc::Receiver<(u64, bool, InternalMessage)>, tx: mpsc::Sender<(u64, InternalMessage)>) {
    let mut app = Box::new(App {
        app: application,
        window: None,
        renderer: None,
        renderer_context: None,
        element_tree: None,
        component_tree: None,
        mouse_position: (0.0, 0.0),
    });

    loop {
        if let Some((id, wait_for_response, msg)) = rx.recv().await {
            match msg {
                InternalMessage::RequestRedraw => {
                    let renderer = app.renderer.as_mut().unwrap();

                    renderer.surface_set_clear_color(Color::new_from_rgba_u8(255, 255, 255, 255));

                    let window_element = Container::new().background(Color::new_from_rgba_u8(0, 0, 255, 255));
                    let mut window_element: Box<dyn Element> = window_element.width(Unit::Px(renderer.surface_width())).into();

                    let old_component_tree = app.component_tree.as_ref();
                    app.component_tree = Some(
                        Box::new(construct_render_tree_from_user_tree(app.app.clone(), &mut window_element, old_component_tree))
                    );

                    let mut root = window_element;

                    let computed_style = root.computed_style_mut();

                    // The root element should be 100% window width if the width is not already set.
                    if computed_style.width.is_auto() {
                        root.computed_style_mut().width = Unit::Px(renderer.surface_width());
                    }

                    let mut window_element: Box<dyn Element> = root;
                    window_element = layout(renderer.surface_width(), renderer.surface_height(), app.renderer_context.as_mut().unwrap(), &mut window_element);

                    window_element.draw(renderer, app.renderer_context.as_mut().unwrap());

                    app.element_tree = Some(window_element);

                    renderer.submit();
                    app.element_tree.as_ref().unwrap().clone_box().print_tree();
                    send_response(id, wait_for_response, &tx).await;
                }
                InternalMessage::Close => {
                    send_response(id, wait_for_response, &tx).await;
                    break;
                }
                InternalMessage::Confirmation => {}
                InternalMessage::Resume(window, renderer) => {
                    if app.element_tree.is_none() {
                        reset_unique_widget_id();
                        //let new_view = app.app.view();
                        //app.element_tree = Some(new_view);
                    }

                    if app.renderer_context.is_none() {
                        let font_system = FontSystem::new();
                        let swash_cache = SwashCache::new();
                        let renderer_context = RenderContext {
                            font_system,
                            swash_cache,
                        };
                        app.renderer_context = Some(renderer_context);
                    }

                    app.window = Some(window.clone());
                    app.renderer = renderer;

                    send_response(id, wait_for_response, &tx).await;
                }
                InternalMessage::Resize(new_size) => {
                    let renderer = app.renderer.as_mut().unwrap();
                    renderer.resize_surface(new_size.width.max(1) as f32, new_size.height.max(1) as f32);

                    // On macOS the window needs to be redrawn manually after resizing
                    app.window.as_ref().unwrap().request_redraw();

                    send_response(id, wait_for_response, &tx).await;
                }
                InternalMessage::MouseInput(mouse_input) => {
                    let root = app.element_tree.clone();
                    let mut to_visit = Vec::<Box<dyn Element>>::new();
                    let mut traversal_history = Vec::<Box<dyn Element>>::new();
                    to_visit.push(root.clone().unwrap());
                    traversal_history.push(root.unwrap());

                    while let Some(element) = to_visit.pop() {
                        for child in element.children() {
                            to_visit.push(child.clone());
                            traversal_history.push(child.clone());
                        }
                    }

                    let old_state = RUNTIME.get_state(1).unwrap_or(0u32);
                    RUNTIME.set_state(1, old_state + 1u32);
                    
                    for element in traversal_history.iter().rev() {
                        let in_bounds = element.in_bounds(app.mouse_position.0, app.mouse_position.1);
                        if !in_bounds {
                            continue;
                        }
                    }

                    app.window.as_ref().unwrap().request_redraw();
                    send_response(id, wait_for_response, &tx).await;
                }
                InternalMessage::MouseMoved(mouse_moved) => {
                    app.mouse_position = (mouse_moved.position.0 as f32, mouse_moved.position.1 as f32);
                    send_response(id, wait_for_response, &tx).await;
                }
            }
        }
    }
}

fn layout(_window_width: f32, _window_height: f32, render_context: &mut RenderContext, root_element: &mut Box<dyn Element>) -> Box<dyn Element> {
    let mut taffy_tree: taffy::TaffyTree<LayoutContext> = taffy::TaffyTree::new();
    let root_node = root_element.compute_layout(&mut taffy_tree, &mut render_context.font_system);

    taffy_tree
        .compute_layout_with_measure(root_node, taffy::Size::max_content(), |known_dimensions, available_space, _node_id, node_context, style| {
            measure_content(known_dimensions, available_space, node_context, &mut render_context.font_system)
        })
        .unwrap();

    root_element.finalize_layout(&mut taffy_tree, root_node, 0.0, 0.0);

    root_element.clone()
}
