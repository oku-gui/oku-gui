use std::sync::atomic::{AtomicU64, Ordering};
static ATOMIC_WIDGET_ID: AtomicU64 = AtomicU64::new(0);

pub fn get_current_widget_id_counter() -> u64 {
    ATOMIC_WIDGET_ID.load(Ordering::SeqCst)
}

pub fn create_unique_widget_id() -> u64 {
    ATOMIC_WIDGET_ID.fetch_add(1, Ordering::SeqCst);

    get_current_widget_id_counter()
}

pub fn reset_unique_widget_id() {
    ATOMIC_WIDGET_ID.store(0, Ordering::SeqCst);
}
