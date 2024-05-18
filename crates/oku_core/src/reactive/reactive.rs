use crate::events::EventResult;
use once_cell::unsync::Lazy;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use tokio::sync::Mutex;

pub type OnClickType = Box<dyn FnMut((u32, u32)) -> EventResult>;

pub struct Runtime {
    pub current_widget_id: u64,
    pub state: HashMap<u64, Box<dyn Any>>,
    pub click_handlers: HashMap<u64, OnClickType>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub(crate) fn new() -> Self {
        Self {
            current_widget_id: 0,
            state: Default::default(),
            click_handlers: Default::default(),
        }
    }

    pub(crate) fn get_current_widget_id() -> u64 {
        RUNTIME.with_borrow(|runtime| runtime.current_widget_id)
    }

    pub(crate) fn has_state(key: u64) -> bool {
        RUNTIME.with_borrow(|runtime| runtime.state.contains_key(&key))
    }
    pub(crate) fn get_state<T: Clone + 'static>(key: u64) -> Option<T> {
        RUNTIME.with_borrow(|runtime| {
            let context = runtime.state.get(&key).and_then(|val| val.downcast_ref::<T>()).cloned();
            context
        })
    }

    pub(crate) fn set_state<T: Clone + 'static>(key: u64, value: T) {
        RUNTIME.with_borrow_mut(|runtime| {
            runtime.state.insert(key, Box::new(value.clone()));
        })
    }

    pub(crate) fn get_click_handler(key: u64) -> Option<OnClickType> {
        RUNTIME.with_borrow_mut(|runtime| runtime.click_handlers.remove(&key))
    }

    pub(crate) fn set_click_handler(key: u64, value: OnClickType) {
        RUNTIME.with_borrow_mut(|runtime| {
            runtime.click_handlers.insert(key, value);
        })
    }
}

thread_local! {
    pub static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::new());
}
