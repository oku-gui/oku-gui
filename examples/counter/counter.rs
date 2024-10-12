use oku::user::components::component::ComponentSpecification;
use oku::user::components::props::Props;
use oku::user::elements::container::Container;
use oku::user::elements::text::Text;
use oku_core::engine::events::Message;

use oku::RendererType::Wgpu;
use oku::{oku_main_with_options, OkuOptions};
use oku_core::engine::events::OkuEvent;
use oku_core::user::components::component::{Component, ComponentId, UpdateResult};
use oku_core::user::elements::element::Element;

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
        let mut button = Container::new();
        button.set_id(Some("increment".to_string()));

        let mut button_label = Text::new("increment");
        button_label.set_id(Some("increment".to_string()));

        ComponentSpecification {
            component: Container::new().into(),
            key: Some("counter container".to_string()),
            props: None,
            children: vec![
                ComponentSpecification {
                    component: Text::new(format!("Counter: {}", state.count).as_str()).into(),
                    key: Some("counter text".to_string()),
                    props: None,
                    children: vec![],
                },
                ComponentSpecification {
                    component: button.into(),
                    key: Some("increment button".to_string()),
                    props: None,
                    children: vec![ComponentSpecification {
                        component: button_label.into(),
                        key: Some("increment text".to_string()),
                        props: None,
                        children: vec![],
                    }],
                },
            ],
        }
    }

    fn update(state: &mut Self, id: ComponentId, message: Message, source_element: Option<String>) -> UpdateResult {
        println!("WORKING...");
        if source_element.as_deref() != Some("increment") {
            return UpdateResult::default();
        }

        if let Message::OkuMessage(OkuEvent::Click(click_message)) = message {
            state.count += 1
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
                    component: Counter::component(),
                    key: None,
                    props: None,
                    children: vec![],
                },
                ComponentSpecification {
                    component: Counter::component(),
                    key: None,
                    props: None,
                    children: vec![],
                },
            ],
        },
        Some(OkuOptions { renderer: Wgpu }),
    );
}
