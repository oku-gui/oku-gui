use oku::user::components::component::ComponentSpecification;
use oku::user::components::props::Props;
use oku::user::elements::container::Container;
use oku::user::elements::text::Text;

use oku::RendererType::Wgpu;
use oku::{oku_main_with_options, OkuOptions};

use oku_core::engine::events::{Message, OkuEvent};
use oku_core::user::components::component::{Component, ComponentId, UpdateResult};
use oku_core::user::elements::element::Element;
use oku_core::user::elements::style::FlexDirection;

#[derive(Default, Copy, Clone)]
pub struct Accordion {
    show_content: bool,
}

impl Component for Accordion {
    fn view(state: &Self, _props: Option<Props>, _children: Vec<ComponentSpecification>, id: ComponentId) -> ComponentSpecification {
        let mut accordion_header = Container::new();
        accordion_header.set_id(Some("accordion_header".to_string()));

        let mut accordion_header_text = Text::new("Accordion Example");
        accordion_header_text.set_id(Some("accordion_header".to_string()));

        let accordion_content = if state.show_content {
            ComponentSpecification {
                component: Text::new("My content!").into(),
                key: None,
                props: None,
                children: vec![],
            }
        } else {
            ComponentSpecification {
                component: Container::new().into(),
                key: None,
                props: None,
                children: vec![],
            }
        };

        ComponentSpecification {
            component: Container::new().margin(14.0, 0.0, 0.0, 14.0).flex_direction(FlexDirection::Column).into(),
            key: Some("accordion container".to_string()),
            props: None,
            children: vec![
                ComponentSpecification {
                    component: accordion_header.into(),
                    key: Some("accordion_header".to_string()),
                    props: None,
                    children: vec![
                        ComponentSpecification {
                            component: accordion_header_text.into(),
                            key: None,
                            props: None,
                            children: vec![],
                        },
                    ],
                },
                accordion_content
            ],
        }
    }

    fn update(state: &mut Self, id: ComponentId, message: Message, source_element: Option<String>) -> UpdateResult {
        if source_element.as_deref() != Some("accordion_header") {
            return UpdateResult::default();
        }

        if let Message::OkuMessage(OkuEvent::Click(click_message)) = message {
            state.show_content = !state.show_content
        };

        UpdateResult::new(true, None)
    }
}

fn main() {

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE) // Set the maximum log level you want to capture
        .init();

    oku_main_with_options(
        ComponentSpecification {
            component: Container::new().into(),
            key: None,
            props: None,
            children: vec![
                ComponentSpecification {
                    component: Accordion::component(),
                    key: None,
                    props: None,
                    children: vec![],
                },
                ComponentSpecification {
                    component: Accordion::component(),
                    key: None,
                    props: None,
                    children: vec![],
                },
            ],
        },
        Some(OkuOptions { renderer: Wgpu }),
    );
}