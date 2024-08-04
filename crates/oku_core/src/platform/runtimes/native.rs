use tokio::runtime::Runtime;

pub fn create_native_runtime() -> Runtime {
    tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("Failed to create runtime")
} 