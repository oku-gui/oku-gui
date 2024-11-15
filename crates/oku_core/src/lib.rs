pub mod accessibility;
pub mod components;
pub mod elements;
pub mod engine;
pub mod oku_runtime;
mod oku_winit_state;
mod options;
pub mod platform;
pub mod reactive;
pub mod style;
#[cfg(test)]
mod tests;

pub use oku_runtime::OkuRuntime;

use crate::engine::events::update_queue_entry::UpdateQueueEntry;
use crate::style::{Display, Unit, Wrap};
use elements::container::Container;
use elements::element::Element;
use elements::layout_context::{measure_content, LayoutContext};
use engine::events::{Message};
use engine::renderer::color::Color;
use engine::renderer::renderer::Renderer;
use reactive::tree::{diff_trees, ComponentTreeNode};

use futures::channel::mpsc::channel;
use futures::channel::mpsc::Receiver;
use futures::channel::mpsc::Sender;

pub use options::OkuOptions;

#[cfg(target_arch = "wasm32")]
use {log::info, std::cell::RefCell, web_time as time};

#[cfg(target_arch = "wasm32")]
thread_local! {
    pub static MESSAGE_QUEUE: RefCell<Vec<Message>> = RefCell::new(Vec::new());
}

#[cfg(all(feature = "android", target_os = "android"))]
pub use winit::platform::android::activity::*;

type RendererBox = dyn Renderer;

#[cfg(not(target_arch = "wasm32"))]
use std::time;

use cfg_if::cfg_if;
use components::component::{ComponentId, ComponentSpecification};
use cosmic_text::FontSystem;
use futures::{SinkExt, StreamExt};
use std::any::Any;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use taffy::{AvailableSpace, NodeId, TaffyTree};
use tokio::sync::{RwLock, RwLockReadGuard};

#[cfg(not(target_arch = "wasm32"))]
use tracing::info;

use reactive::element_id::reset_unique_element_id;
use reactive::fiber_node::FiberNode;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::Window;

const WAIT_TIME: time::Duration = time::Duration::from_millis(100);

use crate::engine::app_message::AppMessage;
use crate::engine::events::resource_event::ResourceEvent;
use crate::engine::events::{Event, KeyboardInput, MouseWheel, OkuMessage, PointerButton, PointerMoved};
pub use crate::options::RendererType;
use crate::platform::resource_manager::ResourceManager;
use elements::image::Image;
use engine::events::internal::InternalMessage;

#[cfg(target_os = "android")]
use {
    winit::platform::android::EventLoopBuilderExtAndroid,
    winit::event_loop::EventLoopBuilder
};

pub type PinnedFutureAny = Pin<Box<dyn Future<Output = Box<dyn Any + Send>> + Send>>;

struct App {
    app: ComponentSpecification,
    window: Option<Arc<dyn Window>>,
    font_system: Option<FontSystem>,
    renderer: Option<Box<dyn Renderer + Send>>,
    element_tree: Option<Box<dyn Element>>,
    component_tree: Option<ComponentTreeNode>,
    mouse_position: (f32, f32),
    update_queue: VecDeque<UpdateQueueEntry>,
    user_state: StateStore,
    element_state: StateStore,
    resource_manager: Arc<RwLock<ResourceManager>>,
    winit_sender: Sender<AppMessage>,
}

#[cfg(target_os = "android")]
pub fn oku_main_with_options(application: ComponentSpecification, options: Option<OkuOptions>, app: AndroidApp) {
    #[cfg(target_arch = "wasm32")]
    oku_wasm_init();

    info!("Oku started");

    info!("Created winit event loop");

    let event_loop =
        EventLoopBuilder::default().with_android_app(app).build().expect("Failed to create winit event loop.");
    oku_main_with_options_2(event_loop, application, options)
}

#[cfg(target_arch = "wasm32")]
pub fn oku_wasm_init() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init().expect("could not initialize logger");
}

use crate::reactive::state_store::{StateStore, StateStoreItem};
use oku_winit_state::OkuWinitState;

