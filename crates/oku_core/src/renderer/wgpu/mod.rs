mod camera;
mod context;
mod globals;
mod rectangle;
mod text;
mod render_group;
mod image;
pub(crate) mod texture;

use crate::components::component::ComponentId;
use crate::geometry::Rectangle;
use crate::renderer::color::Color;
use crate::renderer::renderer::{RenderCommand, Renderer};
use crate::renderer::wgpu::camera::Camera;
use crate::renderer::wgpu::context::{create_surface_config, request_adapter, request_device_and_queue, Context};
use crate::renderer::wgpu::globals::{GlobalBuffer, GlobalUniform};
use crate::renderer::wgpu::image::image::{ImagePerFrameData, ImageRenderer};
use crate::resource_manager::{ResourceIdentifier, ResourceManager};
use cosmic_text::FontSystem;
use peniko::kurbo::BezPath;
use std::sync::Arc;
use tokio::sync::RwLockReadGuard;
use winit::window::Window;
use crate::reactive::element_state_store::ElementStateStore;
use crate::renderer::wgpu::rectangle::{PerFrameData, RectangleRenderer};
use crate::renderer::wgpu::render_group::{ClipRectangle, RenderGroup};
use crate::renderer::wgpu::text::text::TextRenderer;
use crate::renderer::wgpu::texture::Texture;

pub struct WgpuRenderer<'a> {
    context: Context<'a>,
    // pipeline2d: Pipeline2D,
    rectangle_renderer: RectangleRenderer,
    text_renderer: TextRenderer,
    image_renderer: ImageRenderer,
    render_commands: Vec<RenderCommand>,
    render_snapshots: Vec<RenderSnapshot>,
}

pub struct RenderSnapshot {
    rectangle_per_frame_data: Option<PerFrameData>,
    text_per_frame_data: Option<PerFrameData>,
    image_per_frame_data: Option<ImagePerFrameData>,
    clip_rectangle: Rectangle,
}

impl<'a> WgpuRenderer<'a> {
    pub(crate) async fn new(window: Arc<dyn Window>) -> WgpuRenderer<'a> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = request_adapter(instance, &surface).await;
        let (device, queue) = request_device_and_queue(&adapter).await;

        let surface_size = window.surface_size();
        let surface_config =
            create_surface_config(&surface, surface_size.width, surface_size.height, &device, &adapter);
        surface.configure(&device, &surface_config);

        let camera = Camera {
            width: surface_config.width as f32,
            height: surface_config.height as f32,
            z_near: 0.0,
            z_far: 100.0,
        };
        let mut global_buffer_uniform = GlobalUniform::new();
        global_buffer_uniform.set_view_proj_with_camera(&camera);

        let global_buffer = GlobalBuffer::new(&device, &global_buffer_uniform);

        let default_texture = Texture::generate_default_white_texture(&device, &queue);
        
        let context = Context {
            camera,
            device,
            queue,
            global_buffer,
            global_buffer_uniform,
            surface,
            surface_config,
            surface_clear_color: Color::WHITE,
            is_srgba_format: false,
            default_texture
        };
        let rectangle_renderer = RectangleRenderer::new(&context);
        let text_renderer = TextRenderer::new(&context);
        let image_renderer = ImageRenderer::new(&context);

        WgpuRenderer {
            context,
            rectangle_renderer,
            text_renderer,
            image_renderer,
            render_commands: vec![],
            render_snapshots: vec![],
        }
    }
}

