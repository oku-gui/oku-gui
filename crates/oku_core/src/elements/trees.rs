use crate::elements::element::Element;
use crate::widget_id::create_unique_widget_id;

/// Assigns new ids to the nodes in the tree using level order traversal.
pub fn assign_tree_new_ids(new_tree: &mut Element) {
    let mut queue: Vec<&mut Element> = Vec::new();
    queue.push(new_tree);

    while let Some(current) = queue.pop() {
        let id = current.id_mut();
        *id = create_unique_widget_id();

        for child in current.children_mut() {
            queue.push(child);
        }
    }
}

/// Diff two trees and return the new tree with stable ids.
pub fn diff_tree(old_tree: Option<&mut Element>, new_tree: Option<&mut Element>) -> Element {
    let new_tree = new_tree.unwrap();

    // The new tree is the only tree, so we assign new ids to it and return it.
    if old_tree.is_none() {
        assign_tree_new_ids(new_tree);
        return new_tree.clone();
    }

    let old_tree = old_tree.unwrap();

    let mut old_queue: Vec<&mut Element> = Vec::new();
    let mut new_queue: Vec<&mut Element> = Vec::new();

    old_queue.push(old_tree);
    new_queue.push(new_tree);

    while !old_queue.is_empty() && !new_queue.is_empty() {
        let old_current = old_queue.pop().unwrap();
        let new_current = new_queue.pop().unwrap();

        if old_current.name() != new_current.name() && new_current.name() != "Empty" {
            assign_tree_new_ids(new_current);
            continue;
        } else if old_current.name() != new_current.name() && new_current.name() == "Empty" {
            *new_current.id_mut() = old_current.id();
            continue;
        } else {
            *new_current.id_mut() = old_current.id();

            let old_children = old_current.children_mut();
            let new_children = new_current.children_mut();

            for child in old_children.iter_mut() {
                old_queue.push(child);
            }

            for child in new_children.iter_mut() {
                new_queue.push(child);
            }
        }
    }

    new_tree.clone()
}
