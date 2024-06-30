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

pub fn app(_props: Option<Props>, children: Vec<ComponentSpecification>, id: u64) -> ComponentSpecification {
    ComponentSpecification {
        component: Container::new().background(Color::new_from_rgba_u8(0, 255, 0, 255)).width(Unit::Px(50.0)).height(Unit::Px(50.0)).into(),
        key: Some("App".to_string()),
        props: None,
        children: vec![ComponentSpecification {
            component: Text::new("Hello, world!").into(),
            key: None,
            props: None,
            children: vec![],
        }],
    }
}

fn main() {
    oku_core::oku_main_with_options(
        ComponentSpecification {
            component: component!(app),
            key: None,
            props: None,
            children: vec![],
        },
        Some(OkuOptions { renderer: Wgpu }),
    );
}
