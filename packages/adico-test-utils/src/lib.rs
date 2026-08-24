//! Common fixtures and assertions used by adico package and integration tests.
//!
//! M1 and M2 add primitive, registry, and CLI helpers here without making them
//! a production dependency of installed components.

#![forbid(unsafe_code)]

/// Executes the shared Select typeahead assertion used by primitive interaction tests.
pub fn assert_select_typeahead(query: &str, options: &[&str], expected_index: Option<usize>) {
    assert_eq!(
        adico_primitives::select::test_support::typeahead_best_match(query, options),
        expected_index,
        "unexpected typeahead focus target for query {query:?}"
    );
}
