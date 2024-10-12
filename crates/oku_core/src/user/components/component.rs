use crate::user::components::props::Props;
use crate::user::elements::element::Element;
use crate::engine::events::Message;
use std::any::{Any, TypeId};
use crate::PinnedFutureAny;

pub type ViewFn = fn(
    data: &GenericUserState,
    props: Option<Props>,
    children: Vec<ComponentSpecification>,
    id: ComponentId,
) -> ComponentSpecification;


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

pub type UpdateFn = fn(state: &mut GenericUserState, id: ComponentId, message: Message, source_element_id: Option<String>) -> UpdateResult;
pub type ComponentId = u64;

#[derive(Clone)]
pub enum ComponentOrElement {
    ComponentSpec(fn() -> Box<GenericUserState>, ViewFn, UpdateFn, String, TypeId),
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

pub type GenericUserState = dyn Any + Send;

pub trait Component
where
    Self: 'static + Default + Send,
{
    fn view(
        state: &Self,
        _props: Option<Props>,
        _children: Vec<ComponentSpecification>,
        id: ComponentId,
    ) -> ComponentSpecification;

    fn generic_view(
        state: &GenericUserState,
        props: Option<Props>,
        children: Vec<ComponentSpecification>,
        id: ComponentId,
    ) -> ComponentSpecification {
        let casted_state: &Self = state.downcast_ref::<Self>().unwrap();

        Self::view(casted_state, props, children, id)
    }

    fn default_state() -> Box<GenericUserState> {
        Box::<Self>::default()
    }

    fn update(state: &mut Self, id: ComponentId, message: Message, source_element: Option<String>) -> UpdateResult;

    fn generic_update(
        state: &mut GenericUserState,
        id: ComponentId,
        message: Message,
        source_element: Option<String>,
    ) -> UpdateResult {
        let casted_state: &mut Self = state.downcast_mut::<Self>().unwrap();

        Self::update(casted_state, id, message, source_element)
    }

    fn component() -> ComponentOrElement {
        ComponentOrElement::ComponentSpec(
            Self::default_state,
            Self::generic_view,
            Self::generic_update,
            std::any::type_name_of_val(&Self::generic_view).to_string(),
            Self::generic_view.type_id(),
        )
    }
}