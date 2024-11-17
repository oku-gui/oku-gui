use std::num::NonZeroUsize;
use vello::{kurbo, peniko, AaConfig, RendererOptions};
use crate::components::component::ComponentId;
use crate::engine::renderer::color::Color;
use crate::engine::renderer::renderer::{Rectangle, RenderCommand, Renderer};
use crate::platform::resource_manager::{ResourceIdentifier, ResourceManager};
use crate::reactive::state_store::StateStore;
use cosmic_text::{FontSystem, SwashCache};
use std::sync::Arc;
use tiny_skia::{ColorSpace, Paint, PixmapPaint, PixmapRef, Transform};
use tokio::sync::RwLockReadGuard;
use vello::kurbo::{Affine, Circle, Ellipse, Line, Rect, RoundedRect, Stroke};
use vello::peniko::{BlendMode, Blob, Fill};
use vello::Scene;
use vello::util::{RenderContext, RenderSurface};
use winit::window::Window;
use crate::elements::text::TextState;
use crate::elements::text_input::TextInputState;
use crate::platform::resource_manager::resource::Resource;

pub struct ActiveRenderState<'s> {
    // The fields MUST be in this order, so that the surface is dropped before the window
    surface: RenderSurface<'s>,
    window: Arc<dyn Window>,
}

enum RenderState<'a> {
    Active(ActiveRenderState<'a>),
    // Cache a window so that it can be reused when the app is resumed after being suspended
    Suspended(Option<Arc<dyn Window>>),
}

pub struct VelloRenderer<'a> {
    render_commands: Vec<RenderCommand>,

    // The vello RenderContext which is a global context that lasts for the
    // lifetime of the application
    context: RenderContext,

    // An array of renderers, one per wgpu device
    renderers: Vec<Option<vello::Renderer>>,

    // State for our example where we store the winit Window and the wgpu Surface
    state: RenderState<'a>,

    // A vello Scene which is a data structure which allows one to build up a
    // description a scene to be drawn (with paths, fills, images, text, etc)
    // which is then passed to a renderer for rendering
    scene: Scene,
    surface_clear_color: Color,
    cache: SwashCache,
}

fn create_vello_renderer(render_cx: &RenderContext, surface: &RenderSurface) -> vello::Renderer {
    vello::Renderer::new(
        &render_cx.devices[surface.dev_id].device,
        RendererOptions {
            surface_format: Some(surface.format),
            use_cpu: false,
            antialiasing_support: vello::AaSupport {
                area: false,
                msaa8: false,
                msaa16: true,
            },
            num_init_threads: None,
        },
    )
        .expect("Couldn't create renderer")
}


impl<'a> VelloRenderer<'a> {

    pub(crate) async fn new(window: Arc<dyn Window>) -> VelloRenderer<'a> {

        let mut vello_renderer = VelloRenderer {
            render_commands: vec![],
            context: RenderContext::new(),
            renderers: vec![],
            state: RenderState::Suspended(None),
            scene: Scene::new(),
            cache: SwashCache::new(),
            surface_clear_color: Color::rgba(255, 255, 255, 255),
        };

        // Create a vello Surface
        let surface_size = window.surface_size();

        let surface = vello_renderer.context.create_surface(
            window.clone(),
            surface_size.width,
            surface_size.height,
            wgpu::PresentMode::AutoVsync,
        ).await.unwrap();

        // Create a vello Renderer for the surface (using its device id)
        vello_renderer.renderers.resize_with(vello_renderer.context.devices.len(), || None);
        vello_renderer.renderers[0].get_or_insert_with(|| create_vello_renderer(&vello_renderer.context, &surface));

        // Save the Window and Surface to a state variable
        vello_renderer.state = RenderState::Active(ActiveRenderState { window, surface });

        vello_renderer
    }
}

fn to_vello_rgba_f32_color(color: Color) -> vello::peniko::Color {
    vello::peniko::Color::rgba(color.r as f64 / 255.0, color.g as f64 / 255.0, color.b as f64 / 255.0, color.a as f64 / 255.0)
}

fn vello_draw_rect(scene: &mut Scene, rectangle: Rectangle, fill_color: Color) {

    let rect = Rect::new(rectangle.x as f64, rectangle.y as f64,
                         (rectangle.x + rectangle.width) as f64,
                         (rectangle.y + rectangle.height) as f64
    );
    scene.fill(Fill::NonZero, Affine::IDENTITY, to_vello_rgba_f32_color(fill_color), None, &rect);
}

