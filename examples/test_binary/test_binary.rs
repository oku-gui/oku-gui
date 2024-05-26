use std::any::Any;
use oku::application::Props;
use oku::components::component::Component;
use oku::elements::container::Container;
use oku::elements::element::Element;
use oku::elements::text::Text;
use oku_core::elements::style::{AlignItems, FlexDirection, JustifyContent, Unit};
use oku_core::events::EventResult;
use oku_core::reactive::reactive;
use oku_core::reactive::reactive::RUNTIME;
use oku_core::OkuOptions;
use oku_core::RendererType::Wgpu;
use std::sync::Arc;
use oku_core::elements::standard_element::StandardElement;

struct Test1 {}

impl Component for Test1 {
    fn view(&self, _props: Option<&Props>, key: Option<String>) -> Element {
        Element::Text(Text::new(String::from("Hello")))
    }
}

struct Hello {}

impl Component<u64, u64> for Hello {
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
}

struct App {}

impl oku_core::application::Application for App {
    fn view(&self) -> Element {
        let hello = Hello {};
        let hello_props = Props {
            data: Box::new(12_u32),
        };

        hello.view(Some(&hello_props), None)
    }
}

fn main() {
    let application = App {};
    oku_core::oku_main_with_options(Box::new(application), Some(OkuOptions { renderer: Wgpu }));
}
