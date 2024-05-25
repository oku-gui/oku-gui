use once_cell::sync::Lazy;

use std::any::{type_name, Any};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct Runtime {
    pub current_widget_id: u64,
    pub state: Mutex<HashMap<u64, Box<dyn Any + Send>>>,
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
            state: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_current_widget_id() -> u64 {
        RUNTIME.current_widget_id
    }

    pub fn has_state(key: u64) -> bool {
        RUNTIME.state.lock().expect("Failed to lock runtime").contains_key(&key)
    }

    pub fn get_state<T: Clone + Send + Sized + 'static>(key: u64) -> Option<T> {
        let state = RUNTIME.state.lock().expect("Failed to lock runtime");

        println!("type name: {}", type_name::<T>());

        state.get(&key).and_then(|val| val.downcast_ref::<T>()).cloned()
    }

    pub fn set_state<T: Clone + Send + Sized + 'static>(key: u64, value: T) {
        let mut state = RUNTIME.state.lock().expect("Failed to lock runtime");
        state.insert(key, Box::new(value));
    }
}

pub static RUNTIME: Lazy<Runtime> = Lazy::new(Runtime::new);