#[cfg(not(target_os = "android"))]
pub fn oku_main_with_options(application: ComponentSpecification, options: Option<OkuOptions>) {
    #[cfg(target_arch = "wasm32")]
    oku_wasm_init();

    info!("Oku started");

    info!("Creating winit event loop.");

    let event_loop = EventLoop::new().expect("Failed to create winit event loop.");
    info!("Created winit event loop.");

    oku_main_with_options_2(event_loop, application, options)
}

fn oku_main_with_options_2(
    event_loop: EventLoop,
    application: ComponentSpecification,
    oku_options: Option<OkuOptions>,
) {
    let oku_options = oku_options.unwrap_or_default();

    let runtime = OkuRuntime::new();
    info!("Created async runtime");

    let (app_sender, app_receiver) = channel::<AppMessage>(100);
    let (winit_sender, winit_receiver) = channel::<AppMessage>(100);
    let resource_manager = Arc::new(RwLock::new(ResourceManager::new(app_sender.clone())));

    let app_sender_copy = app_sender.clone();
    let resource_manager_copy = resource_manager.clone();

    let future = async move {
        async_main(application, app_receiver, winit_sender, app_sender_copy, resource_manager_copy).await;
    };

    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
           runtime.runtime_spawn(future);
        } else {
            runtime.runtime_spawn(future);
        }
    }

    let mut app = OkuWinitState::new(runtime, winit_receiver, app_sender, oku_options);

    event_loop.run_app(&mut app).expect("run_app failed");
}

async fn send_response(app_message: AppMessage, sender: &mut Sender<AppMessage>) {
    #[cfg(not(target_arch = "wasm32"))]
    if app_message.blocking {
        sender.send(AppMessage::new(app_message.id, InternalMessage::Confirmation)).await.expect("send failed");
    }
}

async fn async_main(
    application: ComponentSpecification,
    mut app_receiver: Receiver<AppMessage>,
    winit_sender: Sender<AppMessage>,
    mut app_sender: Sender<AppMessage>,
    resource_manager: Arc<RwLock<ResourceManager>>,
) {
    let mut user_state = StateStore::default();

    let dummy_root_value: Box<StateStoreItem> = Box::new(());
    user_state.storage.insert(0, dummy_root_value);

    let mut app = Box::new(App {
        app: application,
        window: None,
        font_system: None,
        renderer: None,
        element_tree: None,
        component_tree: None,
        mouse_position: (0.0, 0.0),
        update_queue: VecDeque::new(),
        user_state,
        element_state: Default::default(),
        resource_manager,
        winit_sender: winit_sender.clone(),
    });

    info!("starting main event loop");
    loop {
        if let Some(app_message) = app_receiver.next().await {
            let mut dummy_message = AppMessage::new(app_message.id, InternalMessage::Confirmation);
            dummy_message.blocking = app_message.blocking;

            match app_message.data {
                InternalMessage::RequestRedraw => {
                    on_request_redraw(&mut app).await;
                    send_response(dummy_message, &mut app.winit_sender).await;
                }
                InternalMessage::Close => {
                    info!("Closing");
                    send_response(dummy_message, &mut app.winit_sender).await;
                    break;
                }
                InternalMessage::Confirmation => {}
                InternalMessage::Resume(window, renderer) => {
                    on_resume(&mut app, window.clone(), renderer).await;
                    send_response(dummy_message, &mut app.winit_sender).await;
                }
                InternalMessage::Resize(new_size) => {
                    on_resize(&mut app, new_size).await;
                    send_response(dummy_message, &mut app.winit_sender).await;
                }
                InternalMessage::MouseWheel(mouse_wheel) => {
                    on_mouse_wheel(&mut app, mouse_wheel).await;
                    send_response(dummy_message, &mut app.winit_sender).await;
                }
                InternalMessage::PointerButton(pointer_button) => {
                    on_pointer_button(&mut app, pointer_button).await;
                    send_response(dummy_message, &mut app.winit_sender).await;
                }
                InternalMessage::PointerMoved(pointer_moved) => {
                    on_pointer_moved(&mut app, pointer_moved.clone()).await;
                    send_response(dummy_message, &mut app.winit_sender).await;
                }
                InternalMessage::ProcessUserEvents => {
                    on_process_user_events(&mut app, &mut app_sender);
                }
                InternalMessage::GotUserMessage(message) => {
                    let update_fn = message.0;
                    let source_component = message.1;
                    let props = message.3;
                    let message = message.2;

                    let state = app.user_state.storage.get_mut(&source_component).unwrap().as_mut();
                    update_fn(state, props, Event::new(Message::UserMessage(message)));
                    app.window.as_ref().unwrap().request_redraw();
                }
                InternalMessage::ResourceEvent(resource_event) => {
                    match resource_event {
                        ResourceEvent::Added(_) => {
                            // println!("Added resource event");
                        }
                        ResourceEvent::Loaded(_) => {}
                        ResourceEvent::UnLoaded(_) => {}
                    }
                }
                InternalMessage::KeyboardInput(keyboard_input) => {
                    on_keyboard_input(&mut app, keyboard_input).await;
                    send_response(dummy_message, &mut app.winit_sender).await;
                }
            }
        }
    }
}

