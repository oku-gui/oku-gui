pub mod application;
pub mod components;
pub mod elements;
//mod lib2;
mod renderer;
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

use wgpu::{Device, Queue, RenderPipeline, Surface, SurfaceConfiguration};
use crate::renderer::color::Color;
use crate::renderer::renderer::{Rectangle, Renderer};
use crate::renderer::softbuffer::SoftBufferRenderer;
use crate::renderer::wgpu::WgpuRenderer;

const WAIT_TIME: time::Duration = time::Duration::from_millis(100);

struct App {
    app: Box<dyn Application + Send>,
    window: Option<Arc<Window>>,
    renderer: Option<Box<dyn Renderer + Send>>
}

pub struct RenderContext {
    font_system: FontSystem,
    swash_cache: SwashCache,
    surface: softbuffer::Surface<Rc<Window>, Rc<Window>>,
    canvas: Pixmap,
    cursor_x: f32,
    cursor_y: f32,
    debug_draw: bool,
    window: Rc<Window>,
}

struct OkuState {
    id: u64,
    rt: tokio::runtime::Runtime,
    request_redraw: bool,
    wait_cancelled: bool,
    close_requested: bool,
    window: Option<Arc<Window>>,
    app_to_winit_rx: mpsc::Receiver<(u64, Message)>,
    winit_to_app_tx: mpsc::Sender<(u64, Message)>
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
        winit_to_app_tx
    };

    event_loop.run_app(&mut app).expect("run_app failed");
}

impl ApplicationHandler for OkuState {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        //println!("new_events: {cause:?}");

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
            if let Some((id, Message::None)) = self.app_to_winit_rx.recv().await {
                println!("Resume Done: {}", id);
            }
            let x = self.winit_to_app_tx.send((id, Message::Resume(window.clone())));
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
                        println!("Close Done: {}", id);
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
                        println!("Resize Done: {}", id);
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
                Key::Character("r") => {
                    self.request_redraw = !self.request_redraw;
                    println!("\nrequest_redraw: {}\n", self.request_redraw);
                }
                Key::Named(NamedKey::Escape) => {
                    self.close_requested = true;
                }
                _ => (),
            },
            WindowEvent::RedrawRequested => {
                self.rt.block_on(async {
                    let id = self.id;
                    self.winit_to_app_tx.send((id, Message::RequestRedraw)).await.expect("send failed");
                    if let Some((id, Message::None)) = self.app_to_winit_rx.recv().await {
                        println!("Redraw Done: {}", id);
                    }
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

unsafe impl Send for SoftBufferRenderer {
    // Implement Send trait for SoftBufferRenderer
    // Ensure that all fields are Send
}

async fn async_main(application: Box<dyn Application + Send>, mut rx: mpsc::Receiver<(u64, Message)>, mut tx: mpsc::Sender<(u64, Message)>) {
    let mut app = App {
        app: application,
        window: None,
        renderer: None,
    };

    loop {
        if let Some((id, msg)) = rx.recv().await {
            match msg {
                Message::RequestRedraw => {
                    let renderer = app.renderer.as_mut().unwrap();
                    
                    renderer.draw_rect(Rectangle::new(0.0, 0.0, 200.0, 200.0), Color::new_from_rgba_u8(0, 255, 0, 255));
                    renderer.draw_rect(Rectangle::new(300.0, 30.0, 200.0, 200.0), Color::new_from_rgba_u8(0, 0, 255, 255));
                    
                    renderer.submit();

                    tx.send((id, Message::None)).await.expect("send failed");
                }
                Message::Close => {
                    println!("close");
                    tx.send((id, Message::None)).await.expect("send failed");
                    break;
                }
                Message::None => {
                    println!("none");
                }
                Message::Resume(window) => {
                    println!("Resumed");

                    let size = window.inner_size();

                    app.window = Some(window.clone());
                    let a = Box::new(WgpuRenderer::new(window.clone()).await);
                    //let a = Box::new(SoftBufferRenderer::new(window.clone(), size.width as f32, size.height as f32));
                    app.renderer = Some(a);

                    tx.send((id, Message::None)).await.expect("send failed");
                }
                Message::Resize(new_size) => {

                    // Reconfigure the surface with the new size

                    let renderer = app.renderer.as_mut().unwrap();

                    renderer.resize_surface(new_size.width.max(1) as f32, new_size.height.max(1) as f32);

                    // On macOS the window needs to be redrawn manually after resizing
                    app.window.as_ref().unwrap().request_redraw();

                    tx.send((id, Message::None)).await.expect("send failed");
                }
            }
        }

        println!("Message processed");
    }
}
