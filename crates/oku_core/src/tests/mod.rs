use crate::components::props::Props;
use crate::ComponentOrElement;
use std::any::Any;
use crate::{component, create_trees_from_render_specification};
use crate::components::component::ComponentSpecification;
use crate::elements::container::Container;
use crate::elements::text::Text;

#[test]
fn create_trees_from_render_specification_with_no_old_tree_generates_ids() {
    fn component(_props: Option<Props>, _children: Vec<ComponentSpecification>, id: u64) -> ComponentSpecification {
        assert_eq!(id, 1);
        Text::new("a").into()
    }

    let component_specification = ComponentSpecification {
        component: Container::new().into(),
        key: None,
        props: None,
        children: vec![
            crate::elements::text::Text::new("a").into(),
            ComponentSpecification {
                component: component!(component),
                key: None,
                props: None,
                children: vec![],
            },
            crate::elements::text::Text::new("c").into(),
        ],
    };

    let root = Container::new().into();

    create_trees_from_render_specification(component_specification, root, None);
}