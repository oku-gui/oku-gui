use crate::components::component::{ComponentId, ComponentSpecification};
use crate::components::props::Props;
use crate::components::UpdateResult;
use crate::elements::element::{CommonElementData, Element, ElementBox};
use crate::elements::layout_context::{
    AvailableSpace, LayoutContext, MetricsDummy, TaffyTextInputContext, TextHashKey,
};
use crate::elements::text::{TextHashValue, TextState};
use crate::elements::ElementStyles;
use crate::events::OkuMessage;
use crate::reactive::state_store::{StateStore, StateStoreItem};
use crate::renderer::color::Color;
use crate::renderer::renderer::Rectangle;
use crate::style::{FontStyle, Style};
use crate::{generate_component_methods_no_children, RendererBox};
use cosmic_text::Edit;
use cosmic_text::{Action, Buffer, Cursor, Motion, Selection, Shaping};
use cosmic_text::{Attrs, Editor, FontSystem, Metrics};
use rustc_hash::FxHasher;
use std::any::Any;
use std::collections::HashMap;
use std::hash::Hasher;
use taffy::{NodeId, TaffyTree};
use winit::dpi::{LogicalPosition, PhysicalPosition};
use winit::event::KeyEvent;
use winit::keyboard::{Key, NamedKey};
use crate::geometry::{Border, LayeredRectangle, Margin, Padding, Position, Size};

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
    pub editor: Editor<'a>,
    pub text: String,
    pub original_text_hash: u64,
}

impl<'a> TextInputState<'a> {
    pub(crate) fn new(
        id: ComponentId,
        metrics: Metrics,
        text_hash: u64,
        editor: Editor<'a>,
        text: String,
        original_text_hash: u64,
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
            editor,
            text,
            original_text_hash,
        }
    }

    pub(crate) fn measure(
        &mut self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        font_system: &mut FontSystem,
        text_hash: u64,
        metrics: Metrics,
    ) -> taffy::Size<f32> {
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
                buffer.set_metrics(font_system, metrics);
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
            taffy::Size {
                width: cached_text_layout_value.computed_width,
                height: cached_text_layout_value.computed_height,
            }
        } else {
            let cached_text_layout_value = cached_text_layout_value.unwrap();
            taffy::Size {
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
        _element_state: &StateStore,
    ) {
        let computed_layer_rectangle_transformed = self.common_element_data.computed_layered_rectangle_transformed.clone();
        let border_rectangle = computed_layer_rectangle_transformed.border_rectangle();
        let content_rectangle = computed_layer_rectangle_transformed.content_rectangle();

        renderer.draw_rect(border_rectangle, self.common_element_data.style.background);
        renderer.draw_text(
            self.common_element_data.component_id,
            content_rectangle,
            self.common_element_data.style.color,
        );
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _font_system: &mut FontSystem,
        element_state: &mut StateStore,
        scale_factor: f64,
    ) -> NodeId {
        let font_size = PhysicalPosition::from_logical(
            LogicalPosition::new(self.common_element_data.style.font_size, self.common_element_data.style.font_size),
            scale_factor,
        ).x;
        
        let font_line_height = font_size * 1.2;
        let metrics = Metrics::new(font_size, font_line_height);

        let style: taffy::Style = self.common_element_data.style.to_taffy_style_with_scale_factor(scale_factor);

        taffy_tree
            .new_leaf_with_context(
                style,
                LayoutContext::TextInput(TaffyTextInputContext::new(self.common_element_data.component_id, metrics)),
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
        font_system: &mut FontSystem,
        element_state: &mut StateStore,
    ) {

        let state: &mut TextInputState = element_state
            .storage
            .get_mut(&self.common_element_data.component_id)
            .unwrap()
            .as_mut()
            .downcast_mut()
            .unwrap();

        let metrics = Metrics::new(
            f32::from_bits(state.last_key.metrics.font_size),
            f32::from_bits(state.last_key.metrics.line_height),
        );

        state.editor.with_buffer_mut(|buffer| {
            buffer.set_metrics(font_system, metrics);

            buffer.set_size(
                font_system,
                state.last_key.width_constraint.map(|w| f32::from_bits(w)),
                state.last_key.height_constraint.map(|h| f32::from_bits(h)),
            );
            buffer.shape_until_scroll(font_system, true);
        });

        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_layer_rectangle(x, y, transform, result, layout_order);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_event(
        &self,
        message: OkuMessage,
        element_state: &mut StateStore,
        font_system: &mut FontSystem,
    ) -> UpdateResult {
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

                                //text_context.editor.set_selection(Selection::Line(Cursor::new(0, 0)));
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

    fn initialize_state(&self, font_system: &mut FontSystem) -> Box<StateStoreItem> {
        let font_size = self.common_element_data.style.font_size;
        let font_line_height = font_size * 1.2;
        let metrics = Metrics::new(font_size, font_line_height);

        let attributes = Attrs::new();

        let buffer = Buffer::new(font_system, metrics);
        let mut editor = Editor::new(buffer);
        editor.borrow_with(font_system);

        let mut text_hasher = FxHasher::default();
        text_hasher.write(self.text.as_ref());
        let text_hash = text_hasher.finish();

        editor.with_buffer_mut(|buffer| buffer.set_text(font_system, &self.text, attributes, Shaping::Advanced));
        editor.action(font_system, Action::Motion(Motion::End));

        let cosmic_text_content = TextInputState::new(
            self.common_element_data.component_id,
            metrics,
            text_hash,
            editor,
            self.text.clone(),
            text_hash,
        );

        Box::new(cosmic_text_content)
    }

    fn update_state(&self, font_system: &mut FontSystem, element_state: &mut StateStore) {
        let state: &mut TextInputState = element_state
            .storage
            .get_mut(&self.common_element_data.component_id)
            .unwrap()
            .as_mut()
            .downcast_mut()
            .unwrap();

        let font_size = self.common_element_data.style.font_size;
        let font_line_height = font_size * 1.2;
        let metrics = Metrics::new(font_size, font_line_height);

        let mut text_hasher = FxHasher::default();
        text_hasher.write(self.text.as_ref());
        let text_hash = text_hasher.finish();

        let attributes = Attrs::new();

        if text_hash != state.original_text_hash {
            state.original_text_hash = text_hash;
            state.text_hash = text_hash;
            state.text = self.text.clone();

            state.editor.with_buffer_mut(|buffer| {
                buffer.set_text(font_system, &self.text, attributes, Shaping::Advanced);
            });
        }
    }
}

impl TextInput {
    pub fn id(mut self, id: &str) -> Self {
        self.common_element_data.id = Some(id.to_string());
        self
    }

    generate_component_methods_no_children!();
}

impl ElementStyles for TextInput {
    fn styles_mut(&mut self) -> &mut Style {
        &mut self.common_element_data.style
    }
}
