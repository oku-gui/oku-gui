use oku::components::ComponentSpecification;
use oku::components::{Component, UpdateResult};
use oku::elements::text_input::TextInput;
use oku::elements::ElementStyles;
use oku::oku_main_with_options;
use oku::style::FlexDirection;
use oku::OkuOptions;
use oku_core::elements::{Container, Font, Text};
use oku_core::events::Message;
use oku_core::events::{Event, OkuMessage};
use oku_core::RendererType;
use oku_core::resource_manager::ResourceIdentifier;

#[derive(Default, Copy, Clone)]
pub struct TextState {}

const FONT: &str =
    "https://github.com/google/material-design-icons/raw/refs/heads/master/variablefont/MaterialSymbolsOutlined%5BFILL%2CGRAD%2Copsz%2Cwght%5D.ttf";

impl Component for TextState {
    type Props = ();

    fn view(_state: &Self, _props: &Self::Props, _children: Vec<ComponentSpecification>) -> ComponentSpecification {
        Container::new()
            .flex_direction(FlexDirection::Column)
            .push(Text::new("Hello, World!").id("hello_text"))
            .push(Font::new(ResourceIdentifier::Url(FONT.to_string())))
            .push(Text::new("search home").font_family("Material Symbols Outlined").font_size(24.0))
            .push(TextInput::new("Test").flex_direction(FlexDirection::Column).id("text_input"))
            .component()
    }

    fn update(_state: &mut Self, _props: &Self::Props, event: Event) -> UpdateResult {
        println!("Source: {:?}", event.target);
        if let Message::OkuMessage(OkuMessage::TextInputChanged(new_val)) = event.message {
            println!("new text: {}", new_val);
        }

        UpdateResult::new()
    }
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    oku_main_with_options(
        TextState::component().into(),
        Some(OkuOptions {
            renderer: RendererType::default(),
            window_title: "text".to_string(),
        }),
    );
}
