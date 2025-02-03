pub(crate) mod component;
mod component_pre_order_iterator;
pub mod props;

pub use component::Component;
pub use component::ComponentId;
pub use component::ComponentSpecification;
pub use component::UpdateResult;

#[cfg(feature = "oku_c")]
pub use component::ComponentData;
#[cfg(feature = "oku_c")]
pub use component::ComponentOrElement;
#[cfg(feature = "oku_c")]
pub use component::ViewFn;
#[cfg(feature = "oku_c")]
pub use component::UpdateFn;