//! Small case-conversion + part-id-matching helpers shared by every axis
//! fetcher (Dioxus and Base UI both key parts by
//! `<ComponentPascalPrefix><PartPascalName>` -- `DialogRoot`, `DialogRoot`
//! -- so the same "strip the component's own prefix, kebab-case the rest"
//! logic applies to both).

pub fn pascal_case(snake_or_kebab: &str) -> String {
    snake_or_kebab
        .split(['_', '-'])
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

pub fn kebab_case(pascal_remainder: &str) -> String {
    let mut result = String::new();
    for (index, ch) in pascal_remainder.chars().enumerate() {
        if ch.is_uppercase() && index > 0 {
            result.push('-');
        }
        result.extend(ch.to_lowercase());
    }
    result
}

/// The part id a name maps to under a given prefix source (e.g. prefix
/// source `dialog` + name `DialogRoot` -> `root`; no-match -> the name's
/// own full kebab-cased form).
pub fn part_id_for(prefix_source: &str, name: &str) -> String {
    let prefix = pascal_case(prefix_source);
    let remainder = name.strip_prefix(&prefix).unwrap_or(name);
    if remainder.is_empty() {
        "root".to_string()
    } else {
        kebab_case(remainder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_prefix_and_kebabs_remainder() {
        assert_eq!(part_id_for("dialog", "DialogRoot"), "root");
        assert_eq!(part_id_for("dialog", "DialogContent"), "content");
        assert_eq!(part_id_for("dialog", "Dialog"), "root");
        assert_eq!(part_id_for("alert_dialog", "AlertDialogRoot"), "root");
    }

    #[test]
    fn falls_back_to_full_name_when_prefix_does_not_match() {
        assert_eq!(part_id_for("dialog", "Portal"), "portal");
    }
}
