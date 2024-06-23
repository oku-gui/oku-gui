use std::any::Any;
use std::sync::{Arc, RwLock};
use crate::components::props::Props;
use crate::elements::element::Element;
use crate::reactive::reactive::RUNTIME;

pub type ViewFn = fn (props: Option<Props>, children: Vec<ComponentDefinition>, id: u64) -> ComponentDefinition;

#[derive(Clone)]
pub enum ComponentOrElement {
    ComponentSpec(ViewFn, String),
    Element(Box<dyn Element>),
}

#[derive(Clone)]
pub struct ComponentDefinition {
    pub component: ComponentOrElement,
    pub key: Option<String>,
    pub props: Option<Props>,
    pub children: Vec<ComponentDefinition>
}

pub trait Component<State = (), Message = ()>
where
    State: Clone + Send + Sized + 'static,
{
    fn view(props: Option<&Props>, key: Option<String>) -> ComponentDefinition;

    fn get_state(&self) -> Option<State> {
        RUNTIME.get_state(0)
    }

    fn set_state(&self, value: State) {
        RUNTIME.set_state(0, value);
    }

    #[allow(unused_variables)]
    fn update(message: Message, state: &mut State) {}
}