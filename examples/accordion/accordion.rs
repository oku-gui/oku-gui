mod accordion_component;

use oku::user::components::component::ComponentOrElement;
use oku::user::components::component::ComponentSpecification;
use oku::user::components::component::UpdateFn;
use oku::user::components::props::Props;
use oku::user::elements::container::Container;
use oku::user::elements::text::Text;

use oku::RendererType::Wgpu;
use oku::{component, oku_main_with_options, OkuOptions};
use std::any::Any;
use crate::accordion_component::{accordion, accordion_content, accordion_header};

pub fn app(
    _props: Option<Props>,
    _children: Vec<ComponentSpecification>,
    _id: u64,
) -> (ComponentSpecification, Option<UpdateFn>) {

    /*
    <accordion>
        <accordion_header>
            Text("Header")
        </accordion_header>

        <accordion_content>
            Text("Content")
        </accordion_content>
    </accordion>
    */

    let root = ComponentSpecification {
        component: Container::new().into(),
        key: None,
        props: None,
        children: vec![
            ComponentSpecification {
                component: component!(accordion),
                key: None,
                props: None,
                children: vec![
                    ComponentSpecification {
                        component: component!(accordion_header),
                        key: None,
                        props: None,
                        children: vec![
                            Text::new("Header").font_size(24.0).into()
                        ],
                    },
                    ComponentSpecification {
                        component: component!(accordion_content),
                        key: None,
                        props: None,
                        children: vec![
                            Text::new("Content").font_size(16.0).into()
                        ],
                    },
                ],
            },
        ],
    };
    (root, None)
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
