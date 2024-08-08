use std::any::Any;
use std::future::Future;
use oku_core::engine::events::{Message, OkuEvent};
use oku_core::user::components::component::{ComponentOrElement, ComponentSpecification, UpdateFn};
use oku_core::user::components::props::Props;
use oku_core::user::elements::container::Container;
use oku_core::user::elements::element::Element;
use oku_core::user::elements::empty::Empty;
use oku_core::user::elements::style::{FlexDirection};
use oku_core::user::reactive::reactive::RUNTIME;

pub(crate) fn accordion(_props: Option<Props>, children: Vec<ComponentSpecification>, id: u64) -> (ComponentSpecification, Option<UpdateFn>) {

    let render_body = RUNTIME.get_state(id).unwrap_or(false);

    let root = ComponentSpecification {
        component: Container::new().flex_direction(FlexDirection::Column).into(),
        key: None,
        props: None,
        children: children.iter().filter_map(|item| {
            match &item.component {
                ComponentOrElement::ComponentSpec(_, component_name, _) => {
                    if component_name == "accordion::accordion_component::accordion_content" && !render_body {
                        // NOTE: Empty isn't working here, it will cause a crash when we diff the tree.
                        // Some(Empty::new().into())
                        None
                    } else  {
                        Some(item.clone())
                    }
                }
                ComponentOrElement::Element(_) => Some(item.clone())
            }
            
        }).into_iter().collect(),
    };
    (root, Some(accordion_update))
}

pub(crate) fn accordion_update(id: u64, message: Message, source_element: Option<String>) -> (bool, Option<Box<dyn Future<Output=Box<dyn Any>>>>) {

    if source_element != Some("toggle accordion header".to_string()) {
        return (true, None);
    }

    let render_body = RUNTIME.get_state(id).unwrap_or(false);

    let render_body = match message {
        Message::OkuMessage(oku_message) => {
            match oku_message {
                OkuEvent::Click(_) => !render_body,
            }
        },
        _ => render_body,
    };
    RUNTIME.set_state(id, render_body);
    (false, None)
}

pub(crate) fn accordion_header(_props: Option<Props>, children: Vec<ComponentSpecification>, _id: u64) -> (ComponentSpecification, Option<UpdateFn>) {

    let mut container = Container::new();
    container.set_id(Some("toggle accordion header".to_string()));

    let root = ComponentSpecification {
        component: container.into(),
        key: None,
        props: None,
        children: children.into_iter().map(|mut item| {
            match &mut item.component {
                ComponentOrElement::ComponentSpec(_, _, _) => {}
                ComponentOrElement::Element(ref mut element) => {
                    element.set_id(Some("toggle accordion header".to_string()))
                }
            };
            item
        }).collect(),
    };
    (root, None)
}

pub(crate) fn accordion_content(_props: Option<Props>, children: Vec<ComponentSpecification>, _id: u64) -> (ComponentSpecification, Option<UpdateFn>) {
    let root = ComponentSpecification {
        component: Container::new().into(),
        key: None,
        props: None,
        children,
    };
    (root, None)
}