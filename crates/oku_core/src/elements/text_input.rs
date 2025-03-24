use crate::components::component::ComponentSpecification;
use crate::components::Props;
use crate::components::UpdateResult;
use crate::elements::cached_editor::CachedEditor;
use crate::elements::common_element_data::CommonElementData;
use crate::elements::element::{Element, ElementBox};
use crate::elements::layout_context::{LayoutContext, TaffyTextInputContext};
use crate::elements::scroll_state::ScrollState;
use crate::elements::ElementStyles;
use crate::events::OkuMessage;
use crate::geometry::{Point, Size};
use crate::reactive::element_state_store::{ElementStateStore, ElementStateStoreItem};
use crate::renderer::color::Color;
use crate::renderer::renderer::TextScroll;
use crate::style::{Display, Style, Unit};
use crate::{generate_component_methods_no_children, RendererBox};
use cosmic_text::{Action, Motion};
use cosmic_text::{Change, Cursor, Edit, Editor};
use cosmic_text::FontSystem;
use std::any::Any;
use std::sync::Arc;
use taffy::{NodeId, TaffyTree};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::Ime;
use winit::keyboard::{Key, NamedKey};
use winit::window::Window;

// A stateful element that shows text.
#[derive(Clone, Default, Debug)]
pub struct TextInput {
    text: String,
    common_element_data: CommonElementData,
}

pub struct TextInputState<'a> {
    pub cached_editor: CachedEditor<'a>,
    pub dragging: bool,
    pub is_ime_active: bool,
    pub is_active: bool,
    pub ime_starting_cursor: Option<Cursor>,
    pub ime_ending_cursor: Option<Cursor>,
    pub(crate) scroll_state: ScrollState,
}

