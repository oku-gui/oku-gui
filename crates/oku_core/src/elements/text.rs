use std::any::Any;
use std::sync::Arc;
use crate::elements::layout_context::{CosmicTextContent, LayoutContext};
use crate::elements::element::Element;
use crate::elements::style::{AlignItems, Display, FlexDirection, JustifyContent, Style, Unit};
use crate::renderer::color::Color;
use crate::renderer::renderer::{Rectangle, Renderer};
use crate::RenderContext;
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics};
use taffy::{NodeId, TaffyTree};
use tiny_skia::{LineCap, LineJoin, Paint, PathBuilder, Rect};
use crate::elements::container::Container;
use crate::events::Message;
use crate::widget_id::create_unique_widget_id;

#[derive(Clone, Default, Debug)]
pub struct Text {
    key: Option<String>,
    tag: Option<String>,
    style: Style,
    children: Vec<Box<dyn Element>>,
    text: String,
    text_buffer: Option<Buffer>,
    computed_x: f32,
    computed_y: f32,
    computed_width: f32,
    computed_height: f32,
    computed_padding: [f32; 4],
    id: Option<String>,
    parent_component_id: u64,
}



impl Text {
    pub fn new(text: &str) -> Text {
        Text {
            key: None,
            tag: None,
            style: Style {
                ..Default::default()
            },
            children: vec![],
            text: text.to_string(),
            text_buffer: None,
            computed_x: 0.0,
            computed_y: 0.0,
            computed_width: 0.0,
            computed_height: 0.0,
            computed_padding: [0.0, 0.0, 0.0, 0.0],
            id: None,
            parent_component_id: 0,
        }
    }
}

impl Element for Text {
    fn children(&self) -> Vec<Box<dyn Element>> {
        Vec::new()
    }

    fn children_mut(&mut self) -> &mut Vec<Box<dyn Element>> {
        &mut self.children
    }

    fn name(&self) -> &'static str {
        "Text"
    }
    
    fn draw(&mut self, renderer: &mut Box<dyn Renderer + Send>, render_context: &mut RenderContext) {
        let text_color = cosmic_text::Color::rgba(self.style.color.r_u8(), self.style.color.g_u8(), self.style.color.b_u8(), self.style.color.a_u8());

        /*let mut paint = Paint {
            anti_alias: false,
            ..Default::default()
        };*/

        if self.text_buffer.is_none() {
            return;
        }

        let element_x = self.computed_x();
        let element_y = self.computed_y();
        let text_buffer = self.text_buffer.as_mut().unwrap();

        //paint.set_color_rgba8(self.style.background.r_u8(), self.style.background.g_u8(), self.style.background.b_u8(), self.style.background.a_u8());
        renderer.draw_rect(Rectangle::new(self.computed_x, self.computed_y, self.computed_width, self.computed_height), self.style.background);
        //render_context.canvas.fill_rect(Rect::from_xywh(self.computed_x, self.computed_y, self.computed_width, self.computed_height).unwrap(), &paint, Transform::identity(), None);

        text_buffer.draw(&mut render_context.font_system, &mut render_context.swash_cache, text_color, |x, y, w, h, color| {
            let r = color.r();
            let g = color.g();
            let b = color.b();
            let a = color.a();
            let a1 = a as f32 / 255.0;
            let a2 = self.style.color.a / 255.0;
            let a = (a1 * a2 * 255.0) as u8;

            //paint.set_color_rgba8(r, g, b, a);

            let p_x: i32 = (element_x + self.computed_padding[3] + x as f32) as i32;
            let p_y: i32 = (element_y + self.computed_padding[0] + y as f32) as i32;

            renderer.draw_rect(Rectangle::new(p_x as f32, p_y as f32, w as f32, h as f32), Color::new_from_rgba_u8(r, g, b, a));
            //render_context.canvas.fill_rect(Rect::from_xywh(p_x as f32, p_y as f32, w as f32, h as f32).unwrap(), &paint, Transform::identity(), None);
        });
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
        let font_size = self.style.font_size;
        let font_line_height = font_size * 1.2;
        let metrics = Metrics::new(font_size, font_line_height);
        let attrs = Attrs::new();

        let style: taffy::Style = self.style.into();

        taffy_tree.new_leaf_with_context(style, LayoutContext::Text(CosmicTextContent::new(metrics, self.text.as_str(), attrs, font_system))).unwrap()
    }

    fn finalize_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, root_node: NodeId, x: f32, y: f32) {
        let result = taffy_tree.layout(root_node).unwrap();
        let buffer = taffy_tree.get_node_context(root_node).unwrap();

        match buffer {
            LayoutContext::Text(cosmic_text) => self.text_buffer = Option::from(cosmic_text.buffer.clone()),
        }

        self.computed_x = x + result.location.x;
        self.computed_y = y + result.location.y;

        self.computed_width = result.size.width;
        self.computed_height = result.size.height;

        self.computed_padding = [result.padding.top, result.padding.right, result.padding.bottom, result.padding.left];
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

impl Text {
    pub fn add_child(self, _widget: Box<dyn Element>) -> Text {
        panic!("Text can't have children.");
    }

    // Styles
    pub const fn margin(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Text {
        self.style.margin = [top, right, bottom, left];
        self
    }
    pub const fn padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Text {
        self.style.padding = [top, right, bottom, left];
        self
    }

    pub const fn background(mut self, background: Color) -> Text {
        self.style.background = background;
        self
    }

    pub const fn color(mut self, color: Color) -> Text {
        self.style.color = color;
        self
    }

    pub const fn font_size(mut self, font_size: f32) -> Text {
        self.style.font_size = font_size;
        self
    }

    pub const fn display(mut self, display: Display) -> Text {
        self.style.display = display;
        self
    }

    pub const fn justify_content(mut self, justify_content: JustifyContent) -> Text {
        self.style.justify_content = Some(justify_content);
        self
    }

    pub const fn align_items(mut self, align_items: AlignItems) -> Text {
        self.style.align_items = Some(align_items);
        self
    }

    pub const fn flex_direction(mut self, flex_direction: FlexDirection) -> Text {
        self.style.flex_direction = flex_direction;
        self
    }

    pub const fn width(mut self, width: Unit) -> Text {
        self.style.width = width;
        self
    }

    pub const fn height(mut self, height: Unit) -> Text {
        self.style.height = height;
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

    pub fn computed_style(&self) -> Style {
        self.style
    }
    pub fn computed_style_mut(&mut self) -> &mut Style {
        &mut self.style
    }

    pub fn in_bounds(&self, x: f32, y: f32) -> bool {
        x >= self.computed_x && x <= self.computed_x + self.computed_width && y >= self.computed_y && y <= self.computed_y + self.computed_height
    }
}
