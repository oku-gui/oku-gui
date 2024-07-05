use oku::elements::style::{FlexDirection, Unit};
use oku::components::component::UpdateFn;
use oku::components::component::ComponentOrElement;
use oku::components::component::{Component, ComponentSpecification};
use oku::elements::container::Container;
use oku::elements::text::Text;
use std::any::Any;
use oku::{component, oku_main_with_options, OkuOptions};
use oku::components::props::Props;
use oku::events::Message;
use oku::reactive::reactive::RUNTIME;
use oku::renderer::color::Color;
use oku::RendererType::Wgpu;
use rand::seq::SliceRandom;

pub fn counter(_props: Option<Props>, children: Vec<ComponentSpecification>, id: u64) -> (ComponentSpecification, Option<UpdateFn>) {
    let count = RUNTIME.get_state(id).unwrap_or(0u32);

    println!("Children len: {}", children.len());

    //let mut component_children = children.clone();
    //component_children.push(Text::new(format!("Counter Count: {}", count).as_str()).into());

    (
        ComponentSpecification {
            component: Text::new(format!("Counter Count: {}", count).as_str()).into(),
            key: None,
            props: None,
            children: vec![],
        },
        Some(update),
    )
}

pub fn app(_props: Option<Props>, children: Vec<ComponentSpecification>, id: u64) -> (ComponentSpecification, Option<UpdateFn>) {

    let mut rng = rand::thread_rng();
    let mut children2: Vec<ComponentSpecification> = vec![
        ComponentSpecification {
            component: component!(counter),
            key: Some("3".to_string()),
            props: None,
            children: vec![
                Text::new(" K: 3").into()
            ],
        },
        ComponentSpecification {
            component: component!(counter),
            key: Some("1".to_string()),
            props: None,
            children: vec![
                Text::new(" K: 1").into()
            ],
        },
        ComponentSpecification {
            component: component!(counter),
            key: Some("2".to_string()),
            props: None,
            children: vec![
                Text::new(" K: 2").into()
            ],
        },
    ];
    children2.shuffle(&mut rng);

    (
        ComponentSpecification {
            component: Container::new().background(Color::new_from_rgba_u8(240, 255, 220, 255)).width(Unit::Px(900.0)).height(Unit::Px(900.0)).flex_direction(FlexDirection::Column).into(),
            key: None,
            props: None,
            children: vec![
                ComponentSpecification {
                    component: Container::new()
                        .background(Color::new_from_rgba_u8(0, 255, 255, 255))
                        .into(),
                    key: None,
                    props: None,
                    children: vec![ComponentSpecification {
                        component: component!(counter),
                        key: None,
                        props: None,
                        children: vec![],
                    }],
                },
                Text::new("Hello World!").into(),
                ComponentSpecification {
                    component: Container::new()
                        .background(Color::new_from_rgba_u8(255, 0, 255, 255))
                        .into(),
                    key: None,
                    props: None,
                    children: vec![ComponentSpecification {
                        component: Container::new().into(),
                        key: None,
                        props: None,
                        children: children2,
                    }],
                },
            ],
        },
        Some(update),
    )
}

pub fn update(id: u64, message: Message) {
    println!("Update: {}", id);
}

fn main() {
    oku_main_with_options(
        ComponentSpecification {
            component: component!(app),
            key: None,
            props: None,
            children: vec![],
        },
        Some(OkuOptions { renderer: Wgpu }),
    );
}
