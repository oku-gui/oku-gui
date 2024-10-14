mod image;
mod identifier;
mod resource;
mod resource_data;

use std::any::Any;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
pub use crate::platform::resource_manager::identifier::ResourceIdentifier;
use crate::platform::resource_manager::image::ImageResource;
use crate::platform::resource_manager::resource::Resource;
use crate::platform::resource_manager::resource::Resource::Image;

pub type ResourceFuture = Pin<Box<dyn Future<Output = Box<dyn Any + Send + Sync>> + Send + Sync>>;

pub struct ResourceManager {
    pub resource_jobs: VecDeque<ResourceFuture>,
    pub resources: HashMap<ResourceIdentifier, Resource>,
}


impl ResourceManager {
    
    pub fn new() -> Self {
        Self {
            resource_jobs: VecDeque::new(),
            resources: HashMap::new(),
        }
    }

    pub fn add(&mut self, resource: ResourceIdentifier) {
        if !self.resources.contains_key(&resource) {
            let image = Resource::Image(ImageResource::new(&resource, None));
            self.resources.insert(resource, image);
        }
        println!("assets {}", self.resources.len())
    }
    
}