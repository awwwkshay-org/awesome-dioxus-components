//! CLI-specific project discovery and safe consumer-project mutation support.

#![forbid(unsafe_code)]

pub mod add;
pub mod cargo;
pub mod css;
pub mod css_build;
pub mod init;
pub mod modules;
pub mod project;

/// The official `@adico` registry's HTTPS base -- kept as a single named
/// constant so it can be repointed (a domain change, a CDN) without a CLI
/// release. See design D7 of `adopt-shadcn-style-registry-serving`.
pub const OFFICIAL_REGISTRY_URL: &str = "https://adico.awwwkshay.com/r/index.json";
