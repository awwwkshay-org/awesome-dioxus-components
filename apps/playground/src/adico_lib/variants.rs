//! Shared adico-owned style vocabulary for registry components.

/// Shared corner-radius scale for every component with a visible bounded
/// surface. `Default` tracks this project's `--radius` theme token, which
/// `tailwind.css` binds to the `rounded-lg` utility
/// (`--radius-lg: var(--radius)`) — so `Default` maps to `rounded-lg`, and
/// every other variant is named by its own visual size rather than by
/// Tailwind's suffix at that position. `Md` exists specifically so a
/// component whose current corner size is Tailwind's fixed (non-token)
/// `rounded-md` can declare `#[props(default = Radius::Md)]` and keep its
/// exact current visual, distinct from `Default`, which reactively tracks
/// the theme's `--radius` custom property.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Radius {
    None,
    Sm,
    Md,
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
            Radius::Md => "rounded-md",
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
        assert_eq!(Radius::Md.class(), "rounded-md");
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
