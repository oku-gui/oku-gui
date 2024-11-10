use crate::elements::element::ElementState;
use crate::engine::events::Message::OkuMessage;
use crate::engine::events::{Message, OkuEvent};
use crate::components::component::{
    ComponentId, ComponentOrElement, ComponentSpecification, GenericUserState, UpdateFn, UpdateResult,
};
use crate::elements::element::Element;
use crate::reactive::element_id::create_unique_element_id;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::elements::Container;
use crate::elements::container::ContainerState;

#[derive(Clone)]
pub struct ComponentTreeNode {
    pub is_element: bool,
    pub key: Option<String>,
    pub tag: String,
    pub update: UpdateFn,
    pub children: Vec<ComponentTreeNode>,
    pub children_keys: HashMap<String, ComponentId>,
    pub id: ComponentId,
    pub(crate) parent_id: Option<ComponentId>,
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
        let mut elements: Vec<(&ComponentTreeNode, usize, bool)> = vec![(self, 0, true)];
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
            println!("{} , Tag: {}, Id: {}, Key: {:?}, Parent: {:?}", prefix, element.tag, element.id, element.key, element.parent_id);
            let children = &element.children;
            for (i, child) in children.iter().enumerate().rev() {
                let is_last = i == children.len() - 1;
                elements.push((child, indent + 1, is_last));
            }
        }
    }
}
fn dummy_update(
    _state: &mut GenericUserState,
    _id: ComponentId,
    _message: Message,
    _source_element_id: Option<String>,
) -> UpdateResult {
    UpdateResult {
        propagate: true,
        result: None,
    }
}

/// Creates a new Component tree and Element tree from a ComponentSpecification.
/// The ids of the Component tree are stable across renders.
pub(crate) fn create_trees_from_render_specification(
    component_specification: ComponentSpecification,
    mut root_element: Box<dyn Element>,
    old_component_tree: Option<&ComponentTreeNode>,
    user_state: &mut HashMap<ComponentId, Box<GenericUserState>>,
    element_state: &mut HashMap<ComponentId, Box<ElementState>>,
) -> (ComponentTreeNode, Box<dyn Element>) {
    //println!("-----------------------------------------");
    unsafe {
        let mut component_tree = ComponentTreeNode {
            is_element: false,
            key: None,
            tag: "root".to_string(),
            update: dummy_update,
            children: vec![],
            children_keys: HashMap::new(),
            id: 0,
            parent_id: None,
        };
        
        // Make sure to set a default state for the root.
        element_state.insert(0, Box::new(ContainerState {
            scroll_delta_y: 0.0,
        }));

        let mut old_component_tree_as_ptr = old_component_tree.map(|old_root| old_root as *const ComponentTreeNode);

        // HACK: This is a workaround to get the first child of the old component tree because we start at the first level on the new tree.
        // This is because the root of the component tree is not a component, but a dummy node.
        if old_component_tree_as_ptr.is_some() {
            old_component_tree_as_ptr =
                Some((*old_component_tree_as_ptr.unwrap()).children.get(0).unwrap() as *const ComponentTreeNode);
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

                    // Store the new tag, i.e. the element's name.
                    let new_tag = element.name().to_string();

                    let default: Box<GenericUserState> = Box::new(());
                    let id = if let Some(old_tag) = old_tag {
                        if new_tag == old_tag {
                            (*tree_node.old_component_node.unwrap()).id
                        } else {
                            create_unique_element_id(user_state, default)
                        }
                    } else {
                        create_unique_element_id(user_state, default)
                    };

                    element.set_component_id(id);
                    
                    if let Some(container) = element.as_any().downcast_ref::<Container>() {
                        if !element_state.contains_key(&id) {
                            element_state.insert(id, Box::new(ContainerState {
                                scroll_delta_y: 0.0,
                            }));
                        }
                    } else {
                        if !element_state.contains_key(&id) {
                            element_state.insert(id, Box::new(()));
                        }
                    }
                    
                    // Move the new element into it's parent and set the parent element to be the new element.
                    tree_node.parent_element_ptr.as_mut().unwrap().children_mut().push(element);
                    parent_element_ptr =
                        tree_node.parent_element_ptr.as_mut().unwrap().children_mut().last_mut().unwrap().as_mut();

                    let new_component_node = ComponentTreeNode {
                        is_element: true,
                        key,
                        tag: new_tag,
                        update: dummy_update,
                        children: vec![],
                        children_keys: HashMap::new(),
                        id,
                        parent_id: Some((*parent_component_ptr).id),
                    };

                    // Add the new component node to the tree and get a pointer to it.
                    parent_component_ptr.as_mut().unwrap().children.push(new_component_node);
                    let new_component_pointer: *mut ComponentTreeNode =
                        (*tree_node.parent_component_node).children.last_mut().unwrap();

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
                ComponentOrElement::ComponentSpec(component_data) => {
                    let children_keys = (*parent_component_ptr).children_keys.clone();

                    let mut initialized = false;

                    let id: ComponentId = if key.is_some() && children_keys.contains_key(&key.clone().unwrap()) {
                        *(children_keys.get(&key.clone().unwrap()).unwrap())
                    } else if let Some(old_tag) = old_tag {
                        if *component_data.tag == old_tag {
                            // If the old tag is the same as the new tag, we can reuse the old id.
                            (*tree_node.old_component_node.unwrap()).id
                        } else {
                            let id = create_unique_element_id(user_state, (component_data.default_state)());

                            initialized = true;
                            id
                        }
                    } else {
                        let id = create_unique_element_id(user_state, (component_data.default_state)());
                        initialized = true;
                        id
                    };

                    {
                        let state_mut = user_state.get_mut(&id).unwrap().as_mut();

                        if initialized {
                            (component_data.update_fn)(state_mut, id, OkuMessage(OkuEvent::Initialized), None);
                        }
                    }

                    let state = user_state.get(&id);
                    let state = state.unwrap().as_ref();
                    let new_component = (component_data.view_fn)(state, props, children, id);

                    let new_component_node = ComponentTreeNode {
                        is_element: false,
                        key: key.clone(),
                        tag: component_data.tag.clone(),
                        update: component_data.update_fn,
                        children: vec![],
                        children_keys: HashMap::new(),
                        id,
                        parent_id: Some((*parent_component_ptr).id),
                    };

                    // Add the current child id to the children_keys hashmap in the parent.
                    if let Some(key) = key.clone() {
                        parent_component_ptr.as_mut().unwrap().children_keys.insert(key, id);
                    }

                    // Add the new component node to the tree and get a pointer to it.
                    parent_component_ptr.as_mut().unwrap().children.push(new_component_node);
                    let new_component_pointer: *mut ComponentTreeNode =
                        (*tree_node.parent_component_node).children.last_mut().unwrap();

                    // The old node should be the first child of the old component node.
                    let old_component_tree = tree_node
                        .old_component_node
                        .map(|old_node| (*old_node).children.get(0).unwrap() as *const ComponentTreeNode);

                    // Add the computed component spec to the to visit list.
                    to_visit.push(TreeVisitorNode {
                        component_specification: Rc::new(RefCell::new(new_component)),
                        parent_element_ptr,
                        parent_component_node: new_component_pointer,
                        old_component_node: old_component_tree,
                    });
                }
            };
        }
        /*println!("-----------------------------------------");
        println!("-----------------------------------------");
        println!("old");
        if let Some(old_component_tree) = old_component_tree {
            old_component_tree.print_tree()
        }
        println!("new");
        component_tree.print_tree();
        println!("-----------------------------------------");
        println!("-----------------------------------------");*/

        (component_tree, root_element)
    }
}