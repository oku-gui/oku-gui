mod elements;
mod style;

use crate::components::props::Props;
use crate::components::UpdateResult;
use crate::events::{Event, Message, OkuMessage};
use crate::oku_main_with_options;
use crate::reactive::state_store::StateStoreItem;

use crate::geometry::Point;
pub use crate::RendererType;
use std::any::{Any, TypeId};
use std::ffi::{c_char, c_void, CStr, CString};
use std::ptr::null;
use winit::event::{ButtonSource, ElementState, Force, MouseButton, PointerSource};

#[repr(C)]
#[derive(Clone, Debug)]
pub struct ElementBox {
    internal: Box<crate::elements::ElementBox>
}

impl ElementBox {
    fn new(element: crate::elements::ElementBox) -> Self {
        Self {
            internal: Box::new(element)
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub enum ComponentOrElement {
    ComponentSpec(ComponentData),
    Element(ElementBox),
}

pub type CViewFn = extern "C" fn(state: *const c_void) -> ComponentSpecification;
pub type CUpdateFn = extern "C" fn(state: *const c_void, event: CEvent) -> CUpdateResult;

#[repr(C)]
pub struct ByteBox {
    pub(crate) data: Box<Vec<u8>>,
}

impl ByteBox {
    #[no_mangle]
    pub extern "C" fn new_byte_box(data: *const u8, size: usize) -> Self {
        let mut vec_data = Vec::new();

        if !data.is_null() {
            let slice = unsafe { std::slice::from_raw_parts(data, size) };
            vec_data.extend_from_slice(slice);
        }

        Self {
            data: Box::new(vec_data),
        }
    }
}

pub type CDefaultStateFn = extern "C" fn() -> ByteBox;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct ComponentData {
    pub default_state: extern "C" fn() -> ByteBox,
    pub default_props: extern "C" fn() -> ByteBox,
    pub view_fn: CViewFn,
    pub update_fn: CUpdateFn,
    /// A unique identifier for view_fn.
    pub tag: *const c_char,
}

/*fn ptr_to_any(ptr: *const u8, size: usize) -> Box<dyn Any + Send> {
    let mut data = Box::new(Vec::new());

    if !ptr.is_null() {
        let slice = unsafe { std::slice::from_raw_parts(ptr, size) };
        data.extend_from_slice(slice);
    }

    Box::new(data)
}*/

/*fn _default_state() -> ByteBox {
    ByteBox::new_byte_box(std::ptr::null(), 0)
}*/

fn dummy_props() -> Props {
    Props::new(())
}

impl ComponentData {
    fn to_rust(self) -> crate::components::ComponentData {
        crate::components::ComponentData {
            is_c: true,
            default_state: unsafe { std::mem::transmute(self.default_state) },
            default_props: dummy_props,
            view_fn: unsafe { std::mem::transmute(self.view_fn) },
            update_fn: unsafe { std::mem::transmute(self.update_fn) },
            tag: unsafe { CStr::from_ptr(self.tag as *mut c_char).to_str().unwrap().to_string() },
            type_id: TypeId::of::<u8>(),
        }
    }

    #[no_mangle]
    pub extern "C" fn component(self) -> ComponentSpecification {
        ComponentSpecification {
            component: ComponentOrElement::ComponentSpec(self),
            key: std::ptr::null(),
            //props: None,
            children: std::ptr::null(),
            children_count: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct ComponentSpecification {
    pub component: ComponentOrElement,
    pub key: *const c_char,
    //pub props: Option<Props>,
    pub children: *const ComponentSpecification,
    pub children_count: usize,
}

impl ComponentSpecification {
    pub(crate) fn to_rust(self) -> crate::components::ComponentSpecification {
        let component_or_element = match self.component {
            ComponentOrElement::ComponentSpec(component_data) => {
                crate::components::ComponentOrElement::ComponentSpec(component_data.to_rust())
            }
            ComponentOrElement::Element(element) => {
                let element_box = *element.internal.clone();
                crate::components::ComponentOrElement::Element(element_box)
            }
        };

        let mut children: Vec<crate::components::ComponentSpecification> = Vec::new();

        if !self.children.is_null() {
            let slice = unsafe { std::slice::from_raw_parts(self.children, self.children_count) };

            for child in slice {
                children.push(child.clone().to_rust());
            }
        }

        crate::components::ComponentSpecification {
            component: component_or_element,
            key: if self.key.is_null() { None } else { None },
            props: None,
            children,
        }
    }
}

#[repr(C)]
pub struct OkuOptions {
    pub renderer: RendererType,
    window_title: *const c_char,
}

impl OkuOptions {
    fn to_rust(&self) -> crate::OkuOptions {
        crate::OkuOptions {
            renderer: self.renderer,
            window_title: unsafe { CStr::from_ptr(self.window_title as *mut c_char).to_str().unwrap().to_string() },
        }
    }
}

#[no_mangle]
extern "C" fn oku_main(application: ComponentSpecification, options: *const OkuOptions) {
    let options = if options.is_null() { None } else { Some(unsafe { &*options }) };
    let q = options.map(|options| options.to_rust());
    let x = application.to_rust();
    oku_main_with_options(x, q);
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceId(i64);

#[repr(C)]
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum CElementState {
    Pressed,
    Released,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CDeviceId(i64);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CPointerButton {
    pub has_device_id: bool,
    pub device_id: CDeviceId,
    pub state: CElementState,

    /// The position of the pointer when the button was pressed.
    ///
    /// ## Platform-specific
    ///
    /// - **Orbital: Always emits `(0., 0.)`.
    /// - **Web:** Doesn't take into account CSS [`border`], [`padding`], or [`transform`].
    ///
    /// [`border`]: https://developer.mozilla.org/en-US/docs/Web/CSS/border
    /// [`padding`]: https://developer.mozilla.org/en-US/docs/Web/CSS/padding
    /// [`transform`]: https://developer.mozilla.org/en-US/docs/Web/CSS/transform
    pub position: Point,

    pub button: CButtonSource,
    pub primary: bool,
}

#[repr(C)]
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum CMouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CFingerId(pub(crate) usize);


#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CButtonSource {
    Mouse(CMouseButton),
    /// See [`PointerSource::Touch`] for more details.
    ///
    /// ## Platform-specific
    ///
    /// **macOS:** Unsupported.
    Touch {
        finger_id: CFingerId,
        has_force: bool,
        force: CForce,
    },
    Unknown(u16),
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CForce {
    /// On iOS, the force is calibrated so that the same number corresponds to
    /// roughly the same amount of pressure on the screen regardless of the
    /// device.
    Calibrated {
        /// The force of the touch, where a value of 1.0 represents the force of
        /// an average touch (predetermined by the system, not user-specific).
        ///
        /// The force reported by Apple Pencil is measured along the axis of the
        /// pencil. If you want a force perpendicular to the device, you need to
        /// calculate this value using the `altitude_angle` value.
        force: f64,
        /// The maximum possible force for a touch.
        ///
        /// The value of this field is sufficiently high to provide a wide
        /// dynamic range for values of the `force` field.
        max_possible_force: f64,
    },
    /// If the platform reports the force as normalized, we have no way of
    /// knowing how much pressure 1.0 corresponds to – we know it's the maximum
    /// amount of force, but as to how much force, you might either have to
    /// press really really hard, or not hard at all, depending on the device.
    Normalized(f64),
}

impl CButtonSource {
    fn from_button_source(p0: &ButtonSource) -> CButtonSource {
        match p0 {
            ButtonSource::Mouse(p1) => CButtonSource::Mouse(match p1 {
                MouseButton::Left => CMouseButton::Left,
                MouseButton::Right => CMouseButton::Right,
                MouseButton::Middle => CMouseButton::Middle,
                MouseButton::Other(p2) => CMouseButton::Other(*p2),
                MouseButton::Back => CMouseButton::Back,
                MouseButton::Forward => CMouseButton::Forward,
            }),
            ButtonSource::Touch { finger_id, force } => CButtonSource::Touch {
                finger_id: CFingerId(unsafe {std::mem::transmute(*finger_id)}),
                has_force: force.is_some(),
                force: match force {
                    Some(force) => {
                        match force {
                            Force::Calibrated { force, max_possible_force } => CForce::Calibrated {
                                force: *force,
                                max_possible_force: *max_possible_force,
                            },
                            Force::Normalized(force) => CForce::Normalized(*force),
                        }
                    }
                    None => CForce::Calibrated {
                        force: 0.0,
                        max_possible_force: 0.0,
                    }
                }
            },
            ButtonSource::Unknown(p1) => CButtonSource::Unknown(*p1),
        }
    }
}

#[repr(C)]
pub enum COkuMessage {
    Initialized,
    PointerButtonEvent(CPointerButton),
    Unsupported,
}

impl CMessage {

    pub(crate) fn from_message(message: &Message) -> Self {
        match message {
            Message::OkuMessage(oku_message) => {
                match oku_message {
                    OkuMessage::Initialized => CMessage::OkuMessage(COkuMessage::Initialized),
                    OkuMessage::PointerButtonEvent(pointer_button) => {
                        CMessage::OkuMessage(COkuMessage::PointerButtonEvent(CPointerButton {
                            has_device_id: pointer_button.device_id.is_some(),
                            device_id: pointer_button.device_id.map(|id| unsafe {std::mem::transmute(id) }).unwrap_or(CDeviceId(0)),
                            state: match pointer_button.state {
                                ElementState::Pressed => CElementState::Pressed,
                                ElementState::Released => CElementState::Released,
                            },
                            position: pointer_button.position,
                            button: CButtonSource::from_button_source(&pointer_button.button),
                            primary: pointer_button.primary,
                        }))
                    }
                    OkuMessage::KeyboardInputEvent(_) => CMessage::OkuMessage(COkuMessage::Unsupported),
                    OkuMessage::PointerMovedEvent(_) => CMessage::OkuMessage(COkuMessage::Unsupported),
                    OkuMessage::MouseWheelEvent(_) => CMessage::OkuMessage(COkuMessage::Unsupported),
                    OkuMessage::TextInputChanged(_) => CMessage::OkuMessage(COkuMessage::Unsupported),
                    OkuMessage::Unsupported => CMessage::OkuMessage(COkuMessage::Unsupported),
                }
            }
            Message::UserMessage(user_message) => {
                CMessage::UserMessage(user_message.downcast_ref::<ByteBox>().unwrap().data.as_ptr() as *const c_void)
            }
        }
    }

}

#[repr(C)]
pub struct CUpdateResult {
    /// Propagate oku_events to the next element. True by default.
    pub propagate: bool,
    /// Prevent default event handlers from running when an oku_event is not explicitly handled.
    /// False by default.
    pub prevent_defaults: bool,
    pub has_result_message: bool,
    pub(crate) result_message: COkuMessage,
}

impl CUpdateResult {

    pub(crate) fn to_rust(&self) -> UpdateResult {
        UpdateResult {
            propagate: self.propagate,
            future: None,
            prevent_defaults: self.prevent_defaults,
            result_message: None,
        }
    }

    #[no_mangle]
    pub extern "C" fn new_update_result() -> CUpdateResult {
        CUpdateResult {
            propagate: true,
            prevent_defaults: false,
            has_result_message: false,
            result_message: COkuMessage::Unsupported,
        }
    }
}


#[repr(C)]
pub struct CEvent {
    /// The id of the element that triggered this event.
    pub target: *const c_char,
    /// The id of an element who is listening to this event.
    pub current_target: *const c_char,
    pub message: CMessage,
}

impl Drop for CEvent {
    fn drop(&mut self) {
        unsafe {
            if !self.target.is_null() {
                drop(CString::from_raw(self.target as *mut c_char));
            }
            if !self.current_target.is_null() {
                drop(CString::from_raw(self.current_target as *mut c_char));
            }
        }
    }
}

#[repr(C)]
pub enum CMessage {
    OkuMessage(COkuMessage),
    UserMessage(*const c_void),
}

impl CEvent {

    pub(crate) unsafe  fn from_event(event: Event) -> Self {
        let target: *const c_char = if let Some(target) = event.target {
            CString::new(target.as_str()).unwrap().into_raw()
        } else {
            null()
        };
        let current_target: *const c_char = if let Some(current_target) = event.current_target {
            CString::new(current_target.as_str()).unwrap().into_raw()
        } else {
            null()
        };
        Self {
            target,
            current_target,
            message: CMessage::from_message(&event.message),
        }
    }

}