fn on_process_user_events(app: &mut Box<App>, app_sender: &mut Sender<AppMessage>) {
    if app.update_queue.is_empty() {
        return;
    }

    for event in app.update_queue.drain(..) {
        let mut app_sender_copy = app_sender.clone();
        let window_clone = app.window.clone().unwrap();
        let f = async move {
            let update_result = event.update_result.future.unwrap();
            let res = update_result.await;
            app_sender_copy
                .send(AppMessage::new(
                    0,
                    InternalMessage::GotUserMessage((
                        event.update_function,
                        event.source_component,
                        res,
                        event.props,
                    )),
                ))
                .await
                .expect("send failed");
            window_clone.request_redraw();
        };
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(f);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            tokio::spawn(f);
        }
    }
}

async fn on_pointer_moved(app: &mut Box<App>, mouse_moved: PointerMoved) {
    app.mouse_position = (mouse_moved.position.x as f32, mouse_moved.position.y as f32);
}

async fn on_mouse_wheel(app: &mut Box<App>, mouse_wheel: MouseWheel) {
    let event = OkuMessage::MouseWheelEvent(mouse_wheel);

    dispatch_event(app, event).await;

    app.window.as_ref().unwrap().request_redraw();
}

async fn on_keyboard_input(app: &mut Box<App>, keyboard_input: KeyboardInput) {
    let event = OkuMessage::KeyboardInputEvent(keyboard_input);

    dispatch_event(app, event).await;

    app.window.as_ref().unwrap().request_redraw();
}

async fn on_resize(app: &mut Box<App>, new_size: PhysicalSize<u32>) {
    if let Some(renderer) = app.renderer.as_mut() {
        renderer.resize_surface(new_size.width.max(1) as f32, new_size.height.max(1) as f32);
    }

    // On macOS the window needs to be redrawn manually after resizing
    #[cfg(target_os = "macos")]
    {
        app.window.as_ref().unwrap().request_redraw();
    }
}

