use std::any::Any;
use std::sync::Arc;

use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::engine::events::{PointerButton, PointerMoved};
use crate::engine::events::resource_event::ResourceEvent;
use crate::engine::renderer::renderer::Renderer;
use crate::user::components::component::UpdateFn;

pub enum InternalMessage {
    RequestRedraw,
    Close,
    Confirmation,
    Resume(Arc<dyn Window>, Option<Box<dyn Renderer + Send>>),
    Resize(PhysicalSize<u32>),
    PointerButton(PointerButton),
    PointerMoved(PointerMoved),
    ProcessUserEvents,
    GotUserMessage((UpdateFn, u64, Option<String>, Box<dyn Any + Send>)),
    ResourceEvent(ResourceEvent)
}