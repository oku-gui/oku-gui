use crate::components::component::{ComponentId, ComponentSpecification};
use crate::components::UpdateResult;
use crate::elements::element::{CommonElementData, Element, ElementBox};
use crate::elements::layout_context::{
    AvailableSpace, LayoutContext, MetricsDummy, TaffyTextInputContext, TextHashKey,
};
use crate::elements::text::TextHashValue;
use crate::engine::renderer::color::Color;
use crate::engine::renderer::renderer::Rectangle;
use crate::reactive::state_store::StateStore;
use crate::style::{AlignItems, Display, FlexDirection, FontStyle, JustifyContent, Unit, Weight};
use crate::RendererBox;
use cosmic_text::{Action, Motion};
use cosmic_text::{Attrs, Editor, FontSystem, Metrics};
use cosmic_text::{Edit};
use rustc_hash::FxHasher;
use std::any::Any;
use std::collections::HashMap;
use std::hash::Hasher;
use taffy::{NodeId, Size, TaffyTree};
use winit::event::KeyEvent;
use winit::keyboard::{Key, NamedKey};
use crate::engine::events::{OkuMessage};

// A stateful element that shows text.
#[derive(Clone, Default, Debug)]
pub struct TextInput {
    text: String,
    common_element_data: CommonElementData,
}

pub struct TextInputState<'a> {
    pub id: ComponentId,
    pub text_hash: u64,
    pub cached_text_layout: HashMap<TextHashKey, TextHashValue>,
    pub last_key: TextHashKey,
    pub metrics: Metrics,
    pub editor: Editor<'a>,
    pub text: String,
}

impl<'a> TextInputState<'a> {
    pub(crate) fn new(
        id: ComponentId,
        metrics: Metrics,
        text_hash: u64,
        editor: Editor<'a>,
        _color: Option<cosmic_text::Color>,
        text: String,
    ) -> Self {
        Self {
            id,
            text_hash,
            cached_text_layout: Default::default(),
            last_key: TextHashKey {
                text_hash,
                width_constraint: None,
                height_constraint: None,
                available_space_width: AvailableSpace::MinContent,
                available_space_height: AvailableSpace::MinContent,
                metrics: MetricsDummy {
                    font_size: metrics.font_size.to_bits(),
                    line_height: metrics.line_height.to_bits(),
                },
            },
            metrics,
            editor,
            text,
        }
    }

    pub(crate) fn measure(
        &mut self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<taffy::AvailableSpace>,
        font_system: &mut FontSystem,
        text_hash: u64,
        metrics: Metrics,
    ) -> Size<f32> {
        // Set width constraint
        let width_constraint = known_dimensions.width.or(match available_space.width {
            taffy::AvailableSpace::MinContent => Some(0.0),
            taffy::AvailableSpace::MaxContent => None,
            taffy::AvailableSpace::Definite(width) => Some(width),
        });

        let height_constraint = known_dimensions.height;

        let available_space_width_u32: AvailableSpace = match available_space.width {
            taffy::AvailableSpace::MinContent => AvailableSpace::MinContent,
            taffy::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
            taffy::AvailableSpace::Definite(width) => AvailableSpace::Definite(width.to_bits()),
        };
        let available_space_height_u32: AvailableSpace = match available_space.height {
            taffy::AvailableSpace::MinContent => AvailableSpace::MinContent,
            taffy::AvailableSpace::MaxContent => AvailableSpace::MaxContent,
            taffy::AvailableSpace::Definite(height) => AvailableSpace::Definite(height.to_bits()),
        };

        let key = TextHashKey {
            text_hash,
            width_constraint: width_constraint.map(|w| w.to_bits()),
            height_constraint: height_constraint.map(|h| h.to_bits()),
            available_space_width: available_space_width_u32,
            available_space_height: available_space_height_u32,
            metrics: MetricsDummy {
                font_size: metrics.font_size.to_bits(),
                line_height: metrics.line_height.to_bits(),
            },
        };

        self.last_key = key;
        let cached_text_layout_value = self.cached_text_layout.get(&key);
        self.text_hash = text_hash;

        if cached_text_layout_value.is_none() {
            self.editor.with_buffer_mut(|buffer| {
                buffer.set_metrics(font_system, self.metrics);
                buffer.set_size(font_system, width_constraint, height_constraint);
            });
            self.editor.shape_as_needed(font_system, true);

            // Determine measured size of text
            let cached_text_layout_value = self.editor.with_buffer(|buffer| {
                let (width, total_lines) = buffer
                    .layout_runs()
                    .fold((0.0, 0usize), |(width, total_lines), run| (run.line_w.max(width), total_lines + 1));
                let height = total_lines as f32 * buffer.metrics().line_height;

                TextHashValue {
                    computed_width: width,
                    computed_height: height,
                }
            });

            self.cached_text_layout.insert(key, cached_text_layout_value);
            Size {
                width: cached_text_layout_value.computed_width,
                height: cached_text_layout_value.computed_height,
            }
        } else {
            let cached_text_layout_value = cached_text_layout_value.unwrap();
            Size {
                width: cached_text_layout_value.computed_width,
                height: cached_text_layout_value.computed_height,
            }
        }
    }
}

