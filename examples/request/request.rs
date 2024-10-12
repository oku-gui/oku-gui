use oku::user::components::component::ComponentSpecification;
use oku::user::components::props::Props;
use oku::user::elements::container::Container;
use oku::user::elements::text::Text;
use oku_core::engine::events::Message;

use oku::RendererType::Wgpu;
use oku::{oku_main_with_options, OkuOptions};
use oku_core::engine::events::OkuEvent;
use oku_core::user::components::component::{Component, UpdateResult};
use oku_core::user::elements::element::Element;
use std::any::Any;
use bytes::Bytes;
use oku_core::PinnedFutureAny;
use oku_core::user::elements::image::Image;

#[derive(Default, Copy, Clone)]
pub struct RequestExample {
    dummy_state: bool
}

impl Component for RequestExample {
    fn view(state: &Self, _props: Option<Props>, _children: Vec<ComponentSpecification>, id: u64) -> ComponentSpecification {
        let mut button = Container::new();
        button.set_id(Some("fetch_image".to_string()));
        let mut button_label = Text::new("Get Random Image");
        button_label.set_id(Some("fetch_image".to_string()));


        ComponentSpecification {
            component: Container::new().into(),
            key: None,
            props: None,
            children: vec![
                ComponentSpecification {
                    component: Image::new("a.jpg").into(),
                    key: None,
                    props: None,
                    children: vec![],
                },
                ComponentSpecification {
                    component: button.into(),
                    key: None,
                    props: None,
                    children: vec![ComponentSpecification {
                        component: button_label.into(),
                        key: None,
                        props: None,
                        children: vec![],
                    }],
                },
            ],
        }
    }

    fn update(state: &mut Self, id: u64, message: Message, source_element: Option<String>) -> UpdateResult {
        if source_element.as_deref() != Some("fetch_image") {
            return UpdateResult::default();
        }

        let res: Option<PinnedFutureAny> = match message {
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
                }

                None
            },
        };



        UpdateResult::new(true, res)
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
                    component: RequestExample::component(),
                    key: None,
                    props: None,
                    children: vec![],
                },
            ],
        },
        Some(OkuOptions { renderer: Wgpu }),
    );
}
