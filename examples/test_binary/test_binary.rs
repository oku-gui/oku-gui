use oku::components::component::Component;
use oku::elements::container::Container;
use oku::elements::text::Text;
use oku_core::components::component::{ComponentOrElement, ComponentSpecification};
use oku_core::components::props::Props;
use oku_core::elements::element::Element;
use oku_core::elements::empty::Empty;
use oku_core::elements::style::{AlignItems, FlexDirection, JustifyContent, Unit};
use oku_core::events::EventResult;
use oku_core::reactive::reactive;
use oku_core::reactive::reactive::RUNTIME;
use oku_core::renderer::color::Color;
use oku_core::RendererType::Wgpu;
use oku_core::{component, OkuOptions};
use std::any::Any;
use std::sync::Arc;

struct Test1 {}

pub fn something(_props: Option<Props>, children: Vec<ComponentSpecification>, id: u64) -> ComponentSpecification {
    ComponentSpecification {
        component: Container::new().background(Color::new_from_rgba_u8(0, 155, 25, 255)).width(Unit::Px(20.0)).height(Unit::Px(20.0)).into(),
        key: Some("Something".to_string()),
        props: None,
        children: vec![Text::new("Hi2").into()],
    }
}

pub fn twwwww(_props: Option<Props>, children: Vec<ComponentSpecification>, id: u64) -> ComponentSpecification {
    ComponentSpecification {
        component: Container::new().background(Color::new_from_rgba_u8(0, 155, 25, 255)).width(Unit::Px(20.0)).height(Unit::Px(20.0)).into(),
        key: Some("twwwww".to_string()),
        props: None,
        children: vec![Text::new("Hi3").into()],
    }
}

pub fn app(_props: Option<Props>, children: Vec<ComponentSpecification>, id: u64) -> ComponentSpecification {
    ComponentSpecification {
        component: Container::new().background(Color::new_from_rgba_u8(0, 255, 0, 255)).width(Unit::Px(50.0)).height(Unit::Px(50.0)).into(),
        key: Some("App".to_string()),
        props: None,
        children: vec![ComponentSpecification {
            component: component!(something),
            key: None,
            props: None,
            children: vec![],
        }],
    }
}

fn foo(_props: Option<Props>, children: Vec<ComponentSpecification>, id: u64) -> ComponentSpecification {
    let background = Container::new().background(Color::new_from_rgba_u8(255, 0, 0, 255)).width(Unit::Px(200.0)).height(Unit::Px(200.0));

    let counter = RUNTIME.get_state::<u32>(id).unwrap_or(0u32);

    let q = if counter % 2 == 0 { Text::new(format!("EQUAL NUMBER: {}", counter).as_str()).into() } else { Empty::new().into() };

    ComponentSpecification {
        component: background.into(),
        key: None,
        props: None,
        children: vec![
            Text::new(format!("Hello, world 1! \n Count: {}", counter).as_str()).into(),
            q,
            ComponentSpecification {
                component: component!(app),
                key: None,
                props: None,
                children: vec![],
            },
            ComponentSpecification {
                component: component!(twwwww),
                key: None,
                props: None,
                children: vec![],
            },
            Text::new("Hello, world 2!").into(),
        ],
    }
}

fn main() {
    oku_core::oku_main_with_options(
        ComponentSpecification {
            component: component!(foo),
            key: None,
            props: None,
            children: vec![],
        },
        Some(OkuOptions { renderer: Wgpu }),
    );
}
