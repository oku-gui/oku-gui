mod image;

use std::default::Default;
use std::any::Any;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use tokio::sync::RwLock;
use crate::platform::resource_manager::image::ImageResource;
//use crate::user::reactive::reactive::Runtime;

pub type ResourceFuture = Pin<Box<dyn Future<Output = Box<dyn Any + Send + Sync>> + Send + Sync>>;

struct ResourceData  {
    resource_identifier: ResourceIdentifier,
    expiration_time: Option<DateTime<Utc>>,
    data: Option<Vec<u8>>,
}

impl ResourceData {
    fn new (resource_identifier: ResourceIdentifier, expiration_time: Option<DateTime<Utc>>) -> Self {
        // Make a request to get the image. async stuff here.
        // Load data.


        match &resource_identifier {
            ResourceIdentifier::Url(url) => {
                let a = Some(Box::pin(async {

                    let res = reqwest::get("https://picsum.photos/800").await;
                    let bytes = res.unwrap().bytes().await.ok();
                    //let boxed: Box<Resource> = Box::new(Default::default());

                    //boxed
                }));
                //RESOURCE_MANAGER.add_job(a);
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

enum Resource {
    Image(ImageResource)
}

impl Resource {
    
    pub fn resource_identifier(&self) -> ResourceIdentifier {
        match self {
            Resource::Image(data) => {
                data.common_data.resource_identifier.clone()
            }
        }
    }

    pub fn data(&self) -> Option<&[u8]> {
        match self {
            Resource::Image(data) => {
                data.common_data.data.as_deref()
            }
        }
    }
    
}

#[derive(Clone, Debug)]
pub enum ResourceIdentifier {
    Url(String),
    File(String),
}

pub struct ResourceManagerState {
    pub resource_jobs: VecDeque<ResourceFuture>,
    pub resources: HashMap<ResourceIdentifier, Arc<Resource>>,
}

pub struct ResourceManager {
    pub state: RwLock<ResourceManagerState>,
}

// "future.png"
// resources.get("futures.png")

impl ResourceManager{

    pub fn new() -> Self {
        Self {
            state: RwLock::new(ResourceManagerState::new()),
        }
    }

    pub async fn add_job(&mut self, job: ResourceFuture) {
        /*async move {
            let mut write_lock = self.state.write().await;
            *write_lock.resource_jobs.push_back(job);
        }*/
    }

}

impl ResourceManagerState {
    
    pub fn new() -> Self {
        ResourceManagerState {
            resource_jobs: VecDeque::new(),
            resources: HashMap::new(),
        }
    }

    pub fn add_job(&mut self, job: ResourceFuture) {
        self.resource_jobs.push_back(job)
    }
    
}

pub static RESOURCE_MANAGER: Lazy<ResourceManager> = Lazy::new(ResourceManager::new);