async fn dispatch_event(app: &mut Box<App>, event: OkuMessage) {
    let current_element_tree = if let Some(current_element_tree) = app.element_tree.as_ref() {
        current_element_tree
    } else {
        return;
    };

    let fiber: FiberNode = FiberNode {
        element: Some(current_element_tree.as_ref()),
        component: Some(app.component_tree.as_ref().unwrap()),
    };

    let mut targets: VecDeque<(ComponentId, Option<String>)> = VecDeque::new();
    let mut target_components: VecDeque<&ComponentTreeNode> = VecDeque::new();

    /////////////////////////////////////////
    // A,0                                 //
    //   /////////////////////////         //
    //   // B,1                 //         //
    //   //   ///////////       //         //
    //   //   //       //       //         //
    //   //   //  C,2  //       //         //
    //   //   //       //       //         //
    //   //   ///////////       //         //
    //   //                     //         //
    //   /////////////////////////         //
    //                                     //
    /////////////////////////////////////////

    // Collect all possible target elements in reverse order.
    // Nodes added last are usually on top, so this elements in visual order.
    for fiber_node in fiber.level_order_iter().collect::<Vec<FiberNode>>().iter().rev() {
        if let Some(element) = fiber_node.element {
            let in_bounds = element.in_bounds(app.mouse_position.0, app.mouse_position.1);
            if in_bounds {
                targets.push_back((element.component_id(), element.get_id().clone()))
            }
        }
    }

    //println!("Targets: {:?}", targets);

    // The targets should be [(2, Some(c)), (1, Some(b)), (0, Some(a))].

    if targets.is_empty() {
        return;
    }

    // The target is always the first node (2, Some(c)).

    let target = targets[0].clone();
    let (_target_component_id, target_element_id) = target.clone();
    
    let mut propagate = true;
    let mut prevent_defaults = false;
    for current_target in targets.iter() {
        if !propagate {
            break;
        }

        //println!("Dispatching to: {:?}", target);
        let (current_target_component_id, current_target_element_id) = current_target.clone();

        // Get the element's component tree ndoe.
        let current_target_component =
            app.component_tree.as_ref().unwrap().pre_order_iter().find(|node| node.id == current_target_component_id).unwrap();

        // Search for the closest non-element ancestor.
        let mut closest_ancestor_component: Option<&ComponentTreeNode> = None;

        let mut to_visit = Some(current_target_component);
        while let Some(node) = to_visit {
            if !to_visit.unwrap().is_element {
                closest_ancestor_component = Some(node);
                to_visit = None;
            } else {
                if node.parent_id.is_none() {
                    to_visit = None;
                } else {
                    let parent_id = node.parent_id.unwrap();
                    to_visit =
                        app.component_tree.as_ref().unwrap().pre_order_iter().find(|node2| node2.id == parent_id);
                }
            }
        }

        // Dispatch the event to the element's component.
        if let Some(node) = closest_ancestor_component {
            target_components.push_back(node);

            let state = app.user_state.storage.get_mut(&node.id).unwrap().as_mut();
            let res = (node.update)(
                state,
                node.props.clone(),
                Event::new(Message::OkuMessage(event.clone()))
                    .current_target(current_target_element_id.clone())
                    .target(target_element_id.clone())
            );
            propagate = propagate && res.propagate;
            prevent_defaults = prevent_defaults || res.prevent_defaults;
            if res.future.is_some() {
                app.update_queue.push_back(UpdateQueueEntry::new(
                    node.id,
                    node.update,
                    res,
                    node.props.clone(),
                ));
            }
        }
    }

    let mut element_events: VecDeque<(OkuMessage, Option<String>)> = VecDeque::new();

    // Handle element events if prevent defaults was not set to true.
    if !prevent_defaults {
        for target in targets.iter() {
            let (target_component_id, _target_element_id) = target.clone();
            let fiber: FiberNode = FiberNode {
                element: app.element_tree.as_ref().map(|f| f.as_ref()),
                component: Some(app.component_tree.as_ref().unwrap()),
            };

            let mut propagate = true;
            let mut prevent_defaults = false;

            let _resource_manager = &mut app.resource_manager;
            for fiber_node in fiber.pre_order_iter().collect::<Vec<FiberNode>>().iter().rev() {
                if !propagate {
                    break;
                }

                if let Some(element) = fiber_node.element {
                    if element.component_id() == target_component_id {
                        let res =
                            element.on_event(event.clone(), &mut app.element_state, app.font_system.as_mut().unwrap());

                        if let Some(result_message) = res.result_message {
                            element_events.push_back((result_message, element.get_id().clone()));
                        }

                        propagate = propagate && res.propagate;
                        prevent_defaults = prevent_defaults || res.prevent_defaults;
                    }
                }
            }
        }
    }

    for (event, target_element_id) in element_events.iter() {
        let mut propagate = true;
        let mut prevent_defaults = false;
        for node in target_components.iter() {
            if !propagate {
                break;
            }

            let state = app.user_state.storage.get_mut(&node.id).unwrap().as_mut();
            let res = (node.update)(
                state,
                node.props.clone(),
                Event::new(Message::OkuMessage(event.clone())).current_target(target_element_id.clone())
            );
            propagate = propagate && res.propagate;
            prevent_defaults = prevent_defaults || res.prevent_defaults;
            if res.future.is_some() {
                app.update_queue.push_back(UpdateQueueEntry::new(
                    node.id,
                    node.update,
                    res,
                    node.props.clone(),
                ));
            }
        }
    }
}

