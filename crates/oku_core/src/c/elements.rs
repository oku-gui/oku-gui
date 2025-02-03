use std::ffi::{c_char, CStr};
use crate::c::{ComponentOrElement, ComponentSpecification, ElementBox};
use crate::c::style::{default_styles, Style};
use crate::elements::element::Element;

#[repr(C)]
struct Text {
    text: *const c_char,
    style: Style,
}

impl Text {

    #[no_mangle]
    pub extern "C" fn new_text(text: *const c_char) -> Text {
        Text {
            text,
            style: default_styles(),
        }
    }

    #[no_mangle]
    pub extern "C" fn text_to_component(&self) -> ComponentSpecification {
        let text = unsafe { CStr::from_ptr(self.text as *mut c_char).to_str().unwrap() };

        let mut text = crate::elements::Text::new(text);
        text.common_element_data_mut().style = self.style.to_rust();

        ComponentSpecification {
            component: ComponentOrElement::Element(ElementBox::new(text.into())),
            key: std::ptr::null(),
            children: std::ptr::null(),
            children_count: 0,
        }
    }
}

#[repr(C)]
struct Container {
    style: Style,
    children: *const ComponentSpecification,
    children_count: usize,
}

impl Container {

    #[no_mangle]
    pub extern "C" fn new_container(children: *const ComponentSpecification, children_count: usize) -> Container {
        Container {
            style: default_styles(),
            children,
            children_count,
        }
    }

    #[no_mangle]
    pub extern "C" fn container_to_component(&self) -> ComponentSpecification {
        ComponentSpecification {
            component: ComponentOrElement::Element(ElementBox::new(crate::elements::Container::new().into())),
            key: std::ptr::null(),
            children: self.children,
            children_count: self.children_count,
        }
    }
}