use oku::components::component::Component;
use oku::elements::container::Container;
use oku::elements::element::Element;
use oku::elements::text::Text;
use oku::Props;
use std::cell::RefCell;
use std::rc::Rc;

use oku::renderer::renderer;

fn use_state<T: Clone>(value: T) -> (impl Fn() -> T, impl FnMut(T)) {
    let val = Rc::new(RefCell::new(value));

    let state = {
        let val = val.clone();
        move || -> T { val.borrow().clone() }
    };

    let set_state = move |v: T| {
        val.replace(v);
    };

    (state, set_state)
}

struct Test1 {}

impl Component for Test1 {
    fn view(&self, _props: Option<&Props>, _children: Vec<Element>) -> Element {
        Element::Text(Text::new(String::from("Hello")))
    }
}

struct Hello {}

impl Component for Hello {
    fn view(&self, props: Option<&Props>, children: Vec<Element>) -> Element {
        let (data, mut set_data) = use_state(String::from("foo"));

        println!("data: {}", data());
        set_data(String::from("bar"));
        println!("data: {}", data());

        let my_data = props.unwrap().get_data::<u32>().unwrap();
        let mut container = Container::new().add_child(Element::Text(Text::new(format!("Hello, world! {}", my_data))));

        for child in children {
            container = container.add_child(child);
        }

        Element::Container(container)
    }
}

struct App {}

impl oku_core::Application for App {
    fn view(&self) -> Element {
        let hello = Hello {};
        let hello_props = Props {
            data: Box::new(12_u32),
        };

        let test1 = Test1 {};

        hello.view(Some(&hello_props), vec![test1.view(None, vec![])])
    }
}

fn main() {
    let application = App {};
    oku_core::main(Box::new(application));
}
