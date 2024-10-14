use chrono::{DateTime, Utc};
use crate::platform::resource_manager::identifier::ResourceIdentifier;
use crate::platform::resource_manager::resource_data::ResourceData;

pub struct ImageResource {
    pub common_data: ResourceData,
}

impl ImageResource {
    pub(crate) fn new(resource: &ResourceIdentifier, expiration_time: Option<DateTime<Utc>>) -> Self {
        ImageResource {
            common_data: ResourceData::new(resource.clone(), expiration_time),
        }
    }
}
