use crate::elements::element::Element;
use crate::widget_id::create_unique_widget_id;

/// Assigns new ids to the nodes in the tree using level order traversal.
pub fn assign_tree_new_ids(new_tree: &mut Element) {
    let mut queue: Vec<&mut Element> = Vec::new();
    queue.push(new_tree);

    while queue.len() > 0 {
        let current = queue.pop().unwrap();
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
    let mut bfs_queue: Vec<&mut Element> = Vec::new();
    bfs_queue.push(new_tree);

    while bfs_queue.len() > 0 {
        let current = bfs_queue.pop().unwrap();
        for child in current.children_mut() {
            bfs_queue.push(child);
        }
    }

    return new_tree.clone();
}
