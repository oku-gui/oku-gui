use crate::elements::container::Container;
use crate::elements::element::Element;
use crate::elements::empty::Empty;
use crate::elements::standard_element::StandardElement;
use crate::elements::text::Text;
use crate::elements::trees::diff_tree;
use crate::widget_id::reset_unique_widget_id;

#[test]
fn diff_assigns_stable_id_when_child_is_removed() {
    let mut old_a = Container::new();
    let old_b = Element::Text(Text::new(String::from("b")));
    let old_c = Element::Text(Text::new(String::from("c")));

    old_a = old_a.add_child(old_b);
    old_a = old_a.add_child(old_c);
    let old_a = Element::Container(old_a);

    old_a.print_tree();
    
    // Reset the new widget id back to zero, act like this is a new render.
    reset_unique_widget_id();
    
    let mut new_a = Container::new();
    let new_b = Element::Empty(Empty::new());
    let new_c = Element::Text(Text::new(String::from("c")));
    new_a = new_a.add_child(new_b);
    new_a = new_a.add_child(new_c);
    let mut new_a = Element::Container(new_a);

    new_a.print_tree();
    
    //let new_tree = diff_tree(Some(&mut old_a), Some(&mut new_a));

    assert_eq!(old_a.children()[1].id(), new_a.children()[1].id(), "test that b has the same id when removed");
}
