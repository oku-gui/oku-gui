use crate::elements::element::Element;
use crate::elements::layout_context::{ImageContext, LayoutContext};
use crate::elements::style::{AlignItems, Display, FlexDirection, JustifyContent, Style, Unit, Weight};
use crate::renderer::color::Color;
use crate::renderer::renderer::{Rectangle, Renderer};
use crate::RenderContext;
use cosmic_text::FontSystem;
use taffy::{NodeId, TaffyTree};

#[derive(Clone, Default, Debug)]
pub struct Image {
    key: Option<String>,
    tag: Option<String>,
    style: Style,
    children: Vec<Box<dyn Element>>,
    image_path: String,
    computed_x: f32,
    computed_y: f32,
    computed_width: f32,
    computed_height: f32,
    computed_padding: [f32; 4],
    id: Option<String>,
    component_id: u64,
}

impl Image {
    pub fn new(image_path: &str) -> Image {
        Image {
            key: None,
            tag: None,
            style: Style {
                ..Default::default()
            },
            children: vec![],
            image_path: image_path.to_string(),
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

impl Element for Image {
    fn children(&self) -> Vec<Box<dyn Element>> {
        Vec::new()
    }

    fn children_as_ref<'a>(&'a self) -> Vec<&'a dyn Element> {
        Vec::new()
    }

    fn children_mut(&mut self) -> &mut Vec<Box<dyn Element>> {
        &mut self.children
    }

    fn name(&self) -> &'static str {
        "Image"
    }

    fn draw(&mut self, renderer: &mut Box<dyn Renderer + Send>, _render_context: &mut RenderContext) {
        renderer.draw_image(
            Rectangle::new(self.computed_x, self.computed_y, self.computed_width, self.computed_height),
            self.image_path.as_str(),
        );
    }

    fn debug_draw(&mut self, render_context: &mut RenderContext) {
    }

    fn compute_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, _font_system: &mut FontSystem) -> NodeId {
        let style: taffy::Style = self.style.into();

        taffy_tree
            .new_leaf_with_context(
                style,
                LayoutContext::Image(ImageContext {
                    width: 200.0,
                    height: 200.0,
                }),
            )
            .unwrap()
    }

    fn finalize_layout(&mut self, taffy_tree: &mut TaffyTree<LayoutContext>, root_node: NodeId, x: f32, y: f32) {
        let result = taffy_tree.layout(root_node).unwrap();

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

impl Image {
    pub fn add_child(self, _widget: Box<dyn Element>) -> Image {
        panic!("Text can't have children.");
    }

    // Styles
    pub const fn margin(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Image {
        self.style.margin = [top, right, bottom, left];
        self
    }
    pub const fn padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Image {
        self.style.padding = [top, right, bottom, left];
        self
    }

    pub const fn background(mut self, background: Color) -> Image {
        self.style.background = background;
        self
    }

    pub const fn color(mut self, color: Color) -> Image {
        self.style.color = color;
        self
    }

    pub const fn font_size(mut self, font_size: f32) -> Image {
        self.style.font_size = font_size;
        self
    }
    pub const fn font_weight(mut self, font_weight: Weight) -> Image {
        self.style.font_weight = font_weight;
        self
    }

    pub const fn display(mut self, display: Display) -> Image {
        self.style.display = display;
        self
    }

    pub const fn justify_content(mut self, justify_content: JustifyContent) -> Image {
        self.style.justify_content = Some(justify_content);
        self
    }

    pub const fn align_items(mut self, align_items: AlignItems) -> Image {
        self.style.align_items = Some(align_items);
        self
    }

    pub const fn flex_direction(mut self, flex_direction: FlexDirection) -> Image {
        self.style.flex_direction = flex_direction;
        self
    }

    pub const fn width(mut self, width: Unit) -> Image {
        self.style.width = width;
        self
    }

    pub const fn height(mut self, height: Unit) -> Image {
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
        x >= self.computed_x
            && x <= self.computed_x + self.computed_width
            && y >= self.computed_y
            && y <= self.computed_y + self.computed_height
    }
}
