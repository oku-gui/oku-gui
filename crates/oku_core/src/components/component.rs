use crate::application::Props;
use crate::elements::element::Element;

pub trait Component {
    fn view(&self, props: Option<&Props>, children: Vec<Element>, key: Option<String>) -> Element;
}
