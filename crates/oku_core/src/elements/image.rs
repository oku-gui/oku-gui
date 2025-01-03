use crate::components::component::ComponentSpecification;
use crate::elements::element::{CommonElementData, Element};
use crate::elements::layout_context::{ImageContext, LayoutContext};
use crate::renderer::color::Color;
use crate::resource_manager::ResourceIdentifier;
use crate::reactive::state_store::StateStore;
use crate::style::{AlignItems, Display, FlexDirection, JustifyContent, Unit, Weight};
use crate::{generate_component_methods_no_children, RendererBox};
use crate::components::props::Props;
use cosmic_text::FontSystem;
use std::any::Any;
use taffy::{NodeId, TaffyTree};
use crate::geometry::{Border, ElementRectangle, Margin, Padding, Size};

#[derive(Clone, Debug)]
pub struct Image {
    pub(crate) resource_identifier: ResourceIdentifier,
    pub common_element_data: CommonElementData,
}

impl Image {
    pub fn new(resource_identifier: ResourceIdentifier) -> Image {
        Image {
            resource_identifier,
            common_element_data: Default::default(),
        }
    }

    pub fn name() -> &'static str {
        "Image"
    }
}

impl Element for Image {
    fn common_element_data(&self) -> &CommonElementData {
        &self.common_element_data
    }

    fn common_element_data_mut(&mut self) -> &mut CommonElementData {
        &mut self.common_element_data
    }

    fn name(&self) -> &'static str {
        "Image"
    }

    fn draw(
        &mut self,
        renderer: &mut RendererBox,
        _font_system: &mut FontSystem,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        _element_state: &StateStore,
    ) {
        self.draw_borders(renderer);
        
        let computed_layer_rectangle_transformed = self.common_element_data.computed_layered_rectangle_transformed.clone();
        let content_rectangle = computed_layer_rectangle_transformed.content_rectangle();
        
        renderer.draw_image(
            content_rectangle,
            self.resource_identifier.clone(),
        );
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _font_system: &mut FontSystem,
        _element_state: &mut StateStore,
        scale_factor: f64,
    ) -> NodeId {
        let style: taffy::Style = self.common_element_data.style.to_taffy_style_with_scale_factor(scale_factor);
        
        taffy_tree
            .new_leaf_with_context(
                style,
                LayoutContext::Image(ImageContext {
                    resource_identifier: self.resource_identifier.clone(),
                }),
            )
            .unwrap()
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        x: f32,
        y: f32,
        layout_order: &mut u32,
        transform: glam::Mat4,
        _font_system: &mut FontSystem,
        _element_state: &mut StateStore,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_layer_rectangle(x, y, transform, result, layout_order);

        self.finalize_borders();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Image {
    // Styles
    pub const fn margin(mut self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Image {
        self.common_element_data.style.margin = [top, right, bottom, left];
        self
    }
    pub const fn padding(mut self, top: Unit, right: Unit, bottom: Unit, left: Unit) -> Image {
        self.common_element_data.style.padding = [top, right, bottom, left];
        self
    }

    pub const fn background(mut self, background: Color) -> Image {
        self.common_element_data.style.background = background;
        self
    }

    pub const fn color(mut self, color: Color) -> Image {
        self.common_element_data.style.color = color;
        self
    }

    pub const fn font_size(mut self, font_size: f32) -> Image {
        self.common_element_data.style.font_size = font_size;
        self
    }
    pub const fn font_weight(mut self, font_weight: Weight) -> Image {
        self.common_element_data.style.font_weight = font_weight;
        self
    }

    pub const fn display(mut self, display: Display) -> Image {
        self.common_element_data.style.display = display;
        self
    }

    pub const fn justify_content(mut self, justify_content: JustifyContent) -> Image {
        self.common_element_data.style.justify_content = Some(justify_content);
        self
    }

    pub const fn align_items(mut self, align_items: AlignItems) -> Image {
        self.common_element_data.style.align_items = Some(align_items);
        self
    }

    pub const fn flex_direction(mut self, flex_direction: FlexDirection) -> Image {
        self.common_element_data.style.flex_direction = flex_direction;
        self
    }

    pub const fn width(mut self, width: Unit) -> Image {
        self.common_element_data.style.width = width;
        self
    }

    pub const fn height(mut self, height: Unit) -> Image {
        self.common_element_data.style.height = height;
        self
    }

    pub const fn max_width(mut self, max_width: Unit) -> Image {
        self.common_element_data.style.max_width = max_width;
        self
    }

    pub const fn max_height(mut self, max_height: Unit) -> Image {
        self.common_element_data.style.max_height = max_height;
        self
    }

    pub fn id(mut self, id: &str) -> Self {
        self.common_element_data.id = Some(id.to_string());
        self
    }

    generate_component_methods_no_children!();
}
