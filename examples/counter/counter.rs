use oku::user::components::component::ComponentOrElement;
use oku::user::components::component::ComponentSpecification;
use oku::user::components::component::UpdateFn;
use oku::user::components::props::Props;
use oku::user::elements::container::Container;
use oku::user::elements::text::Text;
use oku_core::engine::events::Message;

use oku::RendererType::Wgpu;
use oku::{component, oku_main_with_options, OkuOptions};
use std::any::Any;
use std::future::Future;
use std::ops::Deref;
use oku_core::user::elements::element::Element;
use oku_core::engine::events::OkuEvent;
use oku_core::user::components::component::UpdateResult;

pub fn app(
    state: Option<&dyn Any>,
    _props: Option<Props>,
    _children: Vec<ComponentSpecification>,
    id: u64,
) -> (ComponentSpecification, Option<UpdateFn>) {
    let counter: u32 = match state {
        Some(state) => {
            *state.downcast_ref().unwrap()
        },
        None => {
            0
        }
    };
    
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

fn counter_update(state: &mut Option<Box<dyn Any + Send>>, id: u64, message: Message, source_element: Option<String>) -> UpdateResult {
    if source_element.as_deref() != Some("increment") {
        return UpdateResult::default();
    }

    let counter: u32 = match state {
        Some(state) => {
            *state.downcast_mut::<u32>().unwrap()
        }
        None => {
            0
        }
    };
    
    let new_counter = match message {
        Message::OkuMessage(oku_message) => {
            match oku_message {
                OkuEvent::Click(click_message) => counter + 1,
            }
        },
        _ => counter,
    };

    *state = Some(Box::new(counter + 1));

    println!("Counter: {}", new_counter);
    UpdateResult::new(true, None)
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
