use std::any::{Any, TypeId};
use std::sync::{Arc, RwLock};
use crate::components::props::Props;
use crate::elements::element::Element;
use crate::events::Message;
use crate::reactive::reactive::RUNTIME;

pub type ViewFn = fn (props: Option<Props>, children: Vec<ComponentSpecification>, id: u64) -> ComponentSpecification;
pub type UpdateFn = fn (id: u64, message: Message);

#[derive(Clone)]
pub enum ComponentOrElement {
    ComponentSpec(ViewFn, String, TypeId),
    Element(Box<dyn Element>),
}

#[derive(Clone)]
pub struct ComponentSpecification {
    pub component: ComponentOrElement,
    pub key: Option<String>,
    pub props: Option<Props>,
    pub children: Vec<ComponentSpecification>
}
#[macro_export]
macro_rules! component {
    ($name:ident) => {
        ComponentOrElement::ComponentSpec($name, std::any::type_name_of_val(&$name).to_string(), $name.type_id())
    };
}

pub trait Component<State = (), Message = ()>
where
    State: Clone + Send + Sized + 'static,
{
    fn view(&self, props: Option<Props>, children: Vec<ComponentSpecification>, id: u64) -> ComponentSpecification;

/*    fn get_state(&self) -> Option<State> {
        RUNTIME.get_state(0)
    }

    fn set_state(&self, value: State) {
        RUNTIME.set_state(0, value);
    }
*/
    fn update(&self, id: u64, message: crate::events::Message);
}