impl TextInput {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            common_element_data: CommonElementData::default(),
        }
    }

    #[allow(dead_code)]
    fn get_state<'a>(&self, element_state: &'a ElementStateStore) -> &'a TextInputState {
        element_state.storage.get(&self.common_element_data.component_id).unwrap().data.as_ref().downcast_ref().unwrap()
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
        element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        window: Option<Arc<dyn Window>>
    ) {
        let computed_layer_rectangle_transformed =
            self.common_element_data.computed_layered_rectangle_transformed;
        let content_rectangle = computed_layer_rectangle_transformed.content_rectangle();

        self.draw_borders(renderer);

        let is_scrollable = self.common_element_data.is_scrollable();

        if is_scrollable {
            self.maybe_start_layer(renderer);
        }

        let scroll_y = if let Some(state) = element_state
            .storage
            .get(&self.common_element_data.component_id)
            .unwrap()
            .data
            .downcast_ref::<TextInputState>()
        {
            state.scroll_state.scroll_y
        } else {
            0.0
        };
        
        let text_scroll = if is_scrollable {
            Some(TextScroll::new(scroll_y, self.common_element_data.computed_scroll_track.height))
        } else {
            None
        };

        renderer.draw_text(
            self.common_element_data.component_id,
            content_rectangle,
            self.common_element_data.style.color(),
            text_scroll
        );

        if let Some(state) = element_state
            .storage
            .get_mut(&self.common_element_data.component_id)
            .unwrap()
            .data
            .downcast_mut::<TextInputState>()
        {

            if let Some((cursor_x, cursor_y)) = state.cached_editor.editor.cursor_position() {
                if state.is_active {
                    if let Some(window) = window {
                        let content_position = self.common_element_data.computed_layered_rectangle_transformed.content_rectangle();
                        window.set_ime_cursor_area(
                            PhysicalPosition::new(content_position.x + cursor_x as f32, content_position.y + cursor_y as f32).into(),
                            PhysicalSize::new(20.0, 20.0).into(),
                        );
                    }
                }
            }

            state.is_active = false;
        }

        if is_scrollable {
            self.maybe_end_layer(renderer);
        }

        self.draw_scrollbar(renderer);
    }

    fn compute_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        _element_state: &mut ElementStateStore,
        scale_factor: f64,
    ) -> Option<NodeId> {
        self.merge_default_style();
        let style: taffy::Style = self.common_element_data.style.to_taffy_style_with_scale_factor(scale_factor);

        self.common_element_data_mut().taffy_node_id = Some(taffy_tree
            .new_leaf_with_context(
                style,
                LayoutContext::TextInput(TaffyTextInputContext::new(self.common_element_data.component_id)),
            )
            .unwrap());

        self.common_element_data().taffy_node_id
    }

    fn finalize_layout(
        &mut self,
        taffy_tree: &mut TaffyTree<LayoutContext>,
        root_node: NodeId,
        position: Point,
        z_index: &mut u32,
        transform: glam::Mat4,
        element_state: &mut ElementStateStore,
        _pointer: Option<Point>,
        _font_system: &mut FontSystem,
    ) {
        let result = taffy_tree.layout(root_node).unwrap();
        self.resolve_layer_rectangle(position, transform, result, z_index);
        self.finalize_borders();

        self.common_element_data.scrollbar_size = Size::new(result.scrollbar_size.width, result.scrollbar_size.height);
        self.common_element_data.computed_scrollbar_size = Size::new(result.scroll_width(), result.scroll_height());

        let scroll_y = if let Some(container_state) = element_state
            .storage
            .get(&self.common_element_data.component_id)
            .unwrap()
            .data
            .downcast_ref::<TextInputState>()
        {
            container_state.scroll_state.scroll_y
        } else {
            0.0
        };

        self.finalize_scrollbar(scroll_y);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn on_event(
        &self,
        message: OkuMessage,
        element_state: &mut ElementStateStore,
        font_system: &mut FontSystem,
    ) -> UpdateResult {
        let base_state = self.get_base_state_mut(element_state);
        let state = base_state.data.as_mut().downcast_mut::<TextInputState>().unwrap();
        state.is_active = true;

        let scroll_result = state.scroll_state.on_event(&message, &self.common_element_data, &mut base_state.base);

        if !scroll_result.propagate {
            return scroll_result;
        }


        let cached_editor = &mut state.cached_editor;

        let scroll_y = state.scroll_state.scroll_y;

        let content_rect = self.common_element_data.computed_layered_rectangle.content_rectangle();
        let content_position = content_rect.position();

        match message {
            OkuMessage::PointerButtonEvent(pointer_button) => {
                let pointer_position = pointer_button.position;
                let pointer_content_position = pointer_position - content_position;
                if pointer_button.state.is_pressed() && content_rect.contains(&pointer_button.position) {
                    cached_editor.editor.action(
                        font_system,
                        Action::Click {
                            x: pointer_content_position.x as i32,
                            y: (pointer_content_position.y + scroll_y) as i32,
                        },
                    );
                    state.dragging = true;
                } else {
                    state.dragging = false;
                }
                UpdateResult::new().prevent_defaults().prevent_propagate()
            }
            OkuMessage::PointerMovedEvent(moved) => {
                if state.dragging {
                    let pointer_position = moved.position;
                    let pointer_content_position = pointer_position - content_position;
                    cached_editor.editor.action(
                        font_system,
                        Action::Drag {
                            x: pointer_content_position.x as i32,
                            y: (pointer_content_position.y + scroll_y) as i32,
                        },
                    );
                }
                UpdateResult::new().prevent_defaults().prevent_propagate()
            }
            OkuMessage::KeyboardInputEvent(keyboard_input) => {
                let logical_key = keyboard_input.event.logical_key;
                let key_state = keyboard_input.event.state;

                println!("{:?}", logical_key);

                if key_state.is_pressed() {
                    match logical_key {
                        Key::Named(NamedKey::ArrowLeft) => {
                            cached_editor.editor.action(font_system, Action::Motion(Motion::Left))
                        }
                        Key::Named(NamedKey::ArrowRight) => {
                            cached_editor.editor.action(font_system, Action::Motion(Motion::Right))
                        }
                        Key::Named(NamedKey::ArrowUp) => cached_editor.editor.action(font_system, Action::Motion(Motion::Up)),
                        Key::Named(NamedKey::ArrowDown) => {
                            cached_editor.editor.action(font_system, Action::Motion(Motion::Down))
                        }
                        Key::Named(NamedKey::Home) => cached_editor.editor.action(font_system, Action::Motion(Motion::Home)),
                        Key::Named(NamedKey::End) => cached_editor.editor.action(font_system, Action::Motion(Motion::End)),
                        Key::Named(NamedKey::PageUp) => {
                            cached_editor.editor.action(font_system, Action::Motion(Motion::PageUp))
                        }
                        Key::Named(NamedKey::PageDown) => {
                            cached_editor.editor.action(font_system, Action::Motion(Motion::PageDown))
                        }
                        Key::Named(NamedKey::Escape) => cached_editor.editor.action(font_system, Action::Escape),
                        Key::Named(NamedKey::Enter) => cached_editor.editor.action(font_system, Action::Enter),
                        Key::Named(NamedKey::Backspace) => cached_editor.editor.action(font_system, Action::Backspace),
                        Key::Named(NamedKey::Delete) => cached_editor.editor.action(font_system, Action::Delete),
                        Key::Named(key) => {
                            if let Some(text) = key.to_text() {
                                for char in text.chars() {
                                    cached_editor.editor.action(font_system, Action::Insert(char));
                                }
                            }
                        }
                        Key::Character(text) => {
                            for c in text.chars() {
                                cached_editor.editor.action(font_system, Action::Insert(c));

                                //text_context.editor.set_selection(Selection::Line(Cursor::new(0, 0)));
                            }
                        }
                        _ => {}
                    }
                }
                cached_editor.editor.shape_as_needed(font_system, true);
                cached_editor.clear_cache();

                let event_text = cached_editor.get_text();
                UpdateResult::new()
                    .prevent_defaults()
                    .prevent_propagate()
                    .result_message(OkuMessage::TextInputChanged(event_text))
            }

            // This is all a bit hacky and needs some improvement:
            OkuMessage::ImeEvent(ime) => {
                let previous_ime_ending_cursor = state.ime_ending_cursor;

                // FIXME: This shouldn't be possible, we need to close the ime window when a text input loses focus.
                if state.ime_starting_cursor.is_none() && !matches!(ime, Ime::Enabled){
                    // state.ime_starting_cursor = Some(cached_editor.editor.cursor());
                    return Default::default();
                }

                // Deletes all the ime pre-edit text from the editor.
                let delete_ime_pre_edit_text = |editor: &mut Editor| {
                    if let Some(previous_ime_ending_cursor) = previous_ime_ending_cursor {
                        let starting_cursor = state.ime_starting_cursor.unwrap();
                        // println!("starting_cursor: {:?}", starting_cursor);
                        // println!("ending_cursor: {:?}", previous_ime_ending_cursor);
                        editor.delete_range(starting_cursor, previous_ime_ending_cursor);
                    }
                };

                // Set the cursor to the final cursor location of the last change item.
                let mut maybe_set_cursor_to_last_change_item = |editor: &mut Editor, change: &Option<Change>| {
                    if let Some(change) = change {
                        if let Some(change_item) = change.items.last() {
                            editor.set_cursor(change_item.end);
                            state.ime_ending_cursor = Some(change_item.end);
                        }
                    }
                };

                // println!("{:?}", ime);

                match ime {
                    Ime::Enabled => {
                        state.is_ime_active = true;
                        state.ime_starting_cursor = Some(cached_editor.editor.cursor());
                        state.ime_ending_cursor = None;
                    }

                    Ime::Preedit(str, cursor_info) => {
                        let is_cleared = str.is_empty();
                        let _hide_cursor = cursor_info.is_none();

                        if is_cleared {
                            if state.ime_ending_cursor.is_some() {
                                cached_editor.editor.start_change();
                                delete_ime_pre_edit_text(&mut cached_editor.editor);
                                cached_editor.editor.finish_change();
                                state.ime_ending_cursor = None;
                                cached_editor.editor.set_cursor(state.ime_starting_cursor.unwrap());
                            }
                        } else {
                            cached_editor.editor.start_change();
                            delete_ime_pre_edit_text(&mut cached_editor.editor);
                            cached_editor.editor.insert_at(state.ime_starting_cursor.unwrap(), str.as_str(), None);
                            let change = cached_editor.editor.finish_change();
                            maybe_set_cursor_to_last_change_item(&mut cached_editor.editor, &change);
                        }
                    }
                    Ime::Commit(str) => {
                        state.is_ime_active = false;

                        cached_editor.editor.start_change();
                        // delete_ime_pre_edit_text(&mut cached_editor.editor);
                        cached_editor.editor.insert_at(state.ime_starting_cursor.unwrap(), str.as_str(), None);
                        let change = cached_editor.editor.finish_change();
                        maybe_set_cursor_to_last_change_item(&mut cached_editor.editor, &change);
                    }
                    Ime::Disabled => {
                        state.is_ime_active = false;
                        state.ime_starting_cursor = None;
                        state.ime_ending_cursor = None;
                    }
                };

                cached_editor.editor.shape_as_needed(font_system, true);
                cached_editor.clear_cache();

                let event_text = cached_editor.get_text();
                UpdateResult::new()
                    .prevent_defaults()
                    .prevent_propagate()
                    .result_message(OkuMessage::TextInputChanged(event_text))
            }
            
            _ => UpdateResult::new(),
        }
    }

    fn initialize_state(&self, font_system: &mut FontSystem, scaling_factor: f64) -> ElementStateStoreItem {
        let cached_editor = CachedEditor::new(&self.text, &self.common_element_data.style, scaling_factor, font_system);
        let text_input_state = TextInputState {
            cached_editor,
            dragging: false,
            is_ime_active: false,
            is_active: false,
            ime_starting_cursor: None,
            ime_ending_cursor: None,
            scroll_state: ScrollState::default(),
        };

        ElementStateStoreItem {
            base: Default::default(),
            data: Box::new(text_input_state)
        }
    }

    fn update_state(&self, font_system: &mut FontSystem, element_state: &mut ElementStateStore, reload_fonts: bool, scaling_factor: f64) {
        let state: &mut TextInputState = element_state
            .storage
            .get_mut(&self.common_element_data.component_id)
            .unwrap()
            .data
            .as_mut()
            .downcast_mut()
            .unwrap();

        state.cached_editor.update_state(&self.text, &self.common_element_data.style, scaling_factor, reload_fonts, font_system);
    }

    fn default_style(&self) -> Style {
        let mut style = Style::default();
        *style.display_mut() = Display::Block;
        const BORDER_COLOR: Color = Color::from_rgb8(199, 199, 206);
        *style.border_color_mut() = [BORDER_COLOR; 4];
        *style.border_width_mut() = [Unit::Px(1.0); 4];
        *style.border_radius_mut() = [(5.0, 5.0); 4];
        let padding = Unit::Px(4.0);
        *style.padding_mut() = [padding, padding, padding, padding];

        style
    }
}

impl TextInput {
    generate_component_methods_no_children!();
}

impl ElementStyles for TextInput {
    fn styles_mut(&mut self) -> &mut Style {
        self.common_element_data.current_style_mut()
    }
}
