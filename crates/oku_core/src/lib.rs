pub mod user;

#[cfg(test)]
mod tests;
pub mod engine;
mod platform;

use crate::user::components::component::ComponentSpecification;
use user::reactive::fiber_node::FiberNode;
use user::reactive::element_id::reset_unique_element_id;
use cosmic_text::{FontSystem, SwashCache};
use std::sync::Arc;
use std::time;
use tokio::sync::mpsc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceId, ElementState, KeyEvent, MouseButton, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use crate::user::elements::container::Container;
use crate::user::elements::element::Element;
use crate::user::elements::layout_context::{LayoutContext, measure_content};
use crate::user::elements::style::Unit;
use engine::events::{ClickMessage, Message, OkuEvent};
use crate::user::reactive::tree::{ComponentTreeNode, create_trees_from_render_specification};
use engine::renderer::color::Color;
use engine::renderer::renderer::Renderer;
use engine::renderer::softbuffer::SoftwareRenderer;
use engine::renderer::wgpu::WgpuRenderer;
use crate::platform::runtimes::native::create_native_runtime;

const WAIT_TIME: time::Duration = time::Duration::from_millis(100);

struct App {
    app: ComponentSpecification,
    window: Option<Arc<Window>>,
    renderer: Option<Box<dyn Renderer + Send>>,
    renderer_context: Option<RenderContext>,
    element_tree: Option<Box<dyn Element>>,
    component_tree: Option<ComponentTreeNode>,
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

pub fn oku_main_with_options(application: ComponentSpecification, options: Option<OkuOptions>) {
    let rt = create_native_runtime();

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

    fn can_create_surfaces(&mut self, event_loop: &ActiveEventLoop) {
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
                event:
                    KeyEvent {
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

#[derive(Debug)]
enum EventStatus {
    BoundsChecking,
    Propagating,
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

async fn async_main(
    application: ComponentSpecification,
    mut rx: mpsc::Receiver<(u64, bool, InternalMessage)>,
    tx: mpsc::Sender<(u64, InternalMessage)>,
) {
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

                    let window_element = Container::new().into();
                    let old_component_tree = app.component_tree.as_ref();
                    let new_tree =
                        create_trees_from_render_specification(app.app.clone(), window_element, old_component_tree);
                    app.component_tree = Some(new_tree.0);

                    let mut root = new_tree.1;

                    root.style_mut().width = Unit::Percentage(renderer.surface_width());

                    let is_user_root_height_auto = {
                        let root_children = root.children_mut();
                        root_children[0].children()[0].style().height.is_auto()
                    };

                    root.children_mut()[0].style_mut().width = Unit::Px(renderer.surface_width());

                    if is_user_root_height_auto {
                        root.style_mut().height = Unit::Auto;
                    } else {
                        root.style_mut().height = Unit::Px(renderer.surface_height());
                    }

                    root = layout(
                        renderer.surface_width(),
                        renderer.surface_height(),
                        app.renderer_context.as_mut().unwrap(),
                        &mut root,
                    );
                    root.draw(renderer, app.renderer_context.as_mut().unwrap());
                    app.element_tree = Some(root);

                    renderer.submit();
                    send_response(id, wait_for_response, &tx).await;
                }
                InternalMessage::Close => {
                    send_response(id, wait_for_response, &tx).await;
                    break;
                }
                InternalMessage::Confirmation => {}
                InternalMessage::Resume(window, renderer) => {
                    if app.element_tree.is_none() {
                        reset_unique_element_id();
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
                InternalMessage::MouseInput(_mouse_input) => {
                    {
                        println!("Click event");
                        let q = app.element_tree.as_ref().unwrap();
                        app.component_tree.as_ref().unwrap().print_tree();
                        let fiber: FiberNode = FiberNode {
                            element: Some(q.as_ref()),
                            component: Some(app.component_tree.as_ref().unwrap()),
                        };

                        let mut event_status = EventStatus::BoundsChecking;
                        let mut target_component_id: Option<u64> = None;
                        let mut target_element_id: Option<String> = None;
                        for fiber_node in fiber.level_order_iter().collect::<Vec<FiberNode>>().iter().rev() {
                            if let Some(_element) = fiber_node.element {
                                let in_bounds = _element.in_bounds(app.mouse_position.0, app.mouse_position.1);
                                println!(
                                    "Fiber Node - Element: {} - {} - in bounds: {}",
                                    _element.name(),
                                    _element.component_id(),
                                    in_bounds
                                );
                            }
                            if let Some(_component) = fiber_node.component {
                                println!("Fiber Node - Component: {} - {}", _component.tag, _component.id);
                            }
                            println!("Event status: {:?}", event_status);
                            if let Some(element) = fiber_node.element {
                                let in_bounds = element.in_bounds(app.mouse_position.0, app.mouse_position.1);
                                if in_bounds {
                                    target_component_id = Some(element.component_id());
                                    event_status = EventStatus::Propagating;
                                    target_element_id = element.id().clone();
                                    break;
                                }
                            }
                        }
                        if let Some(target_component_id) = target_component_id {
                            println!("Target component id: {}", target_component_id);

                            // Do a pre-order traversal of the component tree to find the target component
                            let target_component = app.component_tree.as_ref().unwrap().pre_order_iter().find(|node| node.id == target_component_id).unwrap();
                            let mut to_visit = Some(target_component);

                            while let Some(node) = to_visit {
                                if let Some(update_fn) = node.update {
                                    let event = OkuEvent::Click(ClickMessage {
                                        mouse_input: MouseInput {
                                            device_id: _mouse_input.device_id,
                                            state: _mouse_input.state,
                                            button: _mouse_input.button,
                                        },
                                        x: app.mouse_position.0 as f64,
                                        y: app.mouse_position.1 as f64,
                                    });
                                    println!("Calling update function");
                                    let res = update_fn(node.id, Message::OkuMessage(event), target_element_id.clone());
                                    if res.0 {
                                        break;
                                    }
                                }

                                if node.parent_id.is_none() {
                                    to_visit = None;
                                } else {
                                    let parent_id = node.parent_id.unwrap();
                                    to_visit = app.component_tree.as_ref().unwrap().pre_order_iter().find(|node2| { 
                                        node2.id == parent_id
                                    });
                                }
                            }
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

fn layout(
    _window_width: f32,
    _window_height: f32,
    render_context: &mut RenderContext,
    root_element: &mut Box<dyn Element>,
) -> Box<dyn Element> {
    let mut taffy_tree: taffy::TaffyTree<LayoutContext> = taffy::TaffyTree::new();
    let root_node = root_element.compute_layout(&mut taffy_tree, &mut render_context.font_system);

    taffy_tree
        .compute_layout_with_measure(
            root_node,
            taffy::Size::max_content(),
            |known_dimensions, available_space, _node_id, node_context, _style| {
                measure_content(known_dimensions, available_space, node_context, &mut render_context.font_system)
            },
        )
        .unwrap();

    root_element.finalize_layout(&mut taffy_tree, root_node, 0.0, 0.0);

    root_element.clone()
}
