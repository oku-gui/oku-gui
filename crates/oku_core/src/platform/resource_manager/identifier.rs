use crate::PinnedFutureAny;
use crate::platform::resource_manager::image::ImageResource;
use crate::platform::resource_manager::resource::Resource;
use crate::platform::resource_manager::resource_data::ResourceData;
use crate::platform::resource_manager::ResourceIdentifier::{File, Url};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum ResourceIdentifier {
    Url(String),
    File(String),
}

impl ResourceIdentifier {
    pub async fn fetch_resource_from_resource_identifier(&self) -> Option<Resource> {
        
        match self {
            Url(url) => {

                let res = reqwest::get(url).await;
                let bytes = res.unwrap().bytes().await.ok();
                
                // Do error checking here.

                let image = image::load_from_memory(&*bytes?).unwrap();
                let width = image.width();
                let height = image.height();
                let generic_resource = ResourceData::new(self.clone(), image.as_bytes().to_vec(), None);
                
                return Some(Resource::Image(ImageResource::new(width, height, generic_resource)));
            }
            // tracing::warn!(name: "ResourceIdentifier", warning = "Resource Identifier {} not supported.");
            _ => {}
        }
        
        None
    }   
}