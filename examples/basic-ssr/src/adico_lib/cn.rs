//! Small source-owned class composition helper for registry components.

/// Joins non-empty class fragments with one space, preserving caller order.
pub fn cn(classes: &[&str]) -> String {
    classes
        .iter()
        .filter_map(|class| (!class.trim().is_empty()).then_some(class.trim()))
        .collect::<Vec<_>>()
        .join(" ")
}
