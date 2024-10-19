use std::collections::HashMap;
use crate::engine::renderer::color::Color;
use crate::engine::renderer::renderer::{Rectangle, RenderCommand, Renderer};
use softbuffer::Buffer;
use std::num::NonZeroU32;
use std::sync::Arc;
use cosmic_text::FontSystem;
use taffy::TaffyTree;
use tiny_skia::{ColorSpace, Paint, Pixmap, Rect, Transform};
use tokio::sync::RwLockReadGuard;
use winit::window::Window;
use crate::platform::resource_manager::{ResourceIdentifier, ResourceManager};
use crate::RenderContext;
use crate::user::components::component::{ComponentId, GenericUserState};
use crate::user::elements::layout_context::LayoutContext;

pub struct SoftwareRenderer {
    render_commands: Vec<RenderCommand>,

    // Surface
    surface: softbuffer::Surface<Arc<dyn Window>, Arc<dyn Window>>,
    surface_width: f32,
    surface_height: f32,
    surface_clear_color: Color,
    framebuffer: Pixmap,
}

impl SoftwareRenderer {
    pub(crate) fn new(window: Arc<dyn Window>) -> Self {
        let width = window.surface_size().width as f32;
        let height = window.surface_size().height as f32;

        let context = softbuffer::Context::new(window.clone()).unwrap();
        let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();
        surface
            .resize(NonZeroU32::new(width as u32).unwrap(), NonZeroU32::new(height as u32).unwrap())
            .expect("TODO: panic message");
        let framebuffer = Pixmap::new(width as u32, height as u32).unwrap();

        Self {
            render_commands: vec![],
            surface,
            surface_width: width,
            surface_height: height,
            surface_clear_color: Color::new_from_rgba_u8(255, 255, 255, 255),
            framebuffer,
        }
    }
}

fn draw_rect(canvas: &mut Pixmap, rectangle: Rectangle, fill_color: Color) {
    let mut paint = Paint::default();
    paint.colorspace = ColorSpace::Linear;
    paint.set_color_rgba8(fill_color.r_u8(), fill_color.g_u8(), fill_color.b_u8(), fill_color.a_u8());
    paint.anti_alias = true;

    let rect = Rect::from_xywh(rectangle.x, rectangle.y, rectangle.width, rectangle.height).unwrap();
    canvas.fill_rect(rect, &paint, Transform::identity(), None);
}

const fn rgba_to_encoded_u32(r: u32, g: u32, b: u32, a: u32) -> u32 {
    b | (g << 8) | (r << 16) | (a << 24)
}

impl Renderer for SoftwareRenderer {
    fn surface_width(&self) -> f32 {
        self.surface_width
    }

    fn surface_height(&self) -> f32 {
        self.surface_height
    }

    fn present_surface(&mut self) {
        todo!()
    }

    fn resize_surface(&mut self, width: f32, height: f32) {
        self.surface_width = width;
        self.surface_height = height;
        let framebuffer = Pixmap::new(width as u32, height as u32).unwrap();
        self.surface
            .resize(NonZeroU32::new(width as u32).unwrap(), NonZeroU32::new(height as u32).unwrap())
            .expect("TODO: panic message");
        self.framebuffer = framebuffer;
    }

    fn surface_set_clear_color(&mut self, color: Color) {
        self.surface_clear_color = color;
    }

    fn draw_rect(&mut self, rectangle: Rectangle, fill_color: Color) {
        self.render_commands.push(RenderCommand::DrawRect(rectangle, fill_color));
    }

    fn draw_text(&mut self, element_id: ComponentId,  rectangle: Rectangle, fill_color: Color) {
        todo!()
    }

    fn draw_image(&mut self, _rectangle: Rectangle, resource_identifier: ResourceIdentifier) {
        todo!()
    }

    fn submit(&mut self, resource_manager: RwLockReadGuard<ResourceManager>, render_context: &mut RenderContext, element_state: &HashMap<ComponentId, Box<GenericUserState>>) {
        self.framebuffer.fill(tiny_skia::Color::from_rgba8(
            self.surface_clear_color.r_u8(),
            self.surface_clear_color.g_u8(),
            self.surface_clear_color.b_u8(),
            self.surface_clear_color.a_u8(),
        ));

        for command in self.render_commands.drain(..) {
            match command {
                RenderCommand::DrawRect(rectangle, fill_color) => {
                    draw_rect(&mut self.framebuffer, rectangle, fill_color);
                }
            }
        }
        let buffer = self.copy_skia_buffer_to_softbuffer(self.surface_width, self.surface_height);
        buffer.present().unwrap();
    }
}

impl SoftwareRenderer {
    fn copy_skia_buffer_to_softbuffer(&mut self, width: f32, height: f32) -> Buffer<Arc<dyn Window>, Arc<dyn Window>> {
        let mut buffer = self.surface.buffer_mut().unwrap();
        for y in 0..height as u32 {
            for x in 0..width as u32 {
                let index = y as usize * width as usize + x as usize;
                let current_pixel = self.framebuffer.pixels()[index];

                let red = current_pixel.red() as u32;
                let green = current_pixel.green() as u32;
                let blue = current_pixel.blue() as u32;
                let alpha = current_pixel.alpha() as u32;

                buffer[index] = rgba_to_encoded_u32(red, green, blue, alpha);
            }
        }
        buffer
    }
}
