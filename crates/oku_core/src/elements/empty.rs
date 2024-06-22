use std::any::Any;
use std::sync::Arc;
use crate::elements::layout_context::LayoutContext;
use crate::elements::style::Style;
use crate::RenderContext;
use cosmic_text::FontSystem;
use taffy::{NodeId, TaffyTree};
use crate::elements::container::Container;
use crate::elements::element::Element;
use crate::events::Message;
use crate::renderer::renderer::Renderer;
use crate::widget_id::create_unique_widget_id;

#[derive(Clone, Default, Debug)]
pub struct Empty {
    id: u64,
    key: Option<String>,
    tag: Option<String>,
    children: Vec<Box<dyn Element>>,
}

impl Empty {
    pub fn new() -> Empty {
        Empty {
            id: create_unique_widget_id(),
            key: None,
            tag: None,
            children: vec![],
        }
    }
}

impl Element for Empty {
    fn children(&self) -> Vec<Box<dyn Element>> {
        vec![]
    }

    fn children_mut(&mut self) -> &mut Vec<Box<dyn Element>> {
        todo!()
    }

    fn name(&self) -> &'static str {
        "Empty"
    }

    fn id(&self) -> u64 {
        todo!()
    }

    fn key(&self) -> Option<String> {
        todo!()
    }

    fn key_mut(&mut self) -> &mut Option<String> {
        todo!()
    }

    fn tag(&self) -> Option<String> {
        self.tag.clone()
    }

    fn tag_mut(&mut self) -> &mut Option<String> {
        &mut self.tag
    }

    fn id_mut(&mut self) -> &mut u64 {
        todo!()
    }

    fn draw(&mut self, renderer: &mut Box<dyn Renderer + Send>, render_context: &mut RenderContext) {
        todo!()
    }

    fn debug_draw(&mut self, render_context: &mut RenderContext) {
        todo!()
    }

    fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, font_system: &mut FontSystem) -> NodeId {
        todo!()
    }

    fn finalize_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, root_node: NodeId, x: f32, y: f32) {
        todo!()
    }

    fn computed_style(&self) -> Style {
        todo!()
    }

    fn computed_style_mut(&mut self) -> &mut Style {
        todo!()
    }

    fn in_bounds(&self, x: f32, y: f32) -> bool {
        todo!()
    }

    fn add_update_handler(&mut self, update: Arc<fn(Message, Box<dyn Any>, u64)>) {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        todo!()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        todo!()
    }
}
