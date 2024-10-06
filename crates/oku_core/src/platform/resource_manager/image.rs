use chrono::{DateTime, Utc};
use crate::platform::resource_manager::{Resource, ResourceData};

pub struct ImageResource {
    pub common_data: ResourceData,
}

impl ImageResource {
    fn new (path: Option<String>, expiration_time: Option<DateTime<Utc>>) -> Self {
        ImageResource {
            common_data: ResourceData::new(path, expiration_time)
        }
    }
}