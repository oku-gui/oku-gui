use crate::elements::component::Component;
use crate::elements::container::Container;
use crate::elements::empty::Empty;
use crate::elements::layout_context::LayoutContext;
use crate::elements::standard_element::StandardElement;
use crate::elements::style::Style;
use crate::elements::text::Text;
use crate::renderer::renderer::Renderer;
use crate::RenderContext;
use cosmic_text::FontSystem;
use std::any::Any;
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};
use crate::events::Message;

#[derive(Clone, Debug)]
pub enum Element {
    Container(Container),
    Text(Text),
    Empty(Empty),
    Component(Component),
}

impl StandardElement for Element {
    fn children(&self) -> Vec<Element> {
        match self {
            Element::Container(container) => container.children(),
            Element::Text(text) => text.children(),
            Element::Empty(empty) => empty.children(),
            Element::Component(component) => component.children(),
        }
    }

    fn children_mut(&mut self) -> &mut Vec<Element> {
        match self {
            Element::Container(container) => container.children_mut(),
            Element::Text(text) => text.children_mut(),
            Element::Empty(empty) => empty.children_mut(),
            Element::Component(component) => component.children_mut(),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Element::Container(container) => container.name(),
            Element::Text(text) => text.name(),
            Element::Empty(empty) => empty.name(),
            Element::Component(component) => component.name(),
        }
    }

    fn id(&self) -> u64 {
        match self {
            Element::Container(container) => container.id(),
            Element::Text(text) => text.id(),
            Element::Empty(empty) => empty.id(),
            Element::Component(component) => component.id(),
        }
    }

    fn key(&self) -> Option<String> {
        match self {
            Element::Container(container) => container.key(),
            Element::Text(text) => text.key(),
            Element::Empty(empty) => empty.key(),
            Element::Component(component) => component.key(),
        }
    }

    fn key_mut(&mut self) -> &mut Option<String> {
        match self {
            Element::Container(container) => container.key_mut(),
            Element::Text(text) => text.key_mut(),
            Element::Empty(empty) => empty.key_mut(),
            Element::Component(component) => component.key_mut(),
        }
    }

    fn id_mut(&mut self) -> &mut u64 {
        match self {
            Element::Container(container) => container.id_mut(),
            Element::Text(text) => text.id_mut(),
            Element::Empty(empty) => empty.id_mut(),
            Element::Component(component) => component.id_mut(),
        }
    }

    fn draw(&mut self, renderer: &mut Box<dyn Renderer + Send>, render_context: &mut RenderContext) {
        match self {
            Element::Container(container) => container.draw(renderer, render_context),
            Element::Text(text) => text.draw(renderer, render_context),
            Element::Empty(empty) => empty.draw(render_context),
            Element::Component(component) => component.draw(renderer, render_context),
        }
    }

    fn debug_draw(&mut self, render_context: &mut RenderContext) {
        match self {
            Element::Container(container) => container.debug_draw(render_context),
            Element::Text(text) => text.debug_draw(render_context),
            Element::Empty(empty) => empty.debug_draw(render_context),
            Element::Component(component) => component.debug_draw(render_context),
        }
    }

    fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, font_system: &mut FontSystem) -> NodeId {
        match self {
            Element::Container(container) => container.compute_layout(taffy_tree, font_system),
            Element::Text(text) => text.compute_layout(taffy_tree, font_system),
            Element::Empty(empty) => empty.compute_layout(taffy_tree, font_system),
            Element::Component(component) => component.compute_layout(taffy_tree, font_system),
        }
    }

    fn finalize_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, root_node: NodeId, x: f32, y: f32) {
        match self {
            Element::Container(container) => container.finalize_layout(taffy_tree, root_node, x, y),
            Element::Text(text) => text.finalize_layout(taffy_tree, root_node, x, y),
            Element::Empty(_) => {}
            Element::Component(component) => component.finalize_layout(taffy_tree, root_node, x, y),
        }
    }

    fn computed_style(&self) -> Style {
        match self {
            Element::Container(container) => container.computed_style(),
            Element::Text(text) => text.computed_style(),
            Element::Empty(empty) => empty.computed_style(),
            Element::Component(component) => component.computed_style(),
        }
    }

    fn computed_style_mut(&mut self) -> &mut Style {
        match self {
            Element::Container(container) => container.computed_style_mut(),
            Element::Text(text) => text.computed_style_mut(),
            Element::Empty(empty) => empty.computed_style_mut(),
            Element::Component(component) => component.computed_style_mut(),
        }
    }

    fn in_bounds(&self, x: f32, y: f32) -> bool {
        match self {
            Element::Container(container) => container.in_bounds(x, y),
            Element::Text(text) => text.in_bounds(x, y),
            Element::Empty(empty) => empty.in_bounds(x, y),
            Element::Component(component) => component.in_bounds(x, y),
        }
    }
    fn add_update_handler(&mut self, update: Arc<fn(msg: Message, state: Box<dyn Any>, id: u64)>) {
        if let Element::Component(component) = self {
            component.add_update_handler(update);
        };
    }
}

impl Element {
    pub fn print_tree(&self) {
        let mut elements: Vec<(Self, usize, bool)> = vec![(self.clone(), 0, true)];
        let mut last_indent = 0;
        let mut last_was_last = true;

        while let Some((element, indent, is_last)) = elements.pop() {
            if indent > last_indent {
                print!("{: <1$}", "", (indent - last_indent) * 4);
            } else if indent < last_indent {
                print!("{: <1$}", "", (last_indent - indent) * 4);
            }

            // Print the prefix
            if indent == 0 {
                print!("");
            } else {
                if last_was_last {
                    print!("    ");
                } else {
                    print!("|   ");
                }

                for _ in 1..indent {
                    print!("|   ");
                }

                if is_last {
                    print!("└── ");
                } else {
                    print!("├── ");
                }
            }

            // Print the element
            println!(
                "{} - ID: {}, Key: {}",
                element.name(),
                element.id(),
                element.key().unwrap_or_else(|| "".to_string())
            );

            // Store the current indent and whether the current element is the last child
            last_indent = indent;
            last_was_last = is_last;

            // Add children to the stack in reverse order
            let children = element.children();
            for (i, child) in children.iter().enumerate().rev() {
                elements.push((child.clone(), indent + 1, i == 0));
            }
        }
    }
}


