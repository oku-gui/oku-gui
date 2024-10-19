use oku::user::components::component::ComponentSpecification;
use oku::user::components::props::Props;
use oku::user::elements::container::Container;
use oku::user::elements::text::Text;
use oku_core::engine::events::{ButtonSource, ElementState, Message, MouseButton};

use oku::oku_main_with_options;
use oku_core::engine::events::OkuEvent::PointerButtonEvent;
use oku_core::user::components::component::{Component, ComponentId, UpdateResult};
use oku_core::user::elements::element::Element;
use oku_core::OkuOptions;
use oku_core::RendererType::Wgpu;

#[derive(Default, Copy, Clone)]
pub struct Counter {
    count: u64,
}

impl Component for Counter {
    fn view(
        state: &Self,
        _props: Option<Props>,
        _children: Vec<ComponentSpecification>,
        id: ComponentId,
    ) -> ComponentSpecification {

        ComponentSpecification {
            component: Container::new().into(),
            key: None,
            props: None,
            children: vec![
                ComponentSpecification {
                    component: Text::new(format!("Counter: {}", state.count).as_str()).into(),
                    key: None,
                    props: None,
                    children: vec![],
                },
                ComponentSpecification {
                    component: Container::new().id(Some("increment".to_string())).into(),
                    key: None,
                    props: None,
                    children: vec![ComponentSpecification {
                        component: Text::new("increment").id(Some("increment".to_string())).into(),
                        key: None,
                        props: None,
                        children: vec![],
                    }],
                },
                ComponentSpecification {
                    component: Text::new("奥奥奥奥奥奥奥奥奥奥奥奥奥奥奥奥奥奥奥奥奥奥奥奥奥奥奥奥奥奥奥").into(),
                    key: None,
                    props: None,
                    children: vec![ComponentSpecification {
                        component: Text::new("increment").id(Some("increment".to_string())).into(),
                        key: None,
                        props: None,
                        children: vec![],
                    }],
                },
            ],
        }
    }

    fn update(state: &mut Self, _id: ComponentId, message: Message, source_element: Option<String>) -> UpdateResult {
        if source_element.as_deref() != Some("increment") {
            return UpdateResult::default();
        }

        if let Message::OkuMessage(PointerButtonEvent(pointer_button)) = message {
            if pointer_button.button == ButtonSource::Mouse(MouseButton::Left)
                && pointer_button.state == ElementState::Pressed
            {
                state.count += 1
            }
        };

        UpdateResult::new(true, None)
    }
}

fn main() {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    oku_main_with_options(
        ComponentSpecification {
            component: Container::new().into(),
            key: None,
            props: None,
            children: vec![
                ComponentSpecification {
                    component: Counter::component(),
                    key: Some(String::from("counter")),
                    props: None,
                    children: vec![],
                },
            ],
        },
        Some(OkuOptions { renderer: Wgpu }),
    );
}
