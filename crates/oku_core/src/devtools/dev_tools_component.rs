use crate::components::props::Props;
use crate::components::{Component, ComponentId, ComponentSpecification, UpdateResult};
use crate::devtools::dev_tools_colors::CONTAINER_BACKGROUND_COLOR;
use crate::devtools::dev_tools_widget::DevTools;
use crate::devtools::element_tree_view::element_tree_view;
use crate::devtools::style_window::styles_window_view;
use crate::elements::element::Element;
use crate::elements::ElementStyles;
use crate::events::{clicked, Event, Message, OkuMessage};
use crate::style::Display::Flex;
use crate::style::{FlexDirection, Unit};

#[derive(Default)]
pub(crate) struct DevToolsComponent {
    pub selected_element: Option<ComponentId>,
    pub inspector_hovered_element: Option<ComponentId>,
}

impl Component for DevToolsComponent {
    type Props = Option<Box<dyn Element>>;

    fn view(state: &Self, props: &Self::Props, _children: Vec<ComponentSpecification>) -> ComponentSpecification {
        let root = props.as_ref().unwrap().clone();
        let element_tree = element_tree_view(&root, state.selected_element);

        // Find the selected element in the element tree, so that we can inspect their style values.
        let mut selected_element: Option<Box<&dyn Element>> = None;
        if state.selected_element.is_some() {
            for element in root.pre_order_iter().collect::<Vec<&dyn Element>>().iter().rev() {
                if element.component_id() != state.selected_element.unwrap() {
                    continue;
                }

                selected_element = Some(Box::new(<&dyn Element>::clone(element)));
                break;
            }
        }

        let styles_window = styles_window_view(selected_element);

        DevTools::new()
            .display(Flex)
            .push_debug_inspector_tree(&root)
            .push_selected_inspector_element(state.selected_element)
            .push_hovered_inspector_element(state.inspector_hovered_element)
            .flex_direction(FlexDirection::Column)
            .background(CONTAINER_BACKGROUND_COLOR)
            .width(Unit::Percentage(100.0))
            .height(Unit::Percentage(100.0))
            .max_height(Unit::Percentage(100.0))
            .push(element_tree)
            .push(styles_window)
            .component()
    }

    fn update(state: &mut Self, _props: &Self::Props, event: Event) -> UpdateResult {
        if let Some(id) = event.target {

            // Set the selected element in the element tree inspector.
            if clicked(&event.message) {
                let component_id: ComponentId = id.parse().unwrap();
                state.selected_element = Some(component_id);
            }

            // Update the hovered element in the inspector tree, so that the DevTools widget can draw a debug overlay.
            if let Message::OkuMessage(OkuMessage::PointerMovedEvent(_pointer_moved_event)) = event.message {
                let component_id: ComponentId = id.parse().unwrap();
                state.inspector_hovered_element = Some(component_id);
            }

        } else {
            state.inspector_hovered_element = None;
        }

        UpdateResult::default()
    }
}

pub fn dev_tools_view(root: &Box<dyn Element>) -> ComponentSpecification {
    DevToolsComponent::component().props(Props::new(Some(root.clone())))
}