async fn on_pointer_button(app: &mut Box<App>, pointer_button: PointerButton) {
    let event = OkuMessage::PointerButtonEvent(pointer_button);

    dispatch_event(app, event).await;

    app.window.as_ref().unwrap().request_redraw();
}

async fn on_resume(app: &mut App, window: Arc<dyn Window>, renderer: Option<Box<dyn Renderer + Send>>) {
    if app.element_tree.is_none() {
        reset_unique_element_id();
        //let new_view = app.app.view();
        //app.element_tree = Some(new_view);
    }

    if app.font_system.is_none() {
        let mut font_system = FontSystem::new();

        font_system.db_mut().load_font_data(include_bytes!("../../../fonts/FiraSans-Regular.ttf").to_vec());
        font_system.db_mut().load_font_data(include_bytes!("../../../fonts/FiraSans-Bold.ttf").to_vec());
        font_system.db_mut().load_font_data(include_bytes!("../../../fonts/FiraSans-Italic.ttf").to_vec());

        #[cfg(target_os = "android")]
        {
            font_system.db_mut().load_fonts_dir("/system/fonts");
            font_system.db_mut().set_sans_serif_family("Roboto");
            font_system.db_mut().set_serif_family("Noto Serif");
            font_system.db_mut().set_monospace_family("Droid Sans Mono"); // Cutive Mono looks more printer-like
            font_system.db_mut().set_cursive_family("Dancing Script");
            font_system.db_mut().set_fantasy_family("Dancing Script");
        }

        app.font_system = Some(font_system);
    }
    if renderer.is_some() {
        app.renderer = renderer;

        // We can't guarantee the order of events on wasm.
        // This ensures a resize is not missed if the renderer was not finished creating when resize is called.
        #[cfg(target_arch = "wasm32")]
        app.renderer
            .as_mut()
            .unwrap()
            .resize_surface(window.surface_size().width as f32, window.surface_size().height as f32);
    }

    app.window = Some(window.clone());
}

// Scans through the component tree and diffs it for resources that need to be updated.
async fn scan_view_for_resources(element: &dyn Element, component: &ComponentTreeNode, app: &mut App) {
    let fiber: FiberNode = FiberNode {
        element: Some(element),
        component: Some(component),
    };

    let resource_manager = &mut app.resource_manager;
    for fiber_node in fiber.level_order_iter().collect::<Vec<FiberNode>>().iter().rev() {
        if let Some(element) = fiber_node.element {
            if element.name() == Image::name() {
                let resource_identifier = element.as_any().downcast_ref::<Image>().unwrap().resource_identifier.clone();
                resource_manager.write().await.add(resource_identifier).await;
            }
        }
    }
}

