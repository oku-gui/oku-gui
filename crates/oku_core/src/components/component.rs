use std::any::Any;
use std::sync::{Arc, RwLock};
use crate::components::props::Props;
use crate::elements::element::Element;
use crate::reactive::reactive::RUNTIME;

type view_fn = fn (props: Option<Props>, children: Vec<ComponentSpecification>, id: u64) -> ComponentSpecification;

#[derive(Clone)]
pub enum ComponentOrElement {
    ComponentSpec(view_fn),
    Element(Box<dyn Element>),
}

#[derive(Clone)]
pub struct ComponentSpecification {
    pub component: ComponentOrElement,
    pub key: Option<String>,
    pub props: Option<Props>,
    pub children: Vec<ComponentSpecification>
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