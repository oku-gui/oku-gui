pub mod user;

pub mod engine;
mod options;
pub mod platform;
#[cfg(test)]
mod tests;

pub use options::OkuOptions;

use crate::user::components::component::{
    ComponentId, ComponentSpecification, GenericUserState,
};
use cosmic_text::{FontSystem, SwashCache};
use std::any::Any;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time;
use tokio::sync::mpsc;
use tracing::info;
use user::reactive::element_id::reset_unique_element_id;
use user::reactive::fiber_node::FiberNode;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

use crate::engine::events::update_queue_entry::UpdateQueueEntry;
use crate::platform::runtimes::native::create_native_runtime;
use crate::user::elements::container::Container;
use crate::user::elements::element::Element;
use crate::user::elements::layout_context::{measure_content, LayoutContext};
use crate::user::elements::style::Unit;
use crate::user::reactive::tree::{create_trees_from_render_specification, ComponentTreeNode};
use engine::events::{Message, OkuEvent};
use engine::renderer::color::Color;
use engine::renderer::renderer::Renderer;
use engine::renderer::softbuffer::SoftwareRenderer;
use engine::renderer::wgpu::WgpuRenderer;

const WAIT_TIME: time::Duration = time::Duration::from_millis(100);

use crate::engine::app_message::AppMessage;
use crate::engine::events::{PointerButton, PointerMoved};
pub use crate::options::RendererType;
use crate::platform::resource_manager::ResourceManager;
use crate::user::elements::image::Image;
use engine::events::internal::InternalMessage;
pub use tokio::join;
pub use tokio::spawn;

pub type PinnedFutureAny = Pin<Box<dyn Future<Output = Box<dyn Any + Send>> + Send>>;

struct App {
    app: ComponentSpecification,
    window: Option<Arc<dyn Window>>,
    renderer: Option<Box<dyn Renderer + Send>>,
    renderer_context: Option<RenderContext>,
    element_tree: Option<Box<dyn Element>>,
    component_tree: Option<ComponentTreeNode>,
    mouse_position: (f32, f32),
    update_queue: VecDeque<UpdateQueueEntry>,
    user_state: HashMap<ComponentId, Box<GenericUserState>>,
    resource_manager: ResourceManager,
    winit_sender: mpsc::Sender<AppMessage>
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
    window: Option<Arc<dyn Window>>,
    winit_receiver: mpsc::Receiver<AppMessage>,
    app_sender: mpsc::Sender<AppMessage>,
    oku_options: OkuOptions,
}

pub fn oku_main(application: ComponentSpecification) {
    oku_main_with_options(application, None)
}

pub fn oku_main_with_options(application: ComponentSpecification, options: Option<OkuOptions>) {
    info!("Oku started");

    let rt = create_native_runtime().expect("Failed to creat async runtime.");

    info!("Created async runtime");

    let event_loop = EventLoop::new().expect("Failed to create winit event loop.");

    info!("Created winit event loop");

    let (app_sender, app_receiver) = mpsc::channel::<AppMessage>(100);
    let (winit_sender, winit_receiver) = mpsc::channel::<AppMessage>(100);

    let x = app_sender.clone();
    rt.spawn(async move {
        async_main(application, app_receiver, winit_sender, x).await;
    });

    let mut app = OkuState {
        id: 0,
        rt,
        request_redraw: false,
        wait_cancelled: false,
        close_requested: false,
        window: None,
        winit_receiver,
        app_sender: app_sender.clone(),
        oku_options: options.unwrap_or_default(),
    };

    event_loop.run_app(&mut app).expect("run_app failed");
}

impl ApplicationHandler for OkuState {
    fn new_events(&mut self, _event_loop: &dyn ActiveEventLoop, cause: StartCause) {
        self.wait_cancelled = matches!(cause, StartCause::WaitCancelled { .. })
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let window_attributes = WindowAttributes::default().with_title("oku");
        let window: Arc<dyn Window> =
            Arc::from(event_loop.create_window(window_attributes).expect("Failed to create window."));
        info!("Created window");

        self.window = Some(window.clone());

        info!("Using {} renderer.", self.oku_options.renderer);

        let renderer: Box<dyn Renderer + Send> = match self.oku_options.renderer {
            RendererType::Software => Box::new(SoftwareRenderer::new(window.clone())) as Box<dyn Renderer + Send>,
            RendererType::Wgpu => Box::new(self.rt.block_on(async { WgpuRenderer::new(window.clone()).await })),
        };

        info!("Created renderer");

        self.send_message(InternalMessage::Resume(window, Some(renderer)), true);
    }

