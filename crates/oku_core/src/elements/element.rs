use crate::elements::container::Container;
use crate::elements::empty::Empty;
use crate::elements::layout_context::LayoutContext;
use crate::elements::standard_element::StandardElement;
use crate::elements::style::Style;
use crate::elements::text::Text;
use crate::renderer::renderer::Renderer;
use crate::RenderContext;
use cosmic_text::FontSystem;
use taffy::{NodeId, TaffyTree};

#[derive(Clone, Debug)]
pub enum Element {
    Container(Container),
    Text(Text),
    Empty(Empty),
}

impl StandardElement for Element {
    fn id(&self) -> u64 {
        match self {
            Element::Container(container) => container.id(),
            Element::Text(text) => text.id(),
            Element::Empty(empty) => empty.id(),
        }
    }

    fn key(&self) -> Option<String> {
        match self {
            Element::Container(container) => container.key(),
            Element::Text(text) => text.key(),
            Element::Empty(empty) => empty.key(),
        }
    }

    fn id_mut(&mut self) -> &mut u64 {
        match self {
            Element::Container(container) => container.id_mut(),
            Element::Text(text) => text.id_mut(),
            Element::Empty(empty) => empty.id_mut(),
        }
    }

    fn draw(&mut self, renderer: &mut Box<dyn Renderer + Send>, render_context: &mut RenderContext) {
        match self {
            Element::Container(container) => container.draw(renderer, render_context),
            Element::Text(text) => text.draw(renderer, render_context),
            Element::Empty(empty) => empty.draw(render_context),
        }
    }

    fn debug_draw(&mut self, render_context: &mut RenderContext) {
        match self {
            Element::Container(container) => container.debug_draw(render_context),
            Element::Text(text) => text.debug_draw(render_context),
            Element::Empty(empty) => empty.debug_draw(render_context),
        }
    }
    fn children(&self) -> Vec<Element> {
        match self {
            Element::Container(container) => container.children(),
            Element::Text(text) => text.children(),
            Element::Empty(empty) => empty.children(),
        }
    }

    fn children_mut(&mut self) -> &mut Vec<Element> {
        match self {
            Element::Container(container) => container.children_mut(),
            Element::Text(text) => text.children_mut(),
            Element::Empty(empty) => empty.children_mut(),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Element::Container(container) => container.name(),
            Element::Text(text) => text.name(),
            Element::Empty(empty) => empty.name(),
        }
    }

    fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, font_system: &mut FontSystem) -> NodeId {
        match self {
            Element::Container(container) => container.compute_layout(taffy_tree, font_system),
            Element::Text(text) => text.compute_layout(taffy_tree, font_system),
            Element::Empty(empty) => empty.compute_layout(taffy_tree, font_system),
        }
    }

    fn computed_style(&self) -> Style {
        match self {
            Element::Container(container) => container.computed_style(),
            Element::Text(text) => text.computed_style(),
            Element::Empty(empty) => empty.computed_style(),
        }
    }

    fn computed_style_mut(&mut self) -> &mut Style {
        match self {
            Element::Container(container) => container.computed_style_mut(),
            Element::Text(text) => text.computed_style_mut(),
            Element::Empty(empty) => empty.computed_style_mut(),
        }
    }

    fn finalize_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, root_node: NodeId, x: f32, y: f32) {
        match self {
            Element::Container(container) => container.finalize_layout(taffy_tree, root_node, x, y),
            Element::Text(text) => text.finalize_layout(taffy_tree, root_node, x, y),
            Element::Empty(_) => {}
        }
    }

    fn in_bounds(&self, x: f32, y: f32) -> bool {
        match self {
            Element::Container(container) => container.in_bounds(x, y),
            Element::Text(text) => text.in_bounds(x, y),
            Element::Empty(empty) => empty.in_bounds(x, y),
        }
    }
}
