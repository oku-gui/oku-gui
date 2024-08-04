use oku::components::component::ComponentOrElement;
use oku::components::component::ComponentSpecification;
use oku::components::component::UpdateFn;
use oku::components::props::Props;
use oku::elements::container::Container;
use oku::elements::style::Weight;
use oku::elements::style::{AlignItems, JustifyContent};
use oku::elements::style::{FlexDirection, Unit};
use oku::elements::text::Text;
use oku::events::Message;
use oku::reactive::reactive::RUNTIME;

use oku::RendererType::Wgpu;
use oku::{component, oku_main_with_options, OkuOptions};
use std::any::Any;
use std::future::Future;
use oku_core::elements::element::Element;
use oku_core::events::OkuEvent;

pub fn app(
    _props: Option<Props>,
    _children: Vec<ComponentSpecification>,
    id: u64,
) -> (ComponentSpecification, Option<UpdateFn>) {
    let counter = RUNTIME.get_state(id).unwrap_or(0);
    
    let mut button = Container::new();
    button.set_id(Some("increment".to_string()));

    let mut button_label = Text::new("increment");
    button_label.set_id(Some("increment".to_string()));
    
    let root = ComponentSpecification {
        component: Container::new().into(),
        key: Some("counter container".to_string()),
        props: None,
        children: vec![
            ComponentSpecification {
                component: Text::new(format!("Counter: {}", counter).as_str()).into(),
                key: Some("counter text".to_string()),
                props: None,
                children: vec![],
            },
            ComponentSpecification {
                component: button.into(),
                key: Some("increment button".to_string()),
                props: None,
                children: vec![
                    ComponentSpecification {
                        component: button_label.into(),
                        key: Some("increment text".to_string()),
                        props: None,
                        children: vec![],
                    },
                ],
            }
        ],
    };
    (root, Some(counter_update))
}

fn counter_update(id: u64, message: Message, source_element: Option<String>) -> (bool, Option<Box<dyn Future<Output=Box<dyn Any>>>>) {
    if source_element.as_deref() != Some("increment") {
        return (false, None);
    }

    let counter = RUNTIME.get_state(id).unwrap_or(0);
    let new_counter = match message {
        Message::OkuMessage(oku_message) => {
            match oku_message {
                OkuEvent::Click(click_message) => counter + 1,
            }
        },
        _ => counter,
    };
    RUNTIME.set_state(id, new_counter);
    println!("Counter: {}", new_counter);
    (true, None)
}

fn main() {
    oku_main_with_options(
        ComponentSpecification {
            component: Container::new().into(),
            key: None,
            props: None,
            children: vec![
                ComponentSpecification {
                    component: component!(app),
                    key: None,
                    props: None,
                    children: vec![],
                },
                ComponentSpecification {
                    component: component!(app),
                    key: None,
                    props: None,
                    children: vec![],
                },
            ],
        },
        Some(OkuOptions { renderer: Wgpu }),
    );
}
