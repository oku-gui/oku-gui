use std::collections::HashSet;
use crate::elements::element::ElementBox;
use crate::elements::{Container, Text};
use crate::reactive::element_id::reset_unique_element_id;
use crate::reactive::state_store::StateStore;
use crate::reactive::tree::diff_trees;
use parley::FontContext;
use crate::components::{Component, ComponentId, ComponentSpecification, UpdateResult};
use crate::components::component::ComponentOrElement;
use crate::events::Event;
use crate::reactive::element_state_store::ElementStateStore;
use crate::{GlobalState, ReactiveTree};

#[test]
fn diff_trees_same_tag_same_id_are_equal() {
    reset_unique_element_id();
    
    let mut font_context = FontContext::new();

    let initial_view = Container::new().component().push(Text::new("Foo").component());
    let updated_view = Container::new().component().push(Text::new("Foo").component());

    let root_element: ElementBox = Container::new().into();

    let mut user_state = StateStore::default();
    let mut element_state = ElementStateStore::default();
    let mut global_state = GlobalState::from(Box::new(()));
    
    let initial_tree = diff_trees(
        initial_view,
        root_element.clone(),
        None,
        &mut user_state,
        &mut global_state,
        &mut element_state,
        &mut font_context,
        false
    );

    let updated_tree = diff_trees(
        updated_view,
        root_element.clone(),
        Some(&initial_tree.component_tree),
        &mut user_state,
        &mut global_state,
        &mut element_state,
        &mut font_context,
        false
    );

    let initial_id = &initial_tree.component_tree.children[0].children[0].id;
    let updated_id = &updated_tree.component_tree.children[0].children[0].id;

    assert_eq!(initial_id, updated_id, "Elements with identical content tags and positions have the same id.");
}

#[test]
fn diff_trees_after_one_iteration_adjacent_nodes_different_ids() {
    let mut font_context = FontContext::new();
    reset_unique_element_id();

    let root_node_1 = Container::new().component().push(Text::new("Foo").component());
    let root_node_2 = Container::new()
        .component()
        .push(Text::new("Foo").component())
        .push(Text::new("Bar").component());

    let root_element: ElementBox = Container::new().into();
    let mut user_state = StateStore::default();
    let mut element_state = ElementStateStore::default();
    let mut global_state = GlobalState::from(Box::new(()));

    let tree_1 = diff_trees(
        root_node_1,
        root_element.clone(),
        None,
        &mut user_state,
        &mut global_state,
        &mut element_state,
        &mut font_context,
        false
    );

    let tree_2 = diff_trees(
        root_node_2,
        root_element.clone(),
        Some(&tree_1.component_tree),
        &mut user_state,
        &mut global_state,
        &mut element_state,
        &mut font_context,
        false
    );

    let initial_id = &tree_1.component_tree.children[0].children[0].id;
    let updated_id = &tree_2.component_tree.children[0].children[1].id;

    assert_ne!(initial_id, updated_id, "Elements in different positions should have different ids.");
}

#[test]
fn remove_unused_element_state_after_removal_is_state_deleted() {
    let mut font_context = FontContext::new();
    reset_unique_element_id();

    let root_component_1 = Container::new().component().push(Text::new("Foo").component().key("key_1"));
    let root_component_2 = Container::new().component();
    let root_element: ElementBox = Container::new().into();

    let mut reactive_tree = ReactiveTree::default();
    let mut global_state = GlobalState::from(Box::new(()));
    
    let tree_1 = diff_trees(
        root_component_1,
        root_element.clone(),
        None,
        &mut reactive_tree.user_state,
        &mut global_state,
        &mut reactive_tree.element_state,
        &mut font_context,
        false
    );

    let text_element_id = tree_1.component_tree.children[0].children[0].id;

    reactive_tree.component_tree = Some(tree_1.component_tree);
    reactive_tree.element_tree = Some(tree_1.element_tree.internal);
    reactive_tree.element_ids = tree_1.element_ids;
    reactive_tree.component_ids = tree_1.component_ids;

    let old_element_ids: HashSet<ComponentId> = reactive_tree.element_ids.clone();
    
    let tree_2 = diff_trees(
        root_component_2,
        root_element.clone(),
        Some(reactive_tree.component_tree.as_ref().unwrap()),
        &mut reactive_tree.user_state,
        &mut global_state,
        &mut reactive_tree.element_state,
        &mut font_context,
        false
    );

    reactive_tree.component_tree = Some(tree_2.component_tree);
    reactive_tree.element_tree = Some(tree_2.element_tree.internal);
    reactive_tree.element_ids = tree_2.element_ids;
    reactive_tree.component_ids = tree_2.component_ids;
    
    reactive_tree.element_state.remove_unused_state(&old_element_ids, &reactive_tree.element_ids);
    
    assert!(!reactive_tree.element_state.storage.contains_key(&text_element_id),
            "Unmounted elements should have their state removed."
    );
}

