use oku::user::components::component::ComponentOrElement;
use oku::user::components::component::ComponentSpecification;
use oku::user::components::component::UpdateFn;
use oku::user::components::props::Props;
use oku::user::elements::container::Container;
use oku::user::elements::text::Text;
use oku::user::reactive::reactive::RUNTIME;
use oku_core::engine::events::Message;

use oku::RendererType::Wgpu;
use oku::{component, oku_main_with_options, OkuOptions};
use oku_core::engine::events::OkuEvent;
use oku_core::user::components::component::UpdateResult;
use oku_core::user::elements::element::Element;
use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use bytes::Bytes;
use std::sync::Arc;
use oku_core::user::elements::image::Image;

pub fn app(
    _props: Option<Props>,
    _children: Vec<ComponentSpecification>,
    id: u64,
) -> (ComponentSpecification, Option<UpdateFn>) {
    let counter: Option<Bytes> = RUNTIME.get_state(id).unwrap_or(None);

    let mut button = Container::new();
    button.set_id(Some("increment".to_string()));

    let mut button_label = Text::new("increment");
    button_label.set_id(Some("increment".to_string()));

    let counter = if let Some(data) = counter {
        data.len().to_string()
    } else {
        String::from("None")
    };

    let root = ComponentSpecification {
        component: Container::new().into(),
        key: Some("counter container".to_string()),
        props: None,
        children: vec![
            ComponentSpecification {
                component: Image::new("a.jpg").into(),
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
    };
    (root, Some(counter_update))
}

fn counter_update(id: u64, message: Message, source_element: Option<String>) -> UpdateResult {
    if source_element.as_deref() != Some("increment") {
        return UpdateResult::default();
    }

    let counter = RUNTIME.get_state(id).unwrap_or(0);
    let res: Option<Pin<Box<dyn Future<Output = Box<dyn Any + Send>> + Send>>> = match message {
        Message::OkuMessage(oku_message) => match oku_message {
            OkuEvent::Click(click_message) => {
                Some(Box::pin(async {

                    let res = reqwest::get("https://picsum.photos/800").await;
                    let bytes = res.unwrap().bytes().await.ok();
                    let boxed: Box<dyn Any + Send> = Box::new(bytes);

                    boxed
                }))
            }
        },
        Message::UserMessage(user_message) => {
            if let Ok(image_data) = user_message.downcast::<Option<Bytes>>() {
                std::fs::write("a.jpg", image_data.clone().unwrap().as_ref()).ok();
                RUNTIME.set_state::<Option<Bytes>>(id, *image_data);
                println!("got the data");
            }

            None
        },
        _ => None,
    };

    UpdateResult {
        propagate: true,
        result: res,
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO) // Set the maximum log level you want to capture
        .init();

    oku_main_with_options(
        ComponentSpecification {
            component: Container::new().into(),
            key: None,
            props: None,
            children: vec![ComponentSpecification {
                component: component!(app),
                key: None,
                props: None,
                children: vec![],
            }],
        },
        Some(OkuOptions { renderer: Wgpu }),
    );
}