    fn window_event(&mut self, _event_loop: &dyn ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.send_message(InternalMessage::Close, true);
                self.close_requested = true;
            }
            WindowEvent::PointerButton {
                device_id,
                state,
                position,
                button,
            } => {
                let event = PointerButton::new(device_id, state, position, button);
                self.send_message(InternalMessage::PointerButton(event), false);
            }
            WindowEvent::PointerMoved {
                device_id,
                position,
                source,
            } => {
                self.send_message(InternalMessage::PointerMoved(PointerMoved::new(device_id, position, source)), true);
            }
            WindowEvent::SurfaceResized(new_size) => {
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

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.request_redraw && !self.wait_cancelled && !self.close_requested {
            //self.window.as_ref().unwrap().request_redraw();
        }

        self.send_message(InternalMessage::ProcessUserEvents, false);

        if !self.wait_cancelled {
            event_loop.set_control_flow(ControlFlow::WaitUntil(time::Instant::now() + WAIT_TIME));
        }

        if self.close_requested {
            info!("Exiting winit event loop");

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

    fn send_message(&mut self, message: InternalMessage, blocking: bool) {
        let app_message = AppMessage {
            id: self.id,
            blocking,
            data: message,
        };
        self.rt.block_on(async {
            self.app_sender.send(app_message).await.expect("send failed");
            if blocking {
                if let Some(response) = self.winit_receiver.recv().await {
                    if let(InternalMessage::Confirmation) = response.data {
                        assert_eq!(response.id, self.id, "Expected response message with id {}", self.id);
                    } else {
                        panic!("Expected response message, but response was something else");
                    }
                } else {
                    panic!("Expected response message, but response was empty");
                }
            }
        });
        self.id += 1;
    }
}

async fn send_response(app_message: AppMessage, sender: &mpsc::Sender<AppMessage>) {
    if app_message.blocking {
        sender.send(AppMessage::new(app_message.id, InternalMessage::Confirmation)).await.expect("send failed");
    }
}

async fn async_main(
    application: ComponentSpecification,
    mut app_receiver: mpsc::Receiver<AppMessage>,
    winit_sender: mpsc::Sender<AppMessage>,
    app_sender: mpsc::Sender<AppMessage>,
) {
    let mut user_state = HashMap::new();

    let dummy_root_value: Box<GenericUserState> = Box::new(());
    user_state.insert(0, dummy_root_value);

    let mut app = Box::new(App {
        app: application,
        window: None,
        renderer: None,
        renderer_context: None,
        element_tree: None,
        component_tree: None,
        mouse_position: (0.0, 0.0),
        update_queue: VecDeque::new(),
        user_state,
        resource_manager: ResourceManager::new(),
        winit_sender: winit_sender.clone(),
    });

    info!("starting main event loop");
    loop {
        if let Some(app_message) = app_receiver.recv().await {
            let mut dummy_message = AppMessage::new(app_message.id, InternalMessage::Confirmation);
            dummy_message.blocking = app_message.blocking;

            match app_message.data {
                InternalMessage::RequestRedraw => {
                    on_request_redraw(&mut app).await;
                    send_response(dummy_message, &app.winit_sender).await;
                }
                InternalMessage::Close => {
                    info!("Closing");
                    send_response(dummy_message, &app.winit_sender).await;
                    break;
                }
                InternalMessage::Confirmation => {}
                InternalMessage::Resume(window, renderer) => {
                    on_resume(&mut app, window.clone(), renderer).await;
                    send_response(dummy_message, &app.winit_sender).await;
                }
                InternalMessage::Resize(new_size) => {
                    on_resize(&mut app, new_size).await;
                    send_response(dummy_message, &app.winit_sender).await;
                }
                InternalMessage::PointerButton(pointer_button) => {
                    on_pointer_button(&mut app, pointer_button).await;
                    send_response(dummy_message, &app.winit_sender).await;
                }
                InternalMessage::PointerMoved(pointer_moved) => {
                    on_pointer_moved(&mut app, pointer_moved.clone()).await;
                    send_response(dummy_message, &app.winit_sender).await;
                }
                InternalMessage::ProcessUserEvents => {
                    on_process_user_events(&mut app, &app_sender);
                }
                InternalMessage::GotUserMessage(message) => {
                    let update_fn = message.0;
                    let source_component = message.1;
                    let source_element = message.2;
                    let message = message.3;
                    let state = app.user_state.get_mut(&source_component).unwrap().as_mut();
                    update_fn(state, source_component, Message::UserMessage(message), source_element);
                    app.window.as_ref().unwrap().request_redraw();
                }
            }
        }
    }
}

fn on_process_user_events(app: &mut Box<App>, app_sender: &mpsc::Sender<AppMessage>,) {
    if app.update_queue.is_empty() {
        return;;
    }

    for event in app.update_queue.drain(..) {
        let app_sender_copy = app_sender.clone();
        tokio::spawn(async move {
            let update_result = event.update_result.result.unwrap();
            let res = update_result.await;
            app_sender_copy.send(AppMessage::new(
                0,
                InternalMessage::GotUserMessage((
                    event.update_function,
                    event.source_component,
                    event.source_element,
                    res,
                )),
            ))
                .await
                .expect("send failed");
        });
    }
}

async fn on_pointer_moved(
    app: &mut Box<App>,
    mouse_moved: PointerMoved,
) {
    app.mouse_position = (mouse_moved.position.x as f32, mouse_moved.position.y as f32);
}

async fn on_resize(
    app: &mut Box<App>,
    new_size: PhysicalSize<u32>,
) {
    let renderer = app.renderer.as_mut().unwrap();
    renderer.resize_surface(new_size.width.max(1) as f32, new_size.height.max(1) as f32);

    // On macOS the window needs to be redrawn manually after resizing
    app.window.as_ref().unwrap().request_redraw();
}

async fn on_pointer_button(
    mut app: &mut Box<App>,
    pointer_button: PointerButton,
) {
    {
        let current_element_tree = if let Some(current_element_tree) = app.element_tree.as_ref() {
            current_element_tree
        } else {
            return;
        };

        //app.component_tree.as_ref().unwrap().print_tree();
        let fiber: FiberNode = FiberNode {
            element: Some(current_element_tree.as_ref()),
            component: Some(app.component_tree.as_ref().unwrap()),
        };

        let mut event_status = EventStatus::BoundsChecking;
        let mut target_component_id: Option<u64> = None;
        let mut target_element_id: Option<String> = None;
        for fiber_node in fiber.level_order_iter().collect::<Vec<FiberNode>>().iter().rev() {
            if let Some(_element) = fiber_node.element {
                let in_bounds = _element.in_bounds(app.mouse_position.0, app.mouse_position.1);
                //println!(
                //    "Fiber Node - Element: {} - {} - in bounds: {}",
                //    _element.name(),
                //    _element.component_id(),
                //    in_bounds
                //);
            }
            if let Some(_component) = fiber_node.component {
                //println!("Fiber Node - Component: {} - {}", _component.tag, _component.id);
            }
            //println!("Event status: {:?}", event_status);
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
            // Do a pre-order traversal of the component tree to find the target component
            let target_component = app
                .component_tree
                .as_ref()
                .unwrap()
                .pre_order_iter()
                .find(|node| node.id == target_component_id)
                .unwrap();
            let mut to_visit = Some(target_component);

            while let Some(node) = to_visit {
                let event = OkuEvent::PointerButtonEvent(pointer_button);

                let state = app.user_state.get_mut(&node.id).unwrap().as_mut();
                let res = (node.update)(state, node.id, Message::OkuMessage(event), target_element_id.clone());
                let propagate = res.propagate;
                if res.result.is_some() {
                    app.update_queue.push_back(UpdateQueueEntry::new(
                        node.id,
                        target_element_id.clone(),
                        node.update,
                        res,
                    ));
                }
                if !propagate {
                    break;
                }

                if node.parent_id.is_none() {
                    to_visit = None;
                } else {
                    let parent_id = node.parent_id.unwrap();
                    to_visit =
                        app.component_tree.as_ref().unwrap().pre_order_iter().find(|node2| node2.id == parent_id);
                }
            }
        }
    }

    app.window.as_ref().unwrap().request_redraw();
}

async fn on_resume(
    app: &mut App,
    window: Arc<dyn Window>,
    renderer: Option<Box<dyn Renderer + Send>>
) {
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
}

fn scan_view_for_resources(element: &dyn Element, component: &ComponentTreeNode, app: &mut App) {
    let fiber: FiberNode = FiberNode {
        element: Some(element),
        component: Some(component),
    };

    for fiber_node in fiber.level_order_iter().collect::<Vec<FiberNode>>().iter().rev() {
        if let Some(element) = fiber_node.element {
            if element.name() == Image::name() {
                let resource_identifier = element.as_any().downcast_ref::<Image>().unwrap().resource_identifier.clone();
                app.resource_manager.add(resource_identifier);
            }
        }
    }
}

async fn on_request_redraw(app: &mut App) {
    let window_element = Container::new().into();
    let old_component_tree = app.component_tree.as_ref();
    let new_tree = create_trees_from_render_specification(
        app.app.clone(),
        window_element,
        old_component_tree,
        &mut app.user_state,
    );

    scan_view_for_resources(new_tree.1.as_ref(), &new_tree.0, app);

    app.component_tree = Some(new_tree.0);
    let mut root = new_tree.1;

    let renderer = app.renderer.as_mut().unwrap();

    renderer.surface_set_clear_color(Color::new_from_rgba_u8(255, 255, 255, 255));
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

    layout(renderer.surface_width(), renderer.surface_height(), app.renderer_context.as_mut().unwrap(), root.as_mut());
    root.draw(renderer, app.renderer_context.as_mut().unwrap());
    app.element_tree = Some(root);

    renderer.submit();
}

fn layout(
    _window_width: f32,
    _window_height: f32,
    render_context: &mut RenderContext,
    root_element: &mut dyn Element,
) {
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
}
