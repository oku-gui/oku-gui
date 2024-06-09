use crate::elements::layout_context::LayoutContext;
use crate::elements::element::Element;
use crate::elements::style::{AlignItems, Display, FlexDirection, JustifyContent, Style, Unit};
use crate::renderer::color::Color;
use crate::renderer::renderer::{Rectangle, Renderer};
use crate::RenderContext;
use cosmic_text::FontSystem;
use std::any::Any;
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};
use tiny_skia::{Paint, PathBuilder, Rect};
use crate::events::Message;
use crate::widget_id::create_unique_widget_id;

#[derive(Clone, Default, Debug)]
pub struct Container {
    id: u64,
    key: Option<String>,
    style: Style,
    children: Vec<Box<dyn Element>>,
    computed_x: f32,
    computed_y: f32,
    computed_width: f32,
    computed_height: f32,
    computed_padding: [f32; 4],
}

impl Container {
    pub fn new() -> Container {
        Container {
            id: create_unique_widget_id(),
            key: None,
            style: Style {
                ..Default::default()
            },
            children: vec![],
            computed_x: 0.0,
            computed_y: 0.0,
            computed_width: 0.0,
            computed_height: 0.0,
            computed_padding: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

impl Element for Container {
    fn children(&self) -> Vec<Box<dyn Element>> {
        self.children.clone()
    }

    fn children_mut(&mut self) -> &mut Vec<Box<dyn Element>> {
        &mut self.children
    }

    fn name(&self) -> &'static str {
        "Container"
    }
    fn id(&self) -> u64 {
        self.id
    }

    fn key(&self) -> Option<String> {
        self.key.clone()
    }

    fn key_mut(&mut self) -> &mut Option<String> {
        &mut self.key
    }

    fn id_mut(&mut self) -> &mut u64 {
        &mut self.id
    }

    fn draw(&mut self, renderer: &mut Box<dyn Renderer + Send>, render_context: &mut RenderContext) {
        let mut paint = Paint::default();
        paint.set_color_rgba8(self.style.background.r_u8(), self.style.background.g_u8(), self.style.background.b_u8(), self.style.background.a_u8());
        paint.anti_alias = true;

        renderer.draw_rect(Rectangle::new(self.computed_x, self.computed_y, self.computed_width, self.computed_height), self.style.background);
        //render_context.canvas.fill_rect(Rect::from_xywh(self.computed_x, self.computed_y, self.computed_width, self.computed_height).unwrap(), &paint, Transform::identity(), None);

        for child in self.children.iter_mut() {
            child.draw(renderer, render_context);
        }
    }

    fn debug_draw(&mut self, render_context: &mut RenderContext) {
        let mut paint = Paint::default();
        paint.set_color_rgba8(0, 0, 0, 255);
        paint.anti_alias = true;

        let mut path_builder = PathBuilder::new();
        path_builder.push_rect(Rect::from_xywh(self.computed_x, self.computed_y, self.computed_width, self.computed_height).unwrap());
        path_builder.finish().unwrap();

        //render_context.canvas.stroke_path(&path, &paint, &stroke, Transform::identity(), None);

        for child in self.children.iter_mut() {
            child.debug_draw(render_context);
        }
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
        x >= self.computed_x && x <= self.computed_x + self.computed_width && y >= self.computed_y && y <= self.computed_y + self.computed_height
    }

    fn add_update_handler(&mut self, update: Arc<fn(Message, Box<dyn Any>, id: u64)>) {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        todo!()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        todo!()
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
