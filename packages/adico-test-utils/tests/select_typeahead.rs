use adico_test_utils::assert_select_typeahead;

#[test]
fn keyboard_typeahead_focuses_the_best_available_option() {
    assert_select_typeahead("ber", &["Apple", "Berry", "Cherry"], Some(1));
    assert_select_typeahead("", &["Apple", "Berry", "Cherry"], None);
}
