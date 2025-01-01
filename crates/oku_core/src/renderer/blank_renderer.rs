use cosmic_text::FontSystem;
use peniko::kurbo::BezPath;
use tokio::sync::RwLockReadGuard;
use crate::components::ComponentId;
use crate::geometry::Rectangle;
use crate::reactive::state_store::StateStore;
use crate::renderer::color::Color;
use crate::renderer::renderer::Renderer;
use crate::resource_manager::{ResourceIdentifier, ResourceManager};

pub struct BlankRenderer;

impl Renderer for BlankRenderer {
    fn surface_width(&self) -> f32 {
        0.0
    }

    fn surface_height(&self) -> f32 {
        0.0
    }

    fn present_surface(&mut self) {
    }

    fn resize_surface(&mut self, _width: f32, _height: f32) {
    }

    fn surface_set_clear_color(&mut self, _color: Color) {
    }

    fn draw_rect(&mut self, _rectangle: Rectangle, _fill_color: Color) {
    }

    fn draw_rect_outline(&mut self, _rectangle: Rectangle, _outline_color: Color) {
    }

    fn fill_bez_path(&mut self, _path: BezPath, _color: Color) {
    }

    fn draw_text(&mut self, _element_id: ComponentId, _rectangle: Rectangle, _fill_color: Color) {
    }

    fn draw_image(&mut self, _rectangle: Rectangle, _resource_identifier: ResourceIdentifier) {
    }

    fn push_layer(&mut self, _rect: Rectangle) {
    }

    fn pop_layer(&mut self) {
    }

    fn submit(&mut self, _resource_manager: RwLockReadGuard<ResourceManager>, _font_system: &mut FontSystem, _element_state: &StateStore) {
    }
}