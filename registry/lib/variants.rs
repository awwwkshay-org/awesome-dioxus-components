//! Shared adico-owned style vocabulary for registry components.

/// Shared corner-radius scale for every component with a visible bounded
/// surface. `Default` tracks this project's `--radius` theme token, which
/// `tailwind.css` binds to the `rounded-lg` utility
/// (`--radius-lg: var(--radius)`) — so `Default` maps to `rounded-lg`, not
/// `rounded-md`, and every other variant is named by its own visual size
/// rather than by Tailwind's suffix at that position.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Radius {
    None,
    Sm,
    #[default]
    Default,
    Lg,
    Xl,
    Full,
}

impl Radius {
    pub fn class(self) -> &'static str {
        match self {
            Radius::None => "rounded-none",
            Radius::Sm => "rounded-sm",
            Radius::Default => "rounded-lg",
            Radius::Lg => "rounded-xl",
            Radius::Xl => "rounded-2xl",
            Radius::Full => "rounded-full",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_maps_each_variant_to_its_tailwind_utility() {
        assert_eq!(Radius::None.class(), "rounded-none");
        assert_eq!(Radius::Sm.class(), "rounded-sm");
        assert_eq!(Radius::Default.class(), "rounded-lg");
        assert_eq!(Radius::Lg.class(), "rounded-xl");
        assert_eq!(Radius::Xl.class(), "rounded-2xl");
        assert_eq!(Radius::Full.class(), "rounded-full");
    }

    #[test]
    fn default_variant_is_default() {
        assert_eq!(Radius::default(), Radius::Default);
    }
}
