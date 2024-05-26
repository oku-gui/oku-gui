use crate::elements::element::Element;
use crate::elements::layout_context::LayoutContext;
use crate::elements::style::Style;
use crate::renderer::renderer::Renderer;
use crate::RenderContext;
use cosmic_text::FontSystem;
use std::any::Any;
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};
use crate::events::Message;

pub trait StandardElement {
    fn children(&self) -> Vec<Element>;

    fn children_mut(&mut self) -> &mut Vec<Element>;

    fn name(&self) -> &'static str;

    fn id(&self) -> u64;

    fn key(&self) -> Option<String>;
    fn key_mut(&mut self) -> &mut Option<String>;

    fn id_mut(&mut self) -> &mut u64;

    fn draw(&mut self, renderer: &mut Box<dyn Renderer + Send>, render_context: &mut RenderContext);

    fn debug_draw(&mut self, render_context: &mut RenderContext);

    fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, font_system: &mut FontSystem) -> NodeId;
    fn finalize_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, root_node: NodeId, x: f32, y: f32);

    fn computed_style(&self) -> Style;
    fn computed_style_mut(&mut self) -> &mut Style;

    fn in_bounds(&self, x: f32, y: f32) -> bool;
    fn add_update_handler(&mut self, update: Arc<fn(msg: Message, state: Box<dyn Any>, id: u64)>);
}
