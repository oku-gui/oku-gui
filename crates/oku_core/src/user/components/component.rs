use crate::user::components::props::Props;
use crate::user::elements::element::Element;
use crate::engine::events::Message;
use std::any::{Any, TypeId};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use crate::PinnedFutureAny;

pub type ViewFn = fn(
    data: &(dyn Any + Send),
    props: Option<Props>,
    children: Vec<ComponentSpecification>,
    id: u64,
) -> (Box<dyn Any + Send>, ComponentSpecification, Option<UpdateFn>);


#[derive(Default)]
pub struct UpdateResult {
    pub propagate: bool,
    pub result: Option<PinnedFutureAny>
}

impl UpdateResult {
   pub fn new(propagate: bool, future: Option<PinnedFutureAny>) -> UpdateResult {
       UpdateResult {
           propagate,
           result: future
       }
   }
}

pub type UpdateFn = fn(state: &mut Box<dyn Any + Send>, id: u64, message: Message, source_element_id: Option<String>) -> UpdateResult;

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
    pub children: Vec<ComponentSpecification>,
}

#[macro_export]
macro_rules! component {
    // Match for an associated function or method of a struct
    ($path:path) => {
        {
            let name = $path;
            ComponentOrElement::ComponentSpec(
                name,
                std::any::type_name_of_val(&name).to_string(),
                name.type_id(),
            )
        }
    };

    // Match for an identifier
    ($name:ident) => {
        ComponentOrElement::ComponentSpec(
            $name,
            std::any::type_name_of_val(&$name).to_string(),
            $name.type_id(),
        )
    };
}

pub trait Component<State = (), Message = ()>
where
    State: Clone + Send + Sized + 'static,
{
    fn view(&self, props: Option<Props>, children: Vec<ComponentSpecification>, id: u64) -> ComponentSpecification;

    fn update(&self, id: u64, message: crate::engine::events::Message);
}
