use crate::elements::layout_context::LayoutContext;
use crate::elements::style::Style;
use crate::renderer::renderer::Renderer;
use crate::RenderContext;
use cosmic_text::FontSystem;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};
use crate::events::Message;

pub trait Element: Any + StandardElementClone + Debug + Send {
    fn children(&self) -> Vec<Box<dyn Element>>;

    fn children_mut(&mut self) -> &mut Vec<Box<dyn Element>>;

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

    fn as_any(&self) -> &dyn Any;
    
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl dyn Element {

    pub fn print_tree(&self) {
        let mut elements: Vec<(Box<Self>, usize, bool)> = vec![(self.clone_box(), 0, true)];
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

pub trait StandardElementClone {
    fn clone_box(&self) -> Box<dyn Element>;
}

impl<T> StandardElementClone for T
    where
        T: Element + Clone,
{
    fn clone_box(&self) -> Box<dyn Element> {
        Box::new(self.clone())
    }
}

// We can now implement Clone manually by forwarding to clone_box.
impl Clone for Box<dyn Element> {
    fn clone(&self) -> Box<dyn Element> {
        self.clone_box()
    }
}