use crate::application::Props;
use crate::elements::standard_element::StandardElement;
use crate::reactive::reactive::RUNTIME;

#[derive(Clone)]
pub enum ComponentOrElement {
    ComponentSpec(ComponentSpecification),
    Element(Box<dyn StandardElement>),
}

#[derive(Clone)]
pub struct ComponentSpecification {
    pub component: fn (props: Option<&Props>, key: Option<String>) -> ComponentOrElement,
    pub key: Option<String>,
    pub children: Vec<ComponentOrElement>
}

pub trait Component<State = (), Message = ()>
where
    State: Clone + Send + Sized + 'static,
{
    fn view(props: Option<&Props>, key: Option<String>) -> ComponentSpecification;

    fn get_state(&self) -> Option<State> {
        RUNTIME.get_state(0)
    }

    fn set_state(&self, value: State) {
        RUNTIME.set_state(0, value);
    }

    #[allow(unused_variables)]
    fn update(message: Message, state: &mut State) {}
}