use std::any::Any;
use std::sync::Arc;

#[derive(Clone)]
pub struct Props {
    pub data: Arc<dyn Any + Send + Sync>,
}

unsafe impl Send for Props {}

unsafe impl Sync for Props {}

impl Props {
    pub fn get_data<T: 'static>(&self) -> Option<&T> {
        self.data.downcast_ref::<T>()
    }
}
