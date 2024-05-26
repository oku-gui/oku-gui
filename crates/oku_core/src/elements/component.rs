use crate::elements::container::Container;
use crate::elements::element::Element;
use crate::elements::empty::Empty;
use crate::elements::layout_context::LayoutContext;
use crate::elements::standard_element::StandardElement;
use crate::elements::style::Style;
use crate::renderer::renderer::Renderer;
use crate::RenderContext;
use cosmic_text::FontSystem;
use std::any::Any;
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};

fn default_update(msg: Box<dyn Any>, state: Box<dyn Any>) {}

#[derive(Clone)]
pub struct Component {
    id: u64,
    key: Option<String>,
    children: Vec<Element>,
    pub update: Arc<fn(msg: Box<dyn Any>, state: Box<dyn Any>)>,
}
impl Component {
    pub fn new() -> Component {
        Component {
            id: u64::MAX,
            key: None,
            children: vec![],
            update: Arc::new(default_update),
        }
    }
}

impl Component {
    pub fn add_child(mut self, widget: Element) -> Self {
        self.children.insert(0, widget);
        self
    }

    pub fn children(&self) -> Vec<Element> {
        self.children.clone()
    }

    pub fn children_mut(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }

    pub const fn name(&self) -> &'static str {
        "Component"
    }

    pub const fn id(&self) -> u64 {
        self.id
    }

    pub fn key(&self) -> Option<String> {
        self.key.clone()
    }

    pub fn id_mut(&mut self) -> &mut u64 {
        &mut self.id
    }

    pub(crate) fn draw(&mut self, renderer: &mut Box<dyn Renderer + Send>, render_context: &mut RenderContext) {
        self.children[0].draw(renderer, render_context)
    }

    pub fn debug_draw(&mut self, _render_context: &mut RenderContext) {}

    pub fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, font_system: &mut FontSystem) -> NodeId {
        self.children[0].compute_layout(taffy_tree, font_system)
    }

    pub fn finalize_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, root_node: NodeId, x: f32, y: f32) {
        self.children[0].finalize_layout(taffy_tree, root_node, x, y)
    }

    pub fn computed_style(&self) -> Style {
        self.children[0].computed_style()
    }

    pub fn computed_style_mut(&mut self) -> &mut Style {
        self.children[0].computed_style_mut()
    }

    pub fn in_bounds(&self, x: f32, y: f32) -> bool {
        self.children[0].in_bounds(x, y)
    }

    pub fn add_update_handler(&mut self, update: Arc<fn(msg: Box<dyn Any>, state: Box<dyn Any>)>) {
        self.update = update;
    }
}