impl TextInput {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            common_element_data: Default::default(),
        }
    }

    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a StateStore) -> &'a TextInputState {
        element_state.storage.get(&self.common_element_data.component_id).unwrap().as_ref().downcast_ref().unwrap()
    }
}

impl Element for TextInput {
    fn common_element_data(&self) -> &CommonElementData {
        &self.common_element_data
    }

    fn common_element_data_mut(&mut self) -> &mut CommonElementData {
        &mut self.common_element_data
    }

    fn children_mut(&mut self) -> &mut Vec<ElementBox> {
        &mut self.common_element_data.children
    }

    fn name(&self) -> &'static str {
        "TextInput"
    }

    fn draw(
        &mut self,
        renderer: &mut RendererBox,
        _font_system: &mut FontSystem,
        _taffy_tree: &mut TaffyTree<LayoutContext>,
        _root_node: NodeId,
        element_state: &StateStore,
    ) {
        let text_input_context: &TextInputState = element_state
            .storage
            .get(&self.common_element_data.component_id)
            .unwrap()
            .as_ref()
            .downcast_ref()
            .unwrap();

        
        let bounding_rectangle = Rectangle::new(
            self.common_element_data.computed_x_transformed + self.common_element_data.computed_padding[3],
            self.common_element_data.computed_y_transformed + self.common_element_data.computed_padding[0],
            self.common_element_data.computed_width,
            self.common_element_data.computed_height,
        );
        renderer.draw_rect(bounding_rectangle, self.common_element_data.style.background);

        renderer.draw_text(
            self.common_element_data.component_id,
            bounding_rectangle,
            self.common_element_data.style.color,
        );

        if let Some((x, y)) = text_input_context.editor.cursor_position() {
            // HACK: Draw the cursor here for now.
            renderer.draw_rect(Rectangle::new(
                bounding_rectangle.x + x as f32,
                bounding_rectangle.y + y as f32,
                1.5,
                text_input_context.metrics.line_height
            ), Color::BLACK);
        }
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _font_system: &mut FontSystem,
        element_state: &mut StateStore,
    ) -> NodeId {
        let (text_hash, _text) = if let Some(state) = element_state
            .storage
            .get_mut(&self.common_element_data.component_id)
            .unwrap()
            .as_mut()
            .downcast_mut::<TextInputState>()
        {
            (state.text_hash, state.text.clone())
        } else {
            let mut text_hasher = FxHasher::default();
            text_hasher.write(self.text.as_ref());
            (text_hasher.finish(), self.text.clone())
        };

        let font_size = self.common_element_data.style.font_size;
        let font_line_height = font_size * 1.2;
        let metrics = Metrics::new(font_size, font_line_height);
        let mut attributes = Attrs::new();
        attributes = attributes.style(match self.common_element_data.style.font_style {
            FontStyle::Normal => cosmic_text::Style::Normal,
            FontStyle::Italic => cosmic_text::Style::Italic,
            FontStyle::Oblique => cosmic_text::Style::Oblique,
        });

        attributes.weight = cosmic_text::Weight(self.common_element_data.style.font_weight.0);
        let style: taffy::Style = self.common_element_data.style.into();

        taffy_tree
            .new_leaf_with_context(
                style,
                LayoutContext::TextInput(TaffyTextInputContext::new(
                    self.common_element_data.component_id,
                    metrics,
                    self.text.clone(),
                    text_hash,
                    attributes,
                )),
            )
            .unwrap()
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        x: f32,
        y: f32,
        transform: glam::Mat4,
        font_system: &mut FontSystem,
        element_state: &mut StateStore,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        
        let text_context: &mut TextInputState = element_state
            .storage
            .get_mut(&self.common_element_data.component_id)
            .unwrap()
            .as_mut()
            .downcast_mut()
            .unwrap();

        text_context.editor.with_buffer_mut(|buffer| {
            buffer.set_metrics(font_system, text_context.metrics);

            buffer.set_size(
                font_system,
                text_context.last_key.width_constraint.map(|w| f32::from_bits(w)),
                text_context.last_key.height_constraint.map(|h| f32::from_bits(h)),
            );
            buffer.shape_until_scroll(font_system, true);
        });

        self.common_element_data.computed_x = x + result.location.x;
        self.common_element_data.computed_y = y + result.location.y;
        self.common_element_data.computed_width = result.size.width;
        self.common_element_data.computed_height = result.size.height;
        self.common_element_data.computed_padding =
            [result.padding.top, result.padding.right, result.padding.bottom, result.padding.left];

        let transformed_xy = transform.mul_vec4(glam::vec4(
            self.common_element_data.computed_x,
            self.common_element_data.computed_y,
            0.0,
            1.0,
        ));
        self.common_element_data.computed_x_transformed = transformed_xy.x;
        self.common_element_data.computed_y_transformed = transformed_xy.y;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_event(&self, message: OkuMessage, element_state: &mut StateStore, font_system: &mut FontSystem) -> UpdateResult {
        let text_context: &mut TextInputState = element_state
            .storage
            .get_mut(&self.common_element_data.component_id)
            .unwrap()
            .as_mut()
            .downcast_mut()
            .unwrap();

        let res = match message {
            OkuMessage::KeyboardInputEvent(keyboard_input) => {
                let KeyEvent {
                    logical_key, state, ..
                } = keyboard_input.event;

                if state.is_pressed() {
                    match logical_key {
                        Key::Named(NamedKey::ArrowLeft) => {
                            text_context.editor.action(font_system, Action::Motion(Motion::Left))
                        }
                        Key::Named(NamedKey::ArrowRight) => {
                            text_context.editor.action(font_system, Action::Motion(Motion::Right))
                        }
                        Key::Named(NamedKey::ArrowUp) => {
                            text_context.editor.action(font_system, Action::Motion(Motion::Up))
                        }
                        Key::Named(NamedKey::ArrowDown) => {
                            text_context.editor.action(font_system, Action::Motion(Motion::Down))
                        }
                        Key::Named(NamedKey::Home) => {
                            text_context.editor.action(font_system, Action::Motion(Motion::Home))
                        }
                        Key::Named(NamedKey::End) => {
                            text_context.editor.action(font_system, Action::Motion(Motion::End))
                        }
                        Key::Named(NamedKey::PageUp) => {
                            text_context.editor.action(font_system, Action::Motion(Motion::PageUp))
                        }
                        Key::Named(NamedKey::PageDown) => {
                            text_context.editor.action(font_system, Action::Motion(Motion::PageDown))
                        }
                        Key::Named(NamedKey::Escape) => text_context.editor.action(font_system, Action::Escape),
                        Key::Named(NamedKey::Enter) => text_context.editor.action(font_system, Action::Enter),
                        Key::Named(NamedKey::Backspace) => text_context.editor.action(font_system, Action::Backspace),
                        Key::Named(NamedKey::Delete) => text_context.editor.action(font_system, Action::Delete),
                        Key::Named(key) => {
                            if let Some(text) = key.to_text() {
                                for char in text.chars() {
                                    text_context.editor.action(font_system, Action::Insert(char));
                                }
                            }
                        }
                        Key::Character(text) => {
                            for c in text.chars() {
                                text_context.editor.action(font_system, Action::Insert(c));
                            }
                        }
                        _ => {}
                    }
                }
                text_context.editor.shape_as_needed(font_system, true);

                text_context.editor.with_buffer(|buffer| {
                    let mut buffer_string: String = String::new();
                    let last_line = buffer.lines.len() - 1;
                    for (line_number, line) in buffer.lines.iter().enumerate() {
                        buffer_string.push_str(line.text());
                        if line_number != last_line {
                            buffer_string.push_str("\n");
                        }
                    }

                    let mut text_hasher = FxHasher::default();
                    text_hasher.write(buffer_string.as_bytes());
                    let text_hash = text_hasher.finish();

                    text_context.text_hash = text_hash;
                    text_context.text = buffer_string.clone();

                    UpdateResult::new()
                        .prevent_defaults()
                        .prevent_propagate()
                        .result_message(OkuMessage::TextInputChanged(buffer_string))
                })
            }
            _ => UpdateResult::new(),
        };

        res
    }
}

