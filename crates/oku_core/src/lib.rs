pub mod application;
pub mod components;
pub mod elements;
//mod lib2;
pub mod renderer;
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

use crate::elements::container::Container;
use crate::elements::layout_context::{measure_content, LayoutContext};
use crate::elements::standard_element::StandardElement;
use crate::elements::style::Unit;
use crate::renderer::color::Color;
use crate::renderer::renderer::{Rectangle, Renderer};
use crate::renderer::softbuffer::SoftwareRenderer;
use crate::renderer::wgpu::WgpuRenderer;
use wgpu::{Device, Queue, RenderPipeline, Surface, SurfaceConfiguration};

const WAIT_TIME: time::Duration = time::Duration::from_millis(100);

struct App {
    app: Box<dyn Application + Send>,
    window: Option<Arc<Window>>,
    renderer: Option<Box<dyn Renderer + Send>>,
    renderer_context: Option<RenderContext>,
    element_tree: Option<Element>,
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

pub fn oku_main(application: Box<dyn Application + Send>) {
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Failed to create runtime");

    let event_loop = EventLoop::new().unwrap();

    let (winit_to_app_tx, winit_to_app_rx) = mpsc::channel::<(u64, Message)>(100);
    let (app_to_winit_tx, app_to_winit_rx) = mpsc::channel::<(u64, Message)>(100);

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
    };

    event_loop.run_app(&mut app).expect("run_app failed");
}

impl ApplicationHandler for OkuState {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        self.wait_cancelled = match cause {
            StartCause::WaitCancelled { .. } => true,
            _ => false,
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("oku");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());

        let id = self.id;
        self.rt.block_on(async {
            self.winit_to_app_tx.send((id, Message::Resume(window.clone()))).await.expect("send failed");
            if let Some((_id, Message::None)) = self.app_to_winit_rx.recv().await {}
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
                        //println!("Close Done: {}", id);
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
                        //println!("Resize Done: {}", id);
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
                Key::Named(NamedKey::Escape) => {
                    self.close_requested = true;
                }
                _ => (),
            },
            WindowEvent::RedrawRequested => {
                self.rt.block_on(async {
                    let id = self.id;
                    self.winit_to_app_tx.send((id, Message::RequestRedraw)).await.expect("send failed");
                    if let Some((id, Message::None)) = self.app_to_winit_rx.recv().await {}
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

unsafe impl Send for SoftwareRenderer {
    // Implement Send trait for SoftBufferRenderer
    // Ensure that all fields are Send
}

async fn async_main(application: Box<dyn Application + Send>, mut rx: mpsc::Receiver<(u64, Message)>, mut tx: mpsc::Sender<(u64, Message)>) {
    let mut app = App {
        app: application,
        window: None,
        renderer: None,
        renderer_context: None,
        element_tree: None,
    };

    loop {
        if let Some((id, msg)) = rx.recv().await {
            match msg {
                Message::RequestRedraw => {
                    let renderer = app.renderer.as_mut().unwrap();

                    renderer.surface_set_clear_color(Color::new_from_rgba_u8(22, 0, 100, 255));
                    //renderer.draw_rect(Rectangle::new(0.0, 0.0, 200.0, 200.0), Color::new_from_rgba_u8(0, 255, 0, 255));
                    //renderer.draw_rect(Rectangle::new(300.0, 30.0, 200.0, 200.0), Color::new_from_rgba_u8(0, 0, 255, 255));

                    if let Some(mut root) = app.element_tree.clone() {
                        let mut window_element = Container::new();

                        window_element = window_element.width(Unit::Px(renderer.surface_width()));
                        let computed_style = &root.computed_style_mut();

                        // The root element should be 100% window width if the width is not already set.
                        if computed_style.width.is_auto() {
                            root.computed_style_mut().width = Unit::Px(renderer.surface_width());
                        }

                        window_element = window_element.add_child(root.clone());
                        let mut window_element = Element::Container(window_element);

                        layout(renderer.surface_width(), renderer.surface_height(), app.renderer_context.as_mut().unwrap(), &mut window_element);
                        window_element.draw(renderer, app.renderer_context.as_mut().unwrap());
                    }

                    renderer.submit();

                    tx.send((id, Message::None)).await.expect("send failed");
                }
                Message::Close => {
                    tx.send((id, Message::None)).await.expect("send failed");
                    break;
                }
                Message::None => {}
                Message::Resume(window) => {
                    if app.element_tree.is_none() {
                        let new_view = app.app.view();
                        app.element_tree = Some(new_view);
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
                    //let renderer = Box::new(WgpuRenderer::new(window.clone()).await);
                    let renderer = Box::new(SoftwareRenderer::new(window.clone()));
                    app.renderer = Some(renderer);

                    tx.send((id, Message::None)).await.expect("send failed");
                }
                Message::Resize(new_size) => {
                    let renderer = app.renderer.as_mut().unwrap();
                    renderer.resize_surface(new_size.width.max(1) as f32, new_size.height.max(1) as f32);

                    // On macOS the window needs to be redrawn manually after resizing
                    app.window.as_ref().unwrap().request_redraw();

                    tx.send((id, Message::None)).await.expect("send failed");
                }
            }
        }
    }
}

fn layout(_window_width: f32, _window_height: f32, render_context: &mut RenderContext, root_element: &mut Element) {
    let mut taffy_tree: taffy::TaffyTree<LayoutContext> = taffy::TaffyTree::new();
    let root_node = root_element.compute_layout(&mut taffy_tree, &mut render_context.font_system);

    taffy_tree
        .compute_layout_with_measure(root_node, taffy::Size::max_content(), |known_dimensions, available_space, _node_id, node_context| measure_content(known_dimensions, available_space, node_context, &mut render_context.font_system))
        .unwrap();

    root_element.finalize_layout(&mut taffy_tree, root_node, 0.0, 0.0);
}
