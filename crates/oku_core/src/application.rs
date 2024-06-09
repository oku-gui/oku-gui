use std::any::Any;

pub struct Props {
    pub data: Box<dyn Any + Send>,
}

impl Props {
    pub fn get_data<T: 'static>(&self) -> Option<&T> {
        self.data.downcast_ref::<T>()
    }
}
