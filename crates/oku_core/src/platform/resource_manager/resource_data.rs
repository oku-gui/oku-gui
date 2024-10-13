use crate::platform::resource_manager::identifier::ResourceIdentifier;
use chrono::{DateTime, Utc};

pub struct ResourceData {
    pub(crate) resource_identifier: ResourceIdentifier,
    expiration_time: Option<DateTime<Utc>>,
    pub(crate) data: Option<Vec<u8>>,
}

impl ResourceData {
    pub(crate) fn new(resource_identifier: ResourceIdentifier, expiration_time: Option<DateTime<Utc>>) -> Self {
        match &resource_identifier {
            ResourceIdentifier::Url(url) => {
                let a = Some(Box::pin(async {
                    let res = reqwest::get(url).await;
                    let bytes = res.unwrap().bytes().await.ok();
                    bytes
                }));
            }
            ResourceIdentifier::File(file) => {}
        }

        ResourceData {
            resource_identifier,
            expiration_time,
            data: None,
        }
    }
}
