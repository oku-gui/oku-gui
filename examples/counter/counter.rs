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

pub trait Foo<T: 'static + Default + Send> {
    
    fn view2(state: &T, _props: Option<Props>,
             _children: Vec<ComponentSpecification>,
             id: u64) -> (ComponentSpecification, Option<UpdateFn>);
    fn view(state: &(dyn Any + Send),  _props: Option<Props>, _children: Vec<ComponentSpecification>, id: u64) -> (Box<dyn Any + Send>, ComponentSpecification, Option<UpdateFn>){
        let casted_state: &T = state.downcast_ref::<T>().unwrap();

        // Call view2 with the casted result
        let view = Self::view2(casted_state, _props, _children, id);
        (Self::default_value(), view.0, view.1)
    }
    
    fn default_value() -> Box<dyn Any + Send> {
        Box::new(T::default())
    }


    fn update2(state: &mut T, id: u64, message: Message, source_element: Option<String>) -> UpdateResult;

    fn update(state: &mut dyn Any, id: u64, message: Message, source_element: Option<String>) -> UpdateResult {
        let casted_state: &mut T = state.downcast_mut::<T>().unwrap();

        Self::update2(casted_state, id, message, source_element)
    }
}

pub struct f;

impl Foo<u64> for f {
    fn view2(state: &u64, _props: Option<Props>, _children: Vec<ComponentSpecification>, id: u64) -> (ComponentSpecification, Option<UpdateFn>) {
        let counter = *state;

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

    fn update2(state: &mut u64, id: u64, message: Message, source_element: Option<String>) -> UpdateResult {
        if source_element.as_deref() != Some("increment") {
            return UpdateResult::default();
        }

        let counter: u64 = *state;

        let new_counter = match message {
            Message::OkuMessage(oku_message) => {
                match oku_message {
                    OkuEvent::Click(click_message) => counter + 1,
                }
            },
            _ => counter,
        };

        *state = counter;

        println!("Counter: {}", new_counter);
        UpdateResult::new(true, None)
    }
}

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

fn counter_update(state: &mut Box<dyn Any + Send>, id: u64, message: Message, source_element: Option<String>) -> UpdateResult {
   /* if source_element.as_deref() != Some("increment") {
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

    println!("Counter: {}", new_counter);*/
    UpdateResult::new(true, None)
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE) // Set the maximum log level you want to capture
        .init();
    let x = f::view;
    oku_main_with_options(
        ComponentSpecification {
            component: Container::new().into(),
            key: None,
            props: None,
            children: vec![
                ComponentSpecification {
                    component: component!(f::view),
                    key: None,
                    props: None,
                    children: vec![],
                },
                ComponentSpecification {
                    component: component!(f::view),
                    key: None,
                    props: None,
                    children: vec![],
                },
            ],
        },
        Some(OkuOptions { renderer: Wgpu }),
    );
}
