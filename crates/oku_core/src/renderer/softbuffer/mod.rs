use std::num::NonZeroU32;
use crate::renderer::renderer::{Rectangle, RenderCommand, Renderer, Surface};
use std::rc::Rc;
use std::sync::Arc;
use softbuffer::Buffer;
use tiny_skia::{LineCap, LineJoin, Paint, PathBuilder, Pixmap, Rect, Transform};
use winit::window::Window;

pub struct SoftBufferRenderer {
    render_commands: Vec<RenderCommand>,

    // Surface
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    surface_width: f32,
    surface_height: f32,
    framebuffer: tiny_skia::Pixmap,
}

impl SoftBufferRenderer {
    pub(crate) fn new(window: Arc<Window>, width: f32, height: f32) -> Self {
        let context = softbuffer::Context::new(window.clone()).unwrap();
        let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();
        surface.resize(NonZeroU32::new(width as u32).unwrap(), NonZeroU32::new(height as u32).unwrap()).expect("TODO: panic message");
        let framebuffer = tiny_skia::Pixmap::new(width as u32, height as u32).unwrap();

        Self {
            render_commands: vec![],
            surface,
            surface_width: width,
            surface_height: height,
            framebuffer,
        }
    }
}

fn draw_rect(canvas: &mut Pixmap, rectangle: Rectangle) {
    let mut paint = Paint::default();
    paint.set_color_rgba8(255, 0, 0, 255);
    paint.anti_alias = true;

    let mut path_builder = PathBuilder::new();
    path_builder.push_rect(Rect::from_xywh(rectangle.x, rectangle.y, rectangle.width, rectangle.height).unwrap());
    let path = path_builder.finish().unwrap();

    let stroke = tiny_skia::Stroke {
        width: 3.0,
        miter_limit: 100.0,
        line_cap: LineCap::Butt,
        line_join: LineJoin::Miter,

        dash: None,
        // Dashed lines
        // dash: Some(StrokeDash::new(vec![2.0, 5.0], 5.0).unwrap()),
    };

    canvas.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
}

const fn rgb_to_encoded_u32(r: u32, g: u32, b: u32) -> u32 {
    b | (g << 8) | (r << 16)
}

impl Renderer for SoftBufferRenderer {
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
        let framebuffer = tiny_skia::Pixmap::new(width as u32, height as u32).unwrap();
        self.surface.resize(NonZeroU32::new(width as u32).unwrap(), NonZeroU32::new(height as u32).unwrap()).expect("TODO: panic message");
        self.framebuffer = framebuffer;
    }

    fn draw_rect(&mut self, rectangle: Rectangle) {
        self.render_commands.push(RenderCommand::DrawRect(rectangle));
    }

    fn submit(&mut self) {
        for command in self.render_commands.drain(..) {
            match command {
                RenderCommand::DrawRect(rectangle) => {
                    draw_rect(&mut self.framebuffer, rectangle);
                }
            }
        }

        println!("{}, {}", self.surface_height, self.surface_height);
        let buffer = self.copy_skia_buffer_to_softbuffer(self.surface_width, self.surface_height);
        buffer.present().unwrap();
    }

}

impl SoftBufferRenderer {
    fn copy_skia_buffer_to_softbuffer(&mut self, width: f32, height: f32) -> Buffer<Arc<Window>, Arc<Window>> {
        let mut buffer = self.surface.buffer_mut().unwrap();
        for y in 0..height as u32 {
            for x in 0..width as u32 {
                let index = y as usize * width as usize + x as usize;
                let current_pixel = self.framebuffer.pixels()[index];

                let red = current_pixel.red() as u32;
                let green = current_pixel.green() as u32;
                let blue = current_pixel.blue() as u32;

                buffer[index] = rgb_to_encoded_u32(red, green, blue);
            }
        }
        buffer
    }
}