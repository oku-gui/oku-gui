use std::any::Any;
use crate::user::elements::element::{CommonElementData, Element};
use crate::user::elements::layout_context::LayoutContext;
use crate::user::elements::style::{AlignItems, Display, FlexDirection, JustifyContent, Style, Unit};
use crate::engine::renderer::color::Color;
use crate::engine::renderer::renderer::{Rectangle, Renderer};
use crate::RenderContext;
use cosmic_text::FontSystem;
use taffy::{NodeId, TaffyTree};

#[derive(Clone, Default, Debug)]
pub struct Container {
    pub common_element_data: CommonElementData
}

impl Container {
    pub fn new() -> Container {
        Container {
            common_element_data: Default::default(),
        }
    }
}

impl Element for Container {
    fn common_element_data(&self) -> &CommonElementData {
        &self.common_element_data
    }

    fn common_element_data_mut(&mut self) -> &mut CommonElementData {
        &mut self.common_element_data
    }
    
    fn name(&self) -> &'static str {
        "Container"
    }

    fn draw(&mut self, renderer: &mut Box<dyn Renderer + Send>, render_context: &mut RenderContext) {
        renderer.draw_rect(
            Rectangle::new(self.common_element_data.computed_x, self.common_element_data.computed_y, self.common_element_data.computed_width, self.common_element_data.computed_height),
            self.common_element_data.style.background,
        );

        for child in self.common_element_data.children.iter_mut() {
            child.draw(renderer, render_context);
        }
    }

    fn debug_draw(&mut self, _render_context: &mut RenderContext) {
    }

    fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, font_system: &mut FontSystem) -> NodeId {
        let mut child_nodes: Vec<NodeId> = Vec::with_capacity(self.children().len());

        for child in self.common_element_data.children.iter_mut() {
            let child_node = child.compute_layout(taffy_tree, font_system);
            child_nodes.push(child_node);
        }

        let style: taffy::Style = self.common_element_data.style.into();

        taffy_tree.new_with_children(style, &child_nodes).unwrap()
    }

    fn finalize_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, root_node: NodeId, x: f32, y: f32) {
        let result = taffy_tree.layout(root_node).unwrap();

        self.common_element_data.computed_x = x + result.location.x;
        self.common_element_data.computed_y = y + result.location.y;

        self.common_element_data.computed_width = result.size.width;
        self.common_element_data.computed_height = result.size.height;

        self.common_element_data.computed_padding = [result.padding.top, result.padding.right, result.padding.bottom, result.padding.left];
        for (index, child) in self.common_element_data.children.iter_mut().enumerate() {
            let child2 = taffy_tree.child_at_index(root_node, index).unwrap();
            child.finalize_layout(taffy_tree, child2, self.common_element_data.computed_x, self.common_element_data.computed_y);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Container {
    pub fn add_child(mut self, widget: Box<dyn Element>) -> Container {
        self.common_element_data.children.push(widget);
        self
    }

    // Styles
    pub const fn margin(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Container {
        self.common_element_data.style.margin = [top, right, bottom, left];
        self
    }
    pub const fn padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Container {
        self.common_element_data.style.padding = [top, right, bottom, left];
        self
    }

    pub const fn background(mut self, background: Color) -> Container {
        self.common_element_data.style.background = background;
        self
    }

    pub const fn display(mut self, display: Display) -> Container {
        self.common_element_data.style.display = display;
        self
    }

    pub const fn justify_content(mut self, justify_content: JustifyContent) -> Container {
        self.common_element_data.style.justify_content = Some(justify_content);
        self
    }

    pub const fn align_items(mut self, align_items: AlignItems) -> Container {
        self.common_element_data.style.align_items = Some(align_items);
        self
    }

    pub const fn flex_direction(mut self, flex_direction: FlexDirection) -> Container {
        self.common_element_data.style.flex_direction = flex_direction;
        self
    }

    pub const fn width(mut self, width: Unit) -> Container {
        self.common_element_data.style.width = width;
        self
    }

    pub const fn height(mut self, height: Unit) -> Container {
        self.common_element_data.style.height = height;
        self
    }
}
