use crate::engine::events::Message;
use crate::components::props::Props;
use crate::elements::element::Element;
use crate::PinnedFutureAny;
use std::any::{Any, TypeId};
use std::ops::Deref;

/// A Component's view function.
pub type ViewFn = fn(
    data: &GenericUserState,
    props: Option<Props>,
    children: Vec<ComponentSpecification>,
    id: ComponentId,
) -> ComponentSpecification;

/// The result of an update.
#[derive(Default)]
pub struct UpdateResult {
    pub propagate: bool,
    pub result: Option<PinnedFutureAny>,
}

impl UpdateResult {
    pub fn new(propagate: bool, future: Option<PinnedFutureAny>) -> UpdateResult {
        UpdateResult {
            propagate,
            result: future,
        }
    }
}

/// A Component's update function.
pub type UpdateFn = fn(
    state: &mut GenericUserState,
    id: ComponentId,
    message: Message,
    source_element_id: Option<String>,
) -> UpdateResult;
pub type ComponentId = u64;

#[derive(Clone)]
pub struct ComponentData {
    pub default_state: fn() -> Box<GenericUserState>,
    pub view_fn: ViewFn,
    pub update_fn: UpdateFn,
    /// A unique identifier for view_fn.
    pub tag: String,
    /// The type id of the view function. This is currently not used.
    pub type_id: TypeId,
}

/// An enum containing either an [`Element`] or a [`ComponentData`].
#[derive(Clone)]
pub enum ComponentOrElement {
    ComponentSpec(ComponentData),
    Element(Box<dyn Element>),
}

/// A specification for components and elements.
#[derive(Clone)]
pub struct ComponentSpecification {
    pub component: ComponentOrElement,
    pub key: Option<String>,
    pub props: Option<Props>,
    pub children: Vec<ComponentSpecification>,
}

impl ComponentSpecification {
    pub fn new(component: ComponentOrElement) -> Self {
        ComponentSpecification {
            component,
            key: None,
            props: None,
            children: vec![],
        }
    }

    pub fn key(mut self, key: &str) -> Self {
        if let ComponentOrElement::Element(_) = self.component { 
            panic!("Component cannot have a key.")
        }
        self.key = Some(key.to_owned());
        self
    }

    pub fn props(mut self, props: Props) -> Self {
        self.props = Some(props);
        self
    }

    pub fn children(mut self, children: Vec<ComponentSpecification>) -> Self {
        self.children = children;
        self
    }

    pub fn push(mut self, component: ComponentSpecification) -> Self {
        self.children.push(component);
        self
    }
}

#[macro_export]
macro_rules! component {
    // Match for an associated function or method of a struct
    ($path:path) => {{
        let name = $path;
        ComponentOrElement::ComponentSpec(name, std::any::type_name_of_val(&name).to_string(), name.type_id())
    }};

    // Match for an identifier
    ($name:ident) => {
        ComponentOrElement::ComponentSpec($name, std::any::type_name_of_val(&$name).to_string(), $name.type_id())
    };
}

pub type GenericUserState = dyn Any + Send;

pub trait Component
where
    Self: 'static + Default + Send,
{
    type Props: Send + Sync;

    fn view(
        state: &Self,
        _props: Option<&Self::Props>,
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
        let props: Option<&Self::Props> = props.as_ref().map(|props| props.data.deref().downcast_ref().unwrap());

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

    fn component() -> ComponentSpecification {
        let component_data = ComponentData {
            default_state: Self::default_state,
            view_fn: Self::generic_view,
            update_fn: Self::generic_update,
            tag: std::any::type_name_of_val(&Self::generic_view).to_string(),
            type_id: Self::generic_view.type_id(),
        };

        ComponentSpecification::new(ComponentOrElement::ComponentSpec(component_data))
    }
}