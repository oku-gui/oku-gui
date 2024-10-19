use oku::user::components::component::ComponentSpecification;
use oku::user::components::props::Props;
use oku::user::elements::container::Container;
use oku::user::elements::text::Text;
use oku_core::engine::events::{ButtonSource, ElementState, Message, MouseButton};

use oku::oku_main_with_options;
use oku_core::engine::events::OkuEvent::PointerButtonEvent;
use oku_core::user::components::component::{Component, ComponentId, UpdateResult};
use oku_core::user::elements::element::Element;
use oku_core::OkuOptions;
use oku_core::RendererType::Wgpu;

#[derive(Default, Copy, Clone)]
pub struct Counter {
    count: u64,
}

impl Component for Counter {
    fn view(
        state: &Self,
        _props: Option<Props>,
        _children: Vec<ComponentSpecification>,
        id: ComponentId,
    ) -> ComponentSpecification {
        let button = Container::new().id(Some("increment".to_string()));

        let button_label = Text::new("increment").id(Some("increment".to_string()));

        let big_text = Text::new("

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed pellentesque aliquam massa vel viverra. Maecenas ut ante euismod, cursus erat vel, iaculis massa. Morbi varius nunc sit amet lacus auctor maximus. Integer sem nibh, fermentum non lectus eget, tempor condimentum risus. Praesent porta erat tortor, at vehicula ligula faucibus et. Etiam fringilla mauris et dolor bibendum volutpat. Duis semper mauris rhoncus efficitur fermentum. Vestibulum placerat fringilla libero. Etiam quis fermentum felis, eu faucibus leo. Vestibulum quis eleifend massa. Nullam scelerisque id orci pretium mollis. Donec ornare congue sapien, at aliquam tortor ultrices sit amet. Curabitur a lorem tempus, dapibus dolor a, accumsan ex.

Nulla vel libero non eros eleifend aliquet eget in leo. Morbi pretium, massa sed consequat tristique, ex sapien gravida nunc, laoreet ornare metus nunc nec libero. Aliquam a finibus sem, ac cursus sem. Nunc sed blandit elit, at scelerisque tortor. Aenean consequat dui nec tortor vestibulum ultricies. In accumsan, risus quis viverra blandit, sem nunc maximus ex, vel tincidunt diam lacus eu ligula. Nulla lectus velit, efficitur id neque eget, convallis fermentum purus. Nullam aliquet nisl sit amet lobortis facilisis. Maecenas enim tortor, porttitor sed tempus non, tincidunt at turpis. Duis non posuere nulla. Nullam non felis vitae purus facilisis efficitur. Aenean ut mauris ac sem egestas tristique nec a arcu. Aenean id pulvinar quam. Ut pharetra lacus ut venenatis iaculis. Suspendisse blandit ipsum quis augue faucibus imperdiet.

In id mi volutpat elit cursus scelerisque vel vitae nisl. Proin a justo at nunc venenatis consectetur non non felis. Pellentesque sed hendrerit elit. Donec iaculis blandit dolor non vestibulum. Etiam vitae dapibus mi, at malesuada risus. Sed in ipsum eu lorem sodales aliquam. Suspendisse eros eros, iaculis ullamcorper enim et, congue vulputate neque. Donec a ligula quis nisl lobortis convallis in nec neque. Suspendisse nisl elit, porta a viverra aliquam, bibendum at massa.

Ut aliquam, odio ac placerat tempus, nunc felis ultrices tortor, vel elementum nisi neque at lectus. Vivamus non nulla elit. Duis ut tempus nulla. Pellentesque cursus ex posuere mi fermentum, condimentum tristique erat consectetur. Curabitur vestibulum elit urna, a fermentum erat pellentesque vitae. Nunc convallis lobortis augue, vitae elementum tellus dignissim nec. Donec suscipit felis sagittis orci scelerisque lobortis. Nunc suscipit dolor vitae leo tempor tempus. Aenean eget lobortis sem, laoreet convallis libero. Nunc ornare ex id eros condimentum, dictum tristique quam ultricies.

Mauris risus felis, laoreet et augue vel, vulputate posuere tellus. Donec congue sapien sit amet dolor fringilla, a aliquet neque accumsan. Curabitur eu tempor ligula. Donec scelerisque ligula a risus fermentum pellentesque. Ut dictum vel augue rutrum vestibulum. Sed ut auctor arcu. Nullam lectus orci, lacinia et dictum ut, aliquet pretium neque. Nam ex sem, accumsan eget porttitor at, eleifend non lectus. Mauris viverra tortor auctor augue vestibulum, sit amet rutrum eros hendrerit. Quisque tincidunt mi id arcu molestie aliquam. Sed aliquam est purus. Vivamus porttitor varius finibus. Aliquam sollicitudin dui mauris, ut sodales orci efficitur nec. ");

        ComponentSpecification {
            component: Container::new().into(),
            key: None,
            props: None,
            children: vec![
                ComponentSpecification {
                    component: Text::new(format!("Counter: {}", state.count).as_str()).into(),
                    key: None,
                    props: None,
                    children: vec![],
                },
                ComponentSpecification {
                    component: button.into(),
                    key: None,
                    props: None,
                    children: vec![ComponentSpecification {
                        component: button_label.into(),
                        key: None,
                        props: None,
                        children: vec![],
                    }],
                },
                big_text.into()
            ],
        }
    }

    fn update(state: &mut Self, _id: ComponentId, message: Message, source_element: Option<String>) -> UpdateResult {
        println!("updating counter: Source Element: {:?}", source_element);
        if source_element.as_deref() != Some("increment") {
            return UpdateResult::default();
        }

        println!("trying to incrementing");
        if let Message::OkuMessage(PointerButtonEvent(pointer_button)) = message {
            println!("trying to incrementing 2");
            println!("event {:?}", pointer_button);
            if pointer_button.button == ButtonSource::Mouse(MouseButton::Left)
                && pointer_button.state == ElementState::Pressed
            {
                println!("incrementing state {_id}");
                state.count += 1
            }
        };

        UpdateResult::new(true, None)
    }
}

fn main() {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    oku_main_with_options(
        ComponentSpecification {
            component: Container::new().into(),
            key: None,
            props: None,
            children: vec![
                ComponentSpecification {
                    component: Counter::component(),
                    key: None,
                    props: None,
                    children: vec![],
                },
                ComponentSpecification {
                    component: Counter::component(),
                    key: None,
                    props: None,
                    children: vec![],
                },
            ],
        },
        Some(OkuOptions { renderer: Wgpu }),
    );
}