#[derive(Default)]
struct DummyComponent {
}

impl Component<()> for DummyComponent {
    type Props = ();

    fn view(state: &Self, global_state: &(), props: &Self::Props, children: Vec<ComponentSpecification>) -> ComponentSpecification {
        Text::new("dummy").component()
    }

    fn update(_state: &mut Self, global_state: &mut (), _props: &Self::Props, _message: Event) -> UpdateResult {
        UpdateResult::default()
    }
}

#[test]
fn remove_unused_component_state_after_removal_is_state_deleted() {
    let mut font_context = FontContext::new();
    reset_unique_element_id();

    let root_component_1 = Container::new().component()
        .push(Text::new("Foo").component().key("key_1"))
        .push(DummyComponent::component());
    let root_component_2 =  Container::new().component()
        .push(Text::new("Foo").component().key("key_1"));
    let root_element: ElementBox = Container::new().into();

    let mut reactive_tree = ReactiveTree::default();
    let mut global_state = GlobalState::from(Box::new(()));
    
    let tree_1 = diff_trees(
        root_component_1,
        root_element.clone(),
        None,
        &mut reactive_tree.user_state,
        &mut global_state,
        &mut reactive_tree.element_state,
        &mut font_context,
        false
    );

    let dummy_component_id = tree_1.component_tree.children[0].children[1].id;

    reactive_tree.component_tree = Some(tree_1.component_tree);
    reactive_tree.element_tree = Some(tree_1.element_tree.internal);
    reactive_tree.element_ids = tree_1.element_ids;
    reactive_tree.component_ids = tree_1.component_ids;

    let old_component_ids: HashSet<ComponentId> = reactive_tree.component_ids.clone();

    let tree_2 = diff_trees(
        root_component_2,
        root_element.clone(),
        Some(reactive_tree.component_tree.as_ref().unwrap()),
        &mut reactive_tree.user_state,
        &mut global_state,
        &mut reactive_tree.element_state,
        &mut font_context,
        false
    );

    reactive_tree.component_tree = Some(tree_2.component_tree);
    reactive_tree.element_tree = Some(tree_2.element_tree.internal);
    reactive_tree.element_ids = tree_2.element_ids;
    reactive_tree.component_ids = tree_2.component_ids;

    reactive_tree.user_state.remove_unused_state(&old_component_ids, &reactive_tree.component_ids);

    assert!(!reactive_tree.user_state.storage.contains_key(&dummy_component_id),
            "Unmounted components should have their state removed."
    );
}

#[test]
fn diff_trees_after_one_iteration_same_key_different_position_same_id() {
    let mut font_context = FontContext::new();
    reset_unique_element_id();

    let root_node_1 = Container::new().component().push(Text::new("Foo").component().key("key_1"));
    let root_node_2 = Container::new()
        .component()
        .push(Text::new("Bar").component())
        .push(Text::new("Foo").component().key("key_1"));

    let root_element: ElementBox = Container::new().into();
    let mut user_state = StateStore::default();
    let mut element_state = ElementStateStore::default();
    let mut global_state = GlobalState::from(Box::new(()));
    
    let tree_1 = diff_trees(
        root_node_1,
        root_element.clone(),
        None,
        &mut user_state,
        &mut global_state,
        &mut element_state,
        &mut font_context,
        false
    );

    let tree_2 = diff_trees(
        root_node_2,
        root_element.clone(),
        Some(&tree_1.component_tree),
        &mut user_state,
        &mut global_state,
        &mut element_state,
        &mut font_context,
        false
    );

    let initial_id = &tree_1.component_tree.children[0].children[0].id;
    let updated_id = &tree_2.component_tree.children[0].children[1].id;

    assert_eq!(initial_id, updated_id, "Elements in different positions with the same key, should have the same id.");
}
