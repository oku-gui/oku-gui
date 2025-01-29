use crate::elements::element::ElementBox;
use crate::elements::{Container, Text};
use crate::reactive::element_id::reset_unique_element_id;
use crate::reactive::state_store::StateStore;
use crate::reactive::tree::diff_trees;
use cosmic_text::FontSystem;
use crate::reactive::element_state_store::ElementStateStore;

#[test]
fn diff_trees_same_tag_same_id_are_equal() {
    reset_unique_element_id();
    
    let mut font_system = FontSystem::new();

    let initial_view = Container::new().component().push(Text::new("Foo").component());
    let updated_view = Container::new().component().push(Text::new("Foo").component());

    let root_element: ElementBox = Container::new().into();

    let mut user_state = StateStore::default();
    let mut element_state = ElementStateStore::default();

    let initial_tree = diff_trees(
        initial_view,
        root_element.clone(),
        None,
        &mut user_state,
        &mut element_state,
        &mut font_system,
        false
    );

    let updated_tree = diff_trees(
        updated_view,
        root_element.clone(),
        Some(&initial_tree.component_tree),
        &mut user_state,
        &mut element_state,
        &mut font_system,
        false
    );

    let initial_id = &initial_tree.component_tree.children[0].children[0].id;
    let updated_id = &updated_tree.component_tree.children[0].children[0].id;

    assert_eq!(initial_id, updated_id, "Elements with identical content tags and positions have the same id.");
}

#[test]
fn diff_trees_after_one_iteration_adjacent_nodes_different_ids() {
    let mut font_system = FontSystem::new();
    reset_unique_element_id();

    let root_node_1 = Container::new().component().push(Text::new("Foo").component());
    let root_node_2 = Container::new()
        .component()
        .push(Text::new("Foo").component())
        .push(Text::new("Bar").component());

    let root_element: ElementBox = Container::new().into();
    let mut user_state = StateStore::default();
    let mut element_state = ElementStateStore::default();

    let tree_1 = diff_trees(
        root_node_1,
        root_element.clone(),
        None,
        &mut user_state,
        &mut element_state,
        &mut font_system,
        false
    );

    let tree_2 = diff_trees(
        root_node_2,
        root_element.clone(),
        Some(&tree_1.component_tree),
        &mut user_state,
        &mut element_state,
        &mut font_system,
        false
    );

    let initial_id = &tree_1.component_tree.children[0].children[0].id;
    let updated_id = &tree_2.component_tree.children[0].children[1].id;

    assert_ne!(initial_id, updated_id, "Elements in different positions should have different ids.");
}

#[test]
fn diff_trees_after_one_iteration_same_key_different_position_same_id() {
    let mut font_system = FontSystem::new();
    reset_unique_element_id();

    let root_node_1 = Container::new().component().push(Text::new("Foo").component().key("key_1"));
    let root_node_2 = Container::new()
        .component()
        .push(Text::new("Bar").component())
        .push(Text::new("Foo").component().key("key_1"));

    let root_element: ElementBox = Container::new().into();
    let mut user_state = StateStore::default();
    let mut element_state = ElementStateStore::default();

    let tree_1 = diff_trees(
        root_node_1,
        root_element.clone(),
        None,
        &mut user_state,
        &mut element_state,
        &mut font_system,
        false
    );

    let tree_2 = diff_trees(
        root_node_2,
        root_element.clone(),
        Some(&tree_1.component_tree),
        &mut user_state,
        &mut element_state,
        &mut font_system,
        false
    );

    let initial_id = &tree_1.component_tree.children[0].children[0].id;
    let updated_id = &tree_2.component_tree.children[0].children[1].id;

    assert_eq!(initial_id, updated_id, "Elements in different positions with the same key, should have the same id.");
}