impl TextInput {
    // Styles
    pub const fn margin(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.common_element_data.style.margin = [top, right, bottom, left];
        self
    }
    pub const fn padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.common_element_data.style.padding = [top, right, bottom, left];
        self
    }

    pub const fn background(mut self, background: Color) -> Self {
        self.common_element_data.style.background = background;
        self
    }

    pub const fn color(mut self, color: Color) -> Self {
        self.common_element_data.style.color = color;
        self
    }

    pub const fn font_size(mut self, font_size: f32) -> Self {
        self.common_element_data.style.font_size = font_size;
        self
    }
    pub const fn font_weight(mut self, font_weight: Weight) -> Self {
        self.common_element_data.style.font_weight = font_weight;
        self
    }

    pub const fn font_style(mut self, font_style: FontStyle) -> Self {
        self.common_element_data.style.font_style = font_style;
        self
    }

    pub const fn display(mut self, display: Display) -> Self {
        self.common_element_data.style.display = display;
        self
    }

    pub const fn justify_content(mut self, justify_content: JustifyContent) -> Self {
        self.common_element_data.style.justify_content = Some(justify_content);
        self
    }

    pub const fn align_items(mut self, align_items: AlignItems) -> Self {
        self.common_element_data.style.align_items = Some(align_items);
        self
    }

    pub const fn flex_direction(mut self, flex_direction: FlexDirection) -> Self {
        self.common_element_data.style.flex_direction = flex_direction;
        self
    }

    pub const fn width(mut self, width: Unit) -> Self {
        self.common_element_data.style.width = width;
        self
    }

    pub const fn height(mut self, height: Unit) -> Self {
        self.common_element_data.style.height = height;
        self
    }

    pub fn id(mut self, id: &str) -> Self {
        self.common_element_data.id = Some(id.to_string());
        self
    }

    pub fn component(self) -> ComponentSpecification {
        ComponentSpecification::new(self.into())
    }
}
