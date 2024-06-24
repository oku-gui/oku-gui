use oku::components::component::Component;
use oku::elements::container::Container;
use oku::elements::text::Text;
use oku_core::components::component::{ComponentOrElement, ComponentDefinition};
use oku_core::elements::element::Element;
use oku_core::elements::style::{AlignItems, FlexDirection, JustifyContent, Unit};
use oku_core::events::EventResult;
use oku_core::reactive::reactive;
use oku_core::reactive::reactive::RUNTIME;
use oku_core::renderer::color::Color;
use oku_core::OkuOptions;
use oku_core::RendererType::Wgpu;
use std::any::Any;
use std::sync::Arc;
use oku_core::components::props::Props;
use oku_core::elements::empty::Empty;

struct Test1 {}

pub fn something(_props: Option<Props>, children: Vec<ComponentDefinition>, id: u64) -> ComponentDefinition {
    //println!("-> app");
    println!("something id: {}", id);
    ComponentDefinition {
        component: Container::new()
            .background(Color::new_from_rgba_u8(0, 155, 25, 255))
            .width(Unit::Px(20.0))
            .height(Unit::Px(20.0))
            .into(),
        key: Some("Something".to_string()),
        props: None,
        children: vec![
            ComponentDefinition {
                component: Text::new("Hi2").into(),
                key: None,
                props: None,
                children: vec![],
            },
        ],
    }
}

pub fn twwwww(_props: Option<Props>, children: Vec<ComponentDefinition>, id: u64) -> ComponentDefinition {
    //println!("-> app");
    println!("twwwww id: {}", id);
    ComponentDefinition {
        component: Container::new()
            .background(Color::new_from_rgba_u8(0, 155, 25, 255))
            .width(Unit::Px(20.0))
            .height(Unit::Px(20.0))
            .into(),
        key: Some("twwwww".to_string()),
        props: None,
        children: vec![
            ComponentDefinition {
                component: Text::new("Hi3").into(),
                key: None,
                props: None,
                children: vec![],
            },
        ],
    }
}

pub fn app(_props: Option<Props>, children: Vec<ComponentDefinition>, id: u64) -> ComponentDefinition {
    //println!("-> app");
    println!("app id: {}", id);
    ComponentDefinition {
        component: Container::new()
            .background(Color::new_from_rgba_u8(0, 255, 0, 255))
            .width(Unit::Px(50.0))
            .height(Unit::Px(50.0))
            .into(),
        key: Some("App".to_string()),
        props: None,
        children: vec![
            ComponentDefinition {
                component: ComponentOrElement::ComponentSpec(something, "something".to_string()),
                key: None,
                props: None,
                children: vec![],
            },
        ],
    }
}

fn foo(_props: Option<Props>, children: Vec<ComponentDefinition>, id: u64) -> ComponentDefinition {
    //println!("-> foo");
    println!("foo id: {}", id);
    let background = Container::new()
        .background(Color::new_from_rgba_u8(255, 0, 0, 255))
        .width(Unit::Px(200.0))
        .height(Unit::Px(200.0));
    
    let counter = RUNTIME.get_state::<u32>(id).unwrap_or(0u32);
    
    let q = if counter % 2 == 0 {
        ComponentDefinition {
        component: Text::new(format!("EQUAL NUMBER: {}", counter).as_str()).into(),
            key: None,
            props: None,
            children: vec![],
        }
    } else {
        ComponentDefinition {
            component: Empty::new().into(),
            key: None,
            props: None,
            children: vec![],
        }
    };
    
    ComponentDefinition {
        component: background.into(),
        key: None,
        props: None,
        children: vec![
            ComponentDefinition {
                component: Text::new(format!("Hello, world 1! \n Count: {}", counter).as_str()).into(),
                key: None,
                props: None,
                children: vec![],
            },
            q,
            ComponentDefinition {
                component: ComponentOrElement::ComponentSpec(app, "app".to_string()),
                key: None,
                props: None,
                children: vec![],
            },
           ComponentDefinition {
               component: ComponentOrElement::ComponentSpec(twwwww, "twwwww".to_string()),
               key: None,
               props: None,
               children: vec![],
           },
            ComponentDefinition {
                component: Text::new("Hello, world 2!").into(),
                key: None,
                props: None,
                children: vec![],
            }
        ],
    }
}

impl Component for Test1 {
    fn view(_props: Option<&Props>, key: Option<String>) -> ComponentDefinition {
        //println!("-> Test1");\
        ComponentDefinition {
            component: ComponentOrElement::ComponentSpec(foo, "foo".to_string()),
            key,
            props: None,
            children: vec![],
        }
    }
}

struct Hello {}

/*impl Component<u64, u64> for Hello {
    fn view(&self, props: Option<&Props>, key: Option<String>) -> Element {
        if RUNTIME.get_state::<u32>(3).is_none() {
            RUNTIME.set_state(3, 0u32);
        }

        let x: u32 = RUNTIME.get_state(3).unwrap();
        let mut container = Container::new().add_child(Element::Text(Text::new(format!("Counter: {}", x))));

        println!("here: {}", container.id());

        container = container.justify_content(JustifyContent::Center);
        container = container.align_items(AlignItems::Center);
        container = container.flex_direction(FlexDirection::Column);

        let mut custom_component = oku::elements::component::Component::new();
        custom_component = custom_component.add_child(Element::Container(container));

        custom_component.add_update_handler(Arc::new(|msg, state, id: u64| {
            //let mut example: u64 = 234;
            //Self::update(example, &mut example);

            println!("{}", id);
            let x: u32 = RUNTIME.get_state(id).unwrap();
            RUNTIME.set_state(id, x + 1);
        }));

        Element::Component(custom_component)
    }

    fn update(message: u64, state: &mut u64) {}
}*/

fn main() {
    oku_core::oku_main_with_options(Test1::view(None, Some(String::from("foo"))), Some(OkuOptions { renderer: Wgpu }));
}