impl Renderer for WgpuRenderer<'_> {
    fn surface_width(&self) -> f32 {
        self.context.surface_config.width as f32
    }

    fn surface_height(&self) -> f32 {
        self.context.surface_config.height as f32
    }

    fn present_surface(&mut self) {
        todo!()
    }

    fn resize_surface(&mut self, width: f32, height: f32) {
        self.context.surface_config.width = width as u32;
        self.context.surface_config.height = height as u32;
        self.context.surface.configure(&self.context.device, &self.context.surface_config);
        self.context.camera = Camera {
            width,
            height,
            z_near: 0.0,
            z_far: 100.0,
        };


        self.context.global_buffer_uniform.set_view_proj_with_camera(&self.context.camera);
        self.context.global_buffer.update(&self.context.queue, &self.context.global_buffer_uniform);
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.context.surface_clear_color = color;
    }

    fn draw_rect(&mut self, rectangle: Rectangle, fill_color: Color) {
        self.render_commands.push(RenderCommand::DrawRect(rectangle, fill_color));
    }

    fn draw_rect_outline(&mut self, rectangle: Rectangle, outline_color: Color) {
        //self.pipeline2d.draw_rect_outline(rectangle, outline_color);
    }

    fn fill_bez_path(&mut self, path: BezPath, color: Color) {
        self.render_commands.push(RenderCommand::FillBezPath(path, color));
    }

    fn draw_text(&mut self, element_id: ComponentId, rectangle: Rectangle, fill_color: Color) {
        self.render_commands.push(RenderCommand::DrawText(rectangle, element_id, fill_color));
    }

    fn draw_image(&mut self, rectangle: Rectangle, resource_identifier: ResourceIdentifier) {
        self.render_commands.push(RenderCommand::DrawImage(rectangle, resource_identifier));
    }

    fn push_layer(&mut self, clip_rect: Rectangle) {
        self.render_commands.push(RenderCommand::PushLayer(clip_rect));
    }

    fn pop_layer(&mut self) {
        self.render_commands.push(RenderCommand::PopLayer);
    }

    fn prepare(&mut self, _resource_manager: RwLockReadGuard<ResourceManager>, font_system: &mut FontSystem, element_state: &ElementStateStore) {

        let render_commands_len = self.render_commands.len();

        if render_commands_len == 0 {
            return;
        }

        
        let render_commands = self.render_commands.drain(..);

        let viewport_clip_rect = Rectangle {
            x: 0.0,
            y: 0.0,
            width: self.context.surface_config.width as f32,
            height: self.context.surface_config.height as f32
        };

        let mut render_groups: Vec<RenderGroup> = Vec::new();
        render_groups.push(RenderGroup {
            clip_rectangle: viewport_clip_rect,
        });

        for (index, command) in render_commands.enumerate() {
            let mut should_submit = index == render_commands_len - 1;

            match command {
                RenderCommand::PushLayer(clip_rectangle) => {
                    let parent_clip_rectangle = render_groups.last().unwrap().clip_rectangle;
                    let constrained_clip_rectangle = clip_rectangle.constrain_to_clip_rectangle(&parent_clip_rectangle);
                    render_groups.push(RenderGroup {
                        clip_rectangle: constrained_clip_rectangle
                    });

                    let snapshot = assemble_render_snapshot(&mut self.context, font_system, element_state, &mut self.rectangle_renderer, &mut self.text_renderer, &mut self.image_renderer, parent_clip_rectangle);
                    self.render_snapshots.push(snapshot);
                }
                RenderCommand::PopLayer => {
                    should_submit = true;
                }
                RenderCommand::DrawRect(rectangle, fill_color) => {
                    self.rectangle_renderer.build(rectangle, fill_color);
                }
                RenderCommand::DrawRectOutline(_, _) => {}
                RenderCommand::DrawImage(rectangle, resource_identifier) => {
                    self.image_renderer.build(rectangle, resource_identifier, Color::WHITE);
                }
                RenderCommand::DrawText(rectangle, component_id, color) => {
                    self.text_renderer.build(rectangle, component_id, color);
                }
                RenderCommand::FillBezPath(_, _) => {

                }
            }

            if should_submit {
                let current_clip_rectangle = render_groups.pop().unwrap().clip_rectangle;
                let snapshot = assemble_render_snapshot(&mut self.context, font_system, element_state, &mut self.rectangle_renderer, &mut self.text_renderer, &mut self.image_renderer, current_clip_rectangle);
                self.render_snapshots.push(snapshot);
            }

        }
    }


    fn submit(&mut self, resource_manager: RwLockReadGuard<ResourceManager>) {
        let mut encoder = self.context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let output = self.context.surface.get_current_texture().unwrap();
        let texture_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let [r, g, b, a] = self.context.surface_clear_color.components;

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Oku Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: r as f64,
                            g: g as f64,
                            b: b as f64,
                            a: a as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            
            for snapshot in &self.render_snapshots {
                
                let clip_rectangle = ClipRectangle::constrain_to_clip_rectangle(&snapshot.clip_rectangle, &ClipRectangle {
                    x: 0.0,
                    y: 0.0,
                    width: output.texture.width() as f32,
                    height: output.texture.height() as f32,
                });
                render_pass.set_scissor_rect(
                    clip_rectangle.x as u32,
                    clip_rectangle.y as u32,
                    clip_rectangle.width as u32,
                    clip_rectangle.height as u32,
                );
                
                if let Some(rectangle_per_frame_data) = snapshot.rectangle_per_frame_data.as_ref() {
                    self.rectangle_renderer.draw(&self.context, &mut render_pass, rectangle_per_frame_data);    
                }

                if let Some(text_per_frame_data) = snapshot.text_per_frame_data.as_ref() {
                    self.text_renderer.draw(&mut self.context, &mut render_pass, text_per_frame_data);
                }

                if let Some(image_per_frame_data) = snapshot.image_per_frame_data.as_ref() {
                    self.image_renderer.draw(&mut self.context, &resource_manager, &mut render_pass, image_per_frame_data);
                }
                
            }
        }

        self.context.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.render_snapshots.clear();
    }
}

fn assemble_render_snapshot(
        context: &mut Context,
        font_system: &mut FontSystem,
        element_state: &ElementStateStore,
        rectangle_renderer: &mut RectangleRenderer,
        text_renderer: &mut TextRenderer,
        image_renderer: &mut ImageRenderer,
        clip_rectangle: ClipRectangle
) -> RenderSnapshot {

    let rectangle_renderer_per_frame_data = rectangle_renderer.prepare(context);
    let text_renderer_per_frame_data = text_renderer.prepare(context, font_system, element_state);
    let image_renderer_per_frame_data = image_renderer.prepare(context);
    
    RenderSnapshot {
        rectangle_per_frame_data: rectangle_renderer_per_frame_data,
        text_per_frame_data: text_renderer_per_frame_data,
        image_per_frame_data: image_renderer_per_frame_data,
        clip_rectangle,
    }
}
