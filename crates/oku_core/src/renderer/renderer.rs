#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl Rectangle {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }
}

pub enum RenderCommand {
    DrawRect(Rectangle),
}

pub trait Surface {
    fn width(&self) -> f32;
    fn height(&self) -> f32;
    fn present(&mut self);
    fn resize(&mut self, width: u32, height: u32);
}

pub trait Renderer {
    // Surface Functions
    fn surface_width(&self) -> f32;
    fn surface_height(&self) -> f32;
    fn present_surface(&mut self);
    fn resize_surface(&mut self, width: f32, height: f32);
    
    
    fn draw_rect(&mut self, rectangle: Rectangle);
    fn submit(&mut self);
}