async fn on_request_redraw(app: &mut App) {
    //let total_time_start = Instant::now();
    let window_element = Container::new().into();
    let old_component_tree = app.component_tree.as_ref();

    let new_tree = diff_trees(
        app.app.clone(),
        window_element,
        old_component_tree,
        &mut app.user_state,
        &mut app.element_state,
    );

    scan_view_for_resources(new_tree.1.internal.as_ref(), &new_tree.0, app).await;

    app.component_tree = Some(new_tree.0);
    let mut root = new_tree.1;
    let surface_width: f32;
    let surface_height: f32;

    if app.renderer.is_none() {
        return;
    }

    {
        let renderer = app.renderer.as_mut().unwrap();
        renderer.surface_set_clear_color(Color::rgba(255, 255, 255, 255));
        surface_width = renderer.surface_width();
        surface_height = renderer.surface_height();
    }

    root.internal.style_mut().width = Unit::Px(surface_width);
    root.internal.style_mut().wrap = Wrap::Wrap;
    root.internal.style_mut().display = Display::Block;

    let is_user_root_height_auto = {
        let root_children = root.internal.children_mut();
        root_children[0].internal.children()[0].style().height.is_auto()
    };

    root.internal.children_mut()[0].internal.style_mut().width = Unit::Px(surface_width);
    root.internal.children_mut()[0].internal.style_mut().wrap = Wrap::Wrap;
    root.internal.children_mut()[0].internal.style_mut().display = Display::Block;

    if is_user_root_height_auto {
        root.internal.style_mut().height = Unit::Auto;
    } else {
        root.internal.style_mut().height = Unit::Px(surface_height);
        root.internal.children_mut()[0].internal.style_mut().height = Unit::Px(surface_height);
    }

    //let layout_start = Instant::now(); // Start measuring time
    let resource_manager = app.resource_manager.read().await;

    let element_state = &mut app.element_state;

    // root.print_tree();

    let (mut taffy_tree, taffy_root) = layout(
        element_state,
        surface_width,
        surface_height,
        app.font_system.as_mut().unwrap(),
        root.internal.as_mut(),
        &resource_manager,
    );
    //let duration = layout_start.elapsed(); // Get the elapsed time
    //println!("Layout Time Taken: {:?} ms -------------------------------------------------------------------------------------------------------------", duration.as_millis());

    {
        let renderer = app.renderer.as_mut().unwrap().as_mut();
        root.internal.draw(renderer, app.font_system.as_mut().unwrap(), &mut taffy_tree, taffy_root, &element_state);
        app.element_tree = Some(root.internal);
        //let renderer_submit_start = Instant::now();
        renderer.submit(resource_manager, app.font_system.as_mut().unwrap(), &element_state);
        //let renderer_duration = renderer_submit_start.elapsed();
        //println!("Renderer Submit Time Taken: {:?} ms", renderer_duration.as_millis());
        //let total_duration = total_time_start.elapsed();
        //println!("Other: {:?} ms", (total_duration - renderer_duration - duration).as_millis());
        //println!("Total Time Taken: {:?} ms", total_duration.as_millis());
    }
    //taffy_tree.print_tree(taffy_root);
}

fn layout<'a>(
    element_state: &mut StateStore,
    _window_width: f32,
    _window_height: f32,
    font_system: &mut FontSystem,
    root_element: &mut dyn Element,
    resource_manager: &RwLockReadGuard<ResourceManager>,
) -> (TaffyTree<LayoutContext<'a>>, NodeId) {
    let mut taffy_tree: taffy::TaffyTree<LayoutContext> = taffy::TaffyTree::new();
    let root_node = root_element.compute_layout(&mut taffy_tree, font_system, element_state);

    let available_space: taffy::Size<taffy::AvailableSpace> = taffy::Size {
        width: AvailableSpace::Definite(_window_width),
        height: AvailableSpace::Definite(_window_height),
    };

    taffy_tree
        .compute_layout_with_measure(
            root_node,
            available_space,
            |known_dimensions, available_space, _node_id, node_context, style| {
                measure_content(
                    element_state,
                    known_dimensions,
                    available_space,
                    node_context,
                    font_system,
                    resource_manager,
                    style,
                )
            },
        )
        .unwrap();

    let transform = glam::Mat4::IDENTITY;

    root_element.finalize_layout(&mut taffy_tree, root_node, 0.0, 0.0, transform, font_system, element_state);

    // root_element.print_tree();
    // taffy_tree.print_tree(root_node);

    (taffy_tree, root_node)
}
