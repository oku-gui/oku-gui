mod image;
mod identifier;
mod resource;
mod resource_data;

use std::any::Any;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc};
use crate::platform::resource_manager::identifier::ResourceIdentifier;
use crate::platform::resource_manager::resource::Resource;

pub type ResourceFuture = Pin<Box<dyn Future<Output = Box<dyn Any + Send + Sync>> + Send + Sync>>;

pub struct ResourceManager {
    pub resource_jobs: VecDeque<ResourceFuture>,
    pub resources: HashMap<ResourceIdentifier, Arc<Resource>>,
}


impl ResourceManager {
    
    pub fn new() -> Self {
        Self {
            resource_jobs: VecDeque::new(),
            resources: HashMap::new(),
        }
    }
    
}