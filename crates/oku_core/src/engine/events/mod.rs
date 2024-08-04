use crate::MouseInput;
use std::any::Any;

pub enum EventResult {
    Stop,
    Continue,
}

pub struct ClickMessage {
    pub(crate) mouse_input: MouseInput,
    pub(crate) x: f64,
    pub(crate) y: f64,
}

pub enum OkuEvent {
    Click(ClickMessage),
}

pub enum Message {
    OkuMessage(OkuEvent),
    UserMessage(Box<dyn Any>),
}
