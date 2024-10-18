use chrono::{DateTime, Utc};
use crate::platform::resource_manager::identifier::ResourceIdentifier;
use crate::platform::resource_manager::resource_data::ResourceData;

pub struct ImageResource {
    pub common_data: ResourceData,
    pub width: u32,
    pub height: u32,
}

impl ImageResource {
    pub(crate) fn new(width: u32, height: u32, data: ResourceData) -> Self {
        ImageResource {
            common_data: data,
            width,
            height,
        }
    }
}
