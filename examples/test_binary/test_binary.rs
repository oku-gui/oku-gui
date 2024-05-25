use oku::application::Props;
use oku::components::component::Component;
use oku::elements::container::Container;
use oku::elements::element::Element;
use oku::elements::text::Text;
use oku_core::events::EventResult;
use oku_core::reactive::reactive;
use oku_core::OkuOptions;
use oku_core::RendererType::Wgpu;

struct Test1 {}

impl Component for Test1 {
    fn view(&self, _props: Option<&Props>, _children: Vec<Element>, key: Option<String>) -> Element {
        Element::Text(Text::new(String::from("Hello")))
    }
}

struct Hello {}

impl Component for Hello {
    fn view(&self, props: Option<&Props>, children: Vec<Element>, key: Option<String>) -> Element {
        // let my_data = props.unwrap().get_data::<u32>().unwrap();
        let mut container = Container::new().add_child(Element::Text(Text::new(format!("Hello, world! {}", 5))));

        let key = 1;
        let value = 5u64;
        reactive::Runtime::set_state(key, value);

        let x: u64 = reactive::Runtime::get_state(key).unwrap();

        for child in children {
            container = container.add_child(child);
        }

        Element::Container(container)
    }
}

struct App {}

impl oku_core::application::Application for App {
    fn view(&self) -> Element {
        let hello = Hello {};
        let hello_props = Props {
            data: Box::new(12_u32),
        };

        let test1 = Test1 {};

        hello.view(Some(&hello_props), vec![test1.view(None, vec![], None)], None)
    }
}

fn main() {
    let application = App {};
    oku_core::oku_main_with_options(Box::new(application), Some(OkuOptions { renderer: Wgpu }));
}
