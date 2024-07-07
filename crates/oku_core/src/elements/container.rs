use crate::elements::element::Element;
use crate::elements::layout_context::LayoutContext;
use crate::elements::style::{AlignItems, Display, FlexDirection, JustifyContent, Style, Unit};
use crate::renderer::color::Color;
use crate::renderer::renderer::{Rectangle, Renderer};
use crate::RenderContext;
use cosmic_text::FontSystem;
use taffy::{NodeId, TaffyTree};

#[derive(Clone, Default, Debug)]
pub struct Container {
    style: Style,
    children: Vec<Box<dyn Element>>,
    computed_x: f32,
    computed_y: f32,
    computed_width: f32,
    computed_height: f32,
    computed_padding: [f32; 4],
    id: Option<String>,
    component_id: u64,
}

impl Container {
    pub fn new() -> Container {
        Container {
            style: Style {
                ..Default::default()
            },
            children: vec![],
            computed_x: 0.0,
            computed_y: 0.0,
            computed_width: 0.0,
            computed_height: 0.0,
            computed_padding: [0.0, 0.0, 0.0, 0.0],
            id: None,
            component_id: 0,
        }
    }
}

impl Element for Container {
    fn children(&self) -> Vec<Box<dyn Element>> {
        self.children.clone()
    }

    fn children_as_ref<'a>(&'a self) -> Vec<&'a dyn Element> {
        self.children.iter().map(|x| x.as_ref()).collect()
    }

    fn children_mut(&mut self) -> &mut Vec<Box<dyn Element>> {
        &mut self.children
    }

    fn name(&self) -> &'static str {
        "Container"
    }

    fn draw(&mut self, renderer: &mut Box<dyn Renderer + Send>, render_context: &mut RenderContext) {
        renderer.draw_rect(
            Rectangle::new(self.computed_x, self.computed_y, self.computed_width, self.computed_height),
            self.style.background,
        );

        for child in self.children.iter_mut() {
            child.draw(renderer, render_context);
        }
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

        taffy_tree.new_with_children(style, &child_nodes).unwrap()
    }

    fn finalize_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, root_node: NodeId, x: f32, y: f32) {
        let result = taffy_tree.layout(root_node).unwrap();

        self.computed_x = x + result.location.x;
        self.computed_y = y + result.location.y;

        self.computed_width = result.size.width;
        self.computed_height = result.size.height;

        self.computed_padding = [result.padding.top, result.padding.right, result.padding.bottom, result.padding.left];
        for (index, child) in self.children.iter_mut().enumerate() {
            let child2 = taffy_tree.child_at_index(root_node, index).unwrap();
            child.finalize_layout(taffy_tree, child2, self.computed_x, self.computed_y);
        }
    }

    fn computed_style(&self) -> Style {
        self.style
    }
    fn computed_style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    fn in_bounds(&self, x: f32, y: f32) -> bool {
        x >= self.computed_x
            && x <= self.computed_x + self.computed_width
            && y >= self.computed_y
            && y <= self.computed_y + self.computed_height
    }

    fn id(&self) -> &Option<String> {
        &self.id
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id;
    }

    fn component_id(&self) -> u64 {
        self.component_id
    }

    fn set_component_id(&mut self, id: u64) {
        self.component_id = id;
    }
}

impl Container {
    pub fn add_child(mut self, widget: Box<dyn Element>) -> Container {
        self.children.push(widget);
        self
    }

    pub const fn computed_x(&self) -> f32 {
        self.computed_x
    }

    pub const fn computed_y(&self) -> f32 {
        self.computed_y
    }

    pub const fn computed_width(&self) -> f32 {
        self.computed_width
    }

    pub const fn computed_height(&self) -> f32 {
        self.computed_height
    }
    pub const fn computed_padding(&self) -> [f32; 4] {
        self.computed_padding
    }

    // Styles
    pub const fn margin(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Container {
        self.style.margin = [top, right, bottom, left];
        self
    }
    pub const fn padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Container {
        self.style.padding = [top, right, bottom, left];
        self
    }

    pub const fn background(mut self, background: Color) -> Container {
        self.style.background = background;
        self
    }

    pub const fn display(mut self, display: Display) -> Container {
        self.style.display = display;
        self
    }

    pub const fn justify_content(mut self, justify_content: JustifyContent) -> Container {
        self.style.justify_content = Some(justify_content);
        self
    }

    pub const fn align_items(mut self, align_items: AlignItems) -> Container {
        self.style.align_items = Some(align_items);
        self
    }

    pub const fn flex_direction(mut self, flex_direction: FlexDirection) -> Container {
        self.style.flex_direction = flex_direction;
        self
    }

    pub const fn width(mut self, width: Unit) -> Container {
        self.style.width = width;
        self
    }

    pub const fn height(mut self, height: Unit) -> Container {
        self.style.height = height;
        self
    }
}
