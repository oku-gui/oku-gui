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
    children: Vec<Box<dyn Element>>,
    style: Style,
    computed_style: Style,
    id: Option<String>,
    parent_component_id: u64,
}

impl Empty {
    pub fn new() -> Empty {
        Empty {
            children: vec![],
            style: Default::default(),
            computed_style: Default::default(),
            id: None,
            parent_component_id: 0,
        }
    }
}

impl Element for Empty {
    fn children(&self) -> Vec<Box<dyn Element>> {
        self.children.clone()
    }

    fn children_mut(&mut self) -> &mut Vec<Box<dyn Element>> {
        &mut self.children
    }

    fn name(&self) -> &'static str {
        "Empty"
    }

    fn draw(&mut self, renderer: &mut Box<dyn Renderer + Send>, render_context: &mut RenderContext) {
    }

    fn debug_draw(&mut self, render_context: &mut RenderContext) {
    }

    fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, font_system: &mut FontSystem) -> NodeId {
        let mut child_nodes: Vec<NodeId> = Vec::with_capacity(self.children().len());

        for child in self.children.iter_mut() {
            let child_node = child.compute_layout(taffy_tree, font_system);
            child_nodes.push(child_node);
        }

        let style: taffy::Style = self.style.into();

        taffy_tree.new_with_children(style, &vec![]).unwrap()
    }

    fn finalize_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, root_node: NodeId, x: f32, y: f32) {
    }

    fn computed_style(&self) -> Style {
        Style::default()
    }

    fn computed_style_mut(&mut self) -> &mut Style {
        &mut self.computed_style
    }

    fn in_bounds(&self, x: f32, y: f32) -> bool {
        false
    }

    fn add_update_handler(&mut self, update: Arc<fn(Message, Box<dyn Any>, u64)>) {
        todo!()
    }

    fn id(&self) -> &Option<String> {
        &self.id
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }

    fn parent_id(&self) -> u64 {
        self.parent_component_id
    }

    fn set_parent_id(&mut self, id: u64) {
        self.parent_component_id = id;
    }
}
