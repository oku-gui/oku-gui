use crate::application::Props;
use crate::elements::element::Element;
use crate::reactive::reactive;

pub trait Component<State = (), Message = ()>
where
    State: Clone + Send + Sized + 'static,
{
    fn view(&self, props: Option<&Props>, children: Vec<Element>, key: Option<String>) -> Element;

    fn get_state(&self) -> Option<State> {
        reactive::Runtime::get_state(0)
    }

    fn set_state(&self, value: State) {
        reactive::Runtime::set_state(0, value);
    }

    #[allow(unused_variables)]
    fn update(&self, message: Message, state: &mut State) {}
}
