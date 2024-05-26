use once_cell::sync::Lazy;

use std::any::{Any};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct Runtime {
    state: Mutex<RuntimeState>,
}

struct RuntimeState {
    pub current_widget_id: u64,
    pub state: HashMap<u64, Box<dyn Any + Send>>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(RuntimeState::new()),
        }
    }

    pub fn get_current_widget_id(&self) -> u64 {
        self.state.lock().unwrap().get_current_widget_id()
    }

    pub fn has_state(&self, key: u64) -> bool {
        self.state.lock().unwrap().has_state(key)
    }

    pub fn get_state<T: Clone + Send + Sized + 'static>(&self, key: u64) -> Option<T> {
        self.state.lock().unwrap().get_state(key)
    }

    pub fn set_state<T: Clone + Send + Sized + 'static>(&self, key: u64, value: T) {
        self.state.lock().unwrap().set_state(key, value);
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeState {
    pub(crate) fn new() -> Self {
        Self {
            current_widget_id: 0,
            state: HashMap::new(),
        }
    }

    pub fn get_current_widget_id(&self) -> u64 {
        self.current_widget_id
    }

    pub fn has_state(&self, key: u64) -> bool {
        self.state.contains_key(&key)
    }

    pub fn get_state<T: Clone + Send + Sized + 'static>(&self, key: u64) -> Option<T> {
        self.state.get(&key).and_then(|val| val.downcast_ref::<T>()).cloned()
    }

    pub fn set_state<T: Clone + Send + Sized + 'static>(&mut self, key: u64, value: T) {
        self.state.insert(key, Box::new(value));
    }
}

pub static RUNTIME: Lazy<Runtime> = Lazy::new(Runtime::new);