impl Renderer for VelloRenderer<'_> {
    fn surface_width(&self) -> f32 {
        match &self.state {
            RenderState::Active(active_render_state) => {
                active_render_state.window.surface_size().width as f32
            }
            RenderState::Suspended(_) => {
                0.0
            }
        }
    }

    fn surface_height(&self) -> f32 {
        match &self.state {
            RenderState::Active(active_render_state) => {
                active_render_state.window.surface_size().height as f32
            }
            RenderState::Suspended(_) => {
                0.0
            }
        }
    }

    fn present_surface(&mut self) {
        todo!()
    }

    fn resize_surface(&mut self, width: f32, height: f32) {

        let render_state = match &mut self.state {
            RenderState::Active(state) => state,
            _ => return,
        };

        self.context.resize_surface(&mut render_state.surface, width as u32, height as u32);
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.surface_clear_color = color;
    }


    fn draw_rect(&mut self, rectangle: Rectangle, fill_color: Color) {
        self.render_commands.push(RenderCommand::DrawRect(rectangle, fill_color));
    }

    fn draw_rect_outline(&mut self, rectangle: Rectangle, outline_color: Color) {}

    fn draw_text(&mut self, element_id: ComponentId, rectangle: Rectangle, fill_color: Color) {
        self.render_commands.push(RenderCommand::DrawText(rectangle, element_id, fill_color));
    }

    fn draw_image(&mut self, rectangle: Rectangle, resource_identifier: ResourceIdentifier) {
        self.render_commands.push(RenderCommand::DrawImage(rectangle, resource_identifier));
    }

    fn push_layer(&mut self, rect: Rectangle) {
        self.render_commands.push(RenderCommand::PushLayer(rect));
    }

    fn pop_layer(&mut self) {
        self.render_commands.push(RenderCommand::PopLayer);
    }

    fn submit(
        &mut self,
        resource_manager: RwLockReadGuard<ResourceManager>,
        font_system: &mut FontSystem,
        element_state: &StateStore,
    ) {
        self.scene.reset();

        let render_state = match &mut self.state {
            RenderState::Active(state) => state,
            _ => panic!("!!!"),
        };
        
        for command in self.render_commands.drain(..) {
            match command {
                RenderCommand::DrawRect(rectangle, fill_color) => {
                    vello_draw_rect(&mut self.scene, rectangle, fill_color);
                }
                RenderCommand::DrawRectOutline(rectangle, outline_color) => {
                    // vello_draw_rect_outline(&mut self.scene, rectangle, outline_color);
                }
                RenderCommand::DrawImage(rectangle, resource_identifier) => {
                    let resource = resource_manager.resources.get(&resource_identifier);

                    if let Some(Resource::Image(resource)) = resource {
                        let image = &resource.image;
                        let data = Arc::new(image.clone().into_raw().to_vec());
                        let blob = Blob::new(data);
                        let vello_image = peniko::Image::new(blob, peniko::Format::Rgba8, image.width() as u32, image.height() as u32);

                        let mut transform= Affine::IDENTITY;
                        transform = transform.with_translation(kurbo::Vec2::new(rectangle.x as f64, rectangle.y as f64));
                        transform = transform.pre_scale_non_uniform(
                            rectangle.width as f64 / image.width() as f64,
                            rectangle.height as f64 / image.height() as f64,
                        );

                        self.scene.draw_image(&vello_image, transform);

                    }
                }
                RenderCommand::DrawText(rect, component_id, fill_color) => {
                    let clip = Rect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64);
                    
                    self.scene.push_layer(BlendMode::default(), 1.0, Affine::IDENTITY, &clip);
                    if let Some(text_context) =
                        element_state.storage.get(&component_id).unwrap().downcast_ref::<TextInputState>()
                    {
                        let editor = &text_context.editor;
                        editor.draw(
                            font_system,
                            &mut self.cache,
                            cosmic_text::Color::rgba(0, 0, 0, 255),
                            cosmic_text::Color::rgba(0, 0, 0, 100),
                            cosmic_text::Color::rgba(0, 0, 200, 255),
                            cosmic_text::Color::rgba(255, 255, 255, 255),
                            |x, y, w, h, colora: cosmic_text::Color| {
                                vello_draw_rect(&mut self.scene, 
                                                Rectangle::new(
                                                    rect.x + x as f32, rect.y + y as f32, w as f32, h as f32
                                                ),
                                                Color::rgba(colora.r(), colora.g(), colora.b(), colora.a())
                                );
                            },
                        );
                    } else if let Some(text_context) =
                        element_state.storage.get(&component_id).unwrap().downcast_ref::<TextState>()
                    {
                        let buffer = &text_context.buffer;

                        buffer.draw(
                            font_system,
                            &mut self.cache,
                            cosmic_text::Color::rgba(
                                fill_color.r_u8(),
                                fill_color.g_u8(),
                                fill_color.b_u8(),
                                fill_color.a_u8(),
                            ),
                            |x, y, w, h, color| {
                                vello_draw_rect(&mut self.scene,
                                                Rectangle::new(
                                                    rect.x + x as f32, rect.y + y as f32, w as f32, h as f32
                                                ),
                                                Color::rgba(color.r(), color.g(), color.b(), color.a())
                                );
                            },
                        );
                    } else {
                        panic!("Unknown state provided to the renderer!");
                    };
                    self.scene.pop_layer();
                },
                /*RenderCommand::PushTransform(transform) => {
                    self.scene.push_transform(transform);
                },
                RenderCommand::PopTransform => {
                    self.scene.pop_transform();
                },*/
                RenderCommand::PushLayer(rect) => {
                    let clip = Rect::new(rect.x as f64, rect.y as f64, (rect.x + rect.width) as f64, (rect.y + rect.height) as f64);
                    self.scene.push_layer(BlendMode::default(), 1.0, Affine::IDENTITY, &clip);
                },
                RenderCommand::PopLayer => {
                    self.scene.pop_layer();
                },
            }
        }
        // Get the RenderSurface (surface + config)
        let surface = &render_state.surface;

        // Get the window size
        let width = surface.config.width;
        let height = surface.config.height;

        // Get a handle to the device
        let device_handle = &self.context.devices[surface.dev_id];

        // Get the surface's texture
        let surface_texture = surface
            .surface
            .get_current_texture()
            .expect("failed to get surface texture");

        // Render to the surface's texture
        self.renderers[surface.dev_id]
            .as_mut()
            .unwrap()
            .render_to_surface(
                &device_handle.device,
                &device_handle.queue,
                &self.scene,
                &surface_texture,
                &vello::RenderParams {
                    base_color: to_vello_rgba_f32_color(self.surface_clear_color),
                    width,
                    height,
                    antialiasing_method: AaConfig::Msaa16,
                },
            )
            .expect("failed to render to surface");

        // Queue the texture to be presented on the surface
        surface_texture.present();

    }
}