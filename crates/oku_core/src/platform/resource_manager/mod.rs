mod identifier;
mod image;
mod resource;
mod resource_data;

use crate::engine::app_message::AppMessage;
use crate::engine::events::internal::InternalMessage;
use crate::engine::events::resource_event::ResourcEvent;
pub use crate::platform::resource_manager::identifier::ResourceIdentifier;
use crate::platform::resource_manager::image::ImageResource;
use crate::platform::resource_manager::resource::Resource;
use std::any::Any;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use tokio::sync::mpsc;

pub type ResourceFuture = Pin<Box<dyn Future<Output = Box<dyn Any + Send + Sync>> + Send + Sync>>;

pub struct ResourceManager {
    pub resource_jobs: VecDeque<ResourceFuture>,
    pub resources: HashMap<ResourceIdentifier, Resource>,
    app_sender: mpsc::Sender<AppMessage>,
}

impl ResourceManager {
    pub fn new(app_sender: mpsc::Sender<AppMessage>) -> Self {
        Self {
            resource_jobs: VecDeque::new(),
            resources: HashMap::new(),
            app_sender,
        }
    }

    pub async fn add(&mut self, resource: ResourceIdentifier) {
        if !self.resources.contains_key(&resource) {
            let image = Resource::Image(ImageResource::new(&resource, None));
            let resource_copy = resource.clone();
            self.resources.insert(resource, image);

            self.app_sender
                .send(AppMessage::new(0, InternalMessage::ResourceEvent(ResourcEvent::Added(resource_copy))))
                .await
                .expect("Failed to send added resource event");
        }
    }
}