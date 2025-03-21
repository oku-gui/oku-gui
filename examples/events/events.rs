#[path = "../util.rs"]
mod util;

use crate::util::setup_logging;
use oku::components::{Component, ComponentSpecification, UpdateResult};
use oku::elements::Container;
use oku::elements::ElementStyles;
use oku::events::{Event};
use oku::oku_main_with_options;
use oku::palette;
use oku::style::Position;
use oku::style::Unit;
use oku::OkuOptions;
use oku::RendererType;

#[derive(Default, Copy, Clone)]
pub struct EventsExample {}

impl Component<()> for EventsExample {
    type Props = ();

    fn view(
        _state: &Self,
        _global_state: &(),
        _props: &Self::Props,
        _children: Vec<ComponentSpecification>,
    ) -> ComponentSpecification {
        Container::new()
            .background(palette::css::RED)
            .width(Unit::Px(400.0))
            .height(Unit::Px(400.0))
            .id("red")
            .push(
                Container::new()
                    .background(palette::css::GREEN)
                    .inset(Unit::Px(50.0), Unit::Px(50.0), Unit::Px(50.0), Unit::Px(50.0))
                    .position(Position::Absolute)
                    .width(Unit::Px(200.0))
                    .height(Unit::Px(200.0))
                    .id("green")
                    .push(
                        Container::new()
                            .background(palette::css::BLUE)
                            .inset(Unit::Px(50.0), Unit::Px(50.0), Unit::Px(50.0), Unit::Px(50.0))
                            .position(Position::Absolute)
                            .width(Unit::Px(100.0))
                            .height(Unit::Px(100.0))
                            .id("blue"),
                    ),
            )
            .component()
    }

    fn update(_state: &mut Self, _global_state: &mut (), _props: &Self::Props, event: Event) -> UpdateResult {
        if event.message.clicked() {
            println!("Target: {:?}, Current Target: {:?}", event.target, event.current_target);
        }

        UpdateResult::new()
    }
}

fn main() {
    setup_logging();

    oku_main_with_options(
        EventsExample::component(),
        Box::new(()),
        Some(OkuOptions {
            renderer: RendererType::default(),
            window_title: "events".to_string(),
        }),
    );
}
