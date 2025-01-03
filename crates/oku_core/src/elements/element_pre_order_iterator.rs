use crate::elements::element::Element;
pub(crate) struct ElementTreePreOrderIterator<'a> {
    stack: Vec<&'a dyn Element>,
}

impl<'a> ElementTreePreOrderIterator<'a> {
    fn new(root: &'a dyn Element) -> Self {
        Self { stack: vec![root] }
    }
}

impl<'a> Iterator for ElementTreePreOrderIterator<'a> {
    type Item = &'a dyn Element;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.pop() {
            for child in node.children().iter().rev() {
                self.stack.push(*child);
            }
            Some(node)
        } else {
            None
        }
    }
}

impl dyn Element {
    pub fn pre_order_iter(&self) -> ElementTreePreOrderIterator {
        ElementTreePreOrderIterator::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::elements::element::ElementBox;
    use crate::elements::{Container, Text};
    use crate::reactive::element_id::reset_unique_element_id;
    use crate::reactive::state_store::StateStore;
    use crate::reactive::tree::diff_trees;
    use cosmic_text::FontSystem;

    #[test]
    fn pre_order_iter_ids_correct_order() {
        let mut font_system = FontSystem::new();
        reset_unique_element_id();

        let initial_view = Container::new().id("1").component().push(Text::new("Foo").id("2").component()).push(
            Container::new()
                .id("3")
                .component()
                .push(Text::new("Bar").id("4").component())
                .push(Text::new("Baz").id("5").component()),
        );
        let root_element: ElementBox = Container::new().id("0").into();

        let mut user_state = StateStore::default();
        let mut element_state = StateStore::default();

        let initial_tree =
            diff_trees(initial_view, root_element.clone(), None, &mut user_state, &mut element_state, &mut font_system, false);

        initial_tree.0.print_tree();
        initial_tree.1.internal.print_tree();

        let mut iter = initial_tree.1.internal.pre_order_iter();
        assert_eq!(iter.next().unwrap().get_id().clone(), Some("0".to_string()));
        assert_eq!(iter.next().unwrap().get_id().clone(), Some("1".to_string()));
        assert_eq!(iter.next().unwrap().get_id().clone(), Some("2".to_string()));
        assert_eq!(iter.next().unwrap().get_id().clone(), Some("3".to_string()));
        assert_eq!(iter.next().unwrap().get_id().clone(), Some("4".to_string()));
        assert_eq!(iter.next().unwrap().get_id().clone(), Some("5".to_string()));
        assert!(iter.next().is_none());
    }
}
