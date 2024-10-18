use crate::platform::resource_manager::ResourceIdentifier;

pub enum ResourceEvent {
    Added(ResourceIdentifier),
    Loaded(ResourceIdentifier),
    UnLoaded(ResourceIdentifier),
}