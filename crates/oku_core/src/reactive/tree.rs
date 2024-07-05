use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::components::component::{ComponentOrElement, ComponentSpecification, UpdateFn};
use crate::elements::element::Element;
use crate::widget_id::create_unique_widget_id;

#[derive(Clone)]
pub struct ComponentTreeNode {
    key: Option<String>,
    tag: String,
    update: Option<UpdateFn>,
    children: Vec<ComponentTreeNode>,
    children_keys: HashMap<String, u64>,
    id: u64,
}

struct UnsafeElement {
    element: *mut dyn Element,
}

#[derive(Clone)]
struct TreeVisitorNode {
    component_specification: Rc<RefCell<ComponentSpecification>>,
    parent_element_ptr: *mut dyn Element,
    parent_component_node: *mut ComponentTreeNode,
    old_component_node: Option<*const ComponentTreeNode>,
}

impl ComponentTreeNode {
    pub fn print_tree(&self) {
        unsafe {
            let mut elements: Vec<(*const ComponentTreeNode, usize, bool)> = vec![(self, 0, true)];
            while let Some((element, indent, is_last)) = elements.pop() {
                let mut prefix = String::new();
                for _ in 0..indent {
                    prefix.push_str("  ");
                }
                if is_last {
                    prefix.push_str("└─");
                } else {
                    prefix.push_str("├─");
                }
                println!("{} , Tag: {}, Id: {}, Key: {:?}", prefix, (*element).tag, (*element).id, (*element).key);
                let children = &(*element).children;
                for (i, child) in children.iter().enumerate().rev() {
                    let is_last = i == children.len() - 1;
                    elements.push((child, indent + 1, is_last));
                }
            }
        }
    }
}

/// Creates a new Component tree and Element tree from a ComponentSpecification.
/// The ids of the Component tree are stable across renders.
pub(crate) fn create_trees_from_render_specification(component_specification: ComponentSpecification, mut root_element: Box<dyn Element>, old_component_tree: Option<&ComponentTreeNode>) -> (ComponentTreeNode, Box<dyn Element>) {
    println!("-----------------------------------------");
    unsafe {
        let mut component_tree = ComponentTreeNode {
            key: None,
            tag: "root".to_string(),
            update: None,
            children: vec![],
            children_keys: HashMap::new(),
            id: 0,
        };

        let mut old_component_tree_as_ptr = old_component_tree.map(|old_root| old_root as *const ComponentTreeNode);

        // HACK: This is a workaround to get the first child of the old component tree because we start at the first level on the new tree.
        // This is because the root of the component tree is not a component, but a dummy node.
        if old_component_tree_as_ptr.is_some() {
            old_component_tree_as_ptr = Some((*old_component_tree_as_ptr.unwrap()).children.get(0).unwrap() as *const ComponentTreeNode);
        }

        let component_root: *mut ComponentTreeNode = &mut component_tree as *mut ComponentTreeNode;
        let root_spec = ComponentSpecification {
            component: ComponentOrElement::Element(root_element.clone()),
            key: None,
            props: None,
            children: vec![component_specification],
        };

        let mut to_visit: Vec<TreeVisitorNode> = vec![TreeVisitorNode {
            component_specification: Rc::new(RefCell::new(root_spec)),
            parent_element_ptr: root_element.as_mut() as *mut dyn Element,
            parent_component_node: component_root,
            old_component_node: old_component_tree_as_ptr,
        }];

        while let Some(tree_node) = to_visit.pop() {
            let key = tree_node.component_specification.borrow().key.clone();
            let children = tree_node.component_specification.borrow().children.clone();
            let props = tree_node.component_specification.borrow().props.clone();

            let old_tag = tree_node.old_component_node.map(|old_node| (*old_node).tag.clone());
            let mut parent_element_ptr = tree_node.parent_element_ptr;
            let parent_component_ptr = tree_node.parent_component_node;

            match &mut tree_node.component_specification.borrow_mut().component {
                ComponentOrElement::Element(element) => {
                    // Create the new element node.
                    let mut element = element.clone();
                    element.set_parent_component_id((*tree_node.parent_component_node).id);

                    // Store the new tag, i.e. the element's name.
                    let new_tag = element.name().to_string();

                    let id = if let Some(old_tag) = old_tag {
                        println!("Old Tag: {}, New Tag: {}", old_tag, new_tag);
                        if new_tag == old_tag {
                            (*tree_node.old_component_node.unwrap()).id
                        } else {
                            create_unique_widget_id()
                        }
                    } else {
                        create_unique_widget_id()
                    };

                    // Move the new element into it's parent and set the parent element to be the new element.
                    tree_node.parent_element_ptr.as_mut().unwrap().children_mut().push(element);
                    parent_element_ptr = tree_node.parent_element_ptr.as_mut().unwrap().children_mut().last_mut().unwrap().as_mut();

                    let new_component_node = ComponentTreeNode {
                        key: None,
                        tag: new_tag,
                        update: None,
                        children: vec![],
                        children_keys: HashMap::new(),
                        id,
                    };

                    // Add the new component node to the tree and get a pointer to it.
                    parent_component_ptr.as_mut().unwrap().children.push(new_component_node);
                    let new_component_pointer: *mut ComponentTreeNode = (*tree_node.parent_component_node).children.last_mut().unwrap();

                    // Get the old children of the old component node.
                    let mut olds: Vec<*const ComponentTreeNode> = vec![];
                    if tree_node.old_component_node.is_some() {
                        for child in (*tree_node.old_component_node.unwrap()).children.iter() {
                            olds.push(child as *const ComponentTreeNode);
                        }
                    }

                    let mut new_to_visits: Vec<TreeVisitorNode> = vec![];
                    // Add the children of the new element to the to visit list.
                    for (index, child) in children.into_iter().enumerate() {
                        
                        // Find old child by key and if no key is found, find by index.
                        let key = child.key.clone();
                        
                        let mut index = index;
                        
                        for (old_index, old_child) in olds.iter().enumerate() {
                            let old_key = (*(*old_child)).key.clone();
                            
                            if old_key == key {
                                if old_key.is_none() || child.key.is_none() {
                                    continue;
                                }
                                index = old_index;
                                break;
                            }
                            
                        }
                        
                        new_to_visits.push(TreeVisitorNode {
                            component_specification: Rc::new(RefCell::new(child)),
                            parent_element_ptr,
                            parent_component_node: new_component_pointer,
                            old_component_node: olds.get(index).copied(),
                        });
                    }

                    to_visit.extend(new_to_visits.into_iter().rev());
                }
                ComponentOrElement::ComponentSpec(component_spec, new_tag, type_id) => {

                    let children_keys = (*parent_component_ptr).children_keys.clone();

                    let id: u64 = if key.is_some() && children_keys.contains_key(&key.clone().unwrap()) {
                        *(children_keys.get(&key.clone().unwrap()).unwrap())
                    } else if let Some(old_tag) = old_tag {
                        println!("Old Tag: {}, New Tag: {}", old_tag, new_tag);
                        if *new_tag == old_tag {
                            // If the old tag is the same as the new tag, we can reuse the old id.
                            (*tree_node.old_component_node.unwrap()).id
                        } else {
                            create_unique_widget_id()
                        }
                    } else {
                        create_unique_widget_id()
                    };

                    let new_component = component_spec(props, children, id);

                    let new_component_node = ComponentTreeNode {
                        key: key.clone(),
                        tag: (*new_tag).clone(),
                        update: None,
                        children: vec![],
                        children_keys: HashMap::new(),
                        id,
                    };

                    // Add the current child id to the children_keys hashmap in the parent.
                    if let Some(key) = key.clone() {
                        parent_component_ptr.as_mut().unwrap().children_keys.insert(key, id);
                    }

                    // Add the new component node to the tree and get a pointer to it.
                    parent_component_ptr.as_mut().unwrap().children.push(new_component_node);
                    let new_component_pointer: *mut ComponentTreeNode = (*tree_node.parent_component_node).children.last_mut().unwrap();

                    // The old node should be the first child of the old component node.
                    let old_component_tree = tree_node.old_component_node.map(|old_node| (*old_node).children.get(0).unwrap() as *const ComponentTreeNode);

                    // Add the computed component spec to the to visit list.
                    to_visit.push(TreeVisitorNode {
                        component_specification: Rc::new(RefCell::new(new_component.0)),
                        parent_element_ptr,
                        parent_component_node: new_component_pointer,
                        old_component_node: old_component_tree,
                    });
                }
            };
        }
        println!("-----------------------------------------");
        println!("-----------------------------------------");
        println!("old");
        if let Some(old_component_tree) = old_component_tree {
            old_component_tree.print_tree()
        }
        println!("new");
        component_tree.print_tree();
        println!("-----------------------------------------");
        println!("-----------------------------------------");

        (component_tree, root_element)
    }
}
