use crate::elements::element::Element;
use crate::elements::layout_context::LayoutContext;
use crate::elements::style::Style;
use crate::RenderContext;
use cosmic_text::FontSystem;
use taffy::{NodeId, TaffyTree};
use crate::widget_id::create_unique_widget_id;

#[derive(Clone, Default, Debug)]
pub struct Empty {
    id: u64,
    key: Option<String>,
    children: Vec<Element>,
}

impl Empty {
    pub fn new() -> Empty {
        Empty {
            id: create_unique_widget_id(),
            key: None,
            children: vec![],
        }
    }
}

impl Empty {
    pub fn add_child(self, _widget: Element) -> Empty {
        panic!("Empty cannot have children");
    }

    pub fn children(&self) -> Vec<Element> {
        self.children.clone()
    }

    pub fn children_mut(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }

    pub const fn name(&self) -> &'static str {
        "Empty"
    }

    pub const fn id(&self) -> u64 {
        self.id
    }

    pub fn key(&self) -> Option<String> {
        self.key.clone()
    }
    pub(crate) fn key_mut(&mut self) -> &mut Option<String> {
        &mut self.key
    }

    pub fn id_mut(&mut self) -> &mut u64 {
        &mut self.id
    }

    pub fn draw(&mut self, _render_context: &mut RenderContext) {}

    pub fn debug_draw(&mut self, _render_context: &mut RenderContext) {}

    pub fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, _font_system: &mut FontSystem) -> NodeId {
        taffy_tree.new_leaf(Style::default().into()).unwrap()
    }

    pub fn finalize_layout(&mut self, _taffy_tree: &mut TaffyTree<LayoutContext>, _root_node: NodeId, _x: f32, _y: f32) {}

    pub fn computed_style(&self) -> Style {
        Style::default()
    }

    pub fn computed_style_mut(&mut self) -> &mut Style {
        panic!("Empty cannot have a style");
    }

    pub fn in_bounds(&self, _x: f32, _y: f32) -> bool {
        false
    }
}
