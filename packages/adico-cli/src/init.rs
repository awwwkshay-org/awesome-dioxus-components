//! Plan-first initialization of a consumer Dioxus project.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use adico_registry_core::{
    ComponentPaths, ComponentsConfiguration, CssConfiguration, RegistryNamespace, RegistrySource,
    ThemeConfiguration,
};
use thiserror::Error;

use crate::modules::{
    ModuleExportRequest, ModuleUpdatePlan, plan_entrypoint_module_update, plan_module_update,
};
use crate::project::{DioxusProject, ProjectDiscoveryError, discover_dioxus_project};

/// User-selected initialization inputs. The official registry is always
/// available by default; an organization can add a local or HTTPS source and
/// make it the default without replacing `@adico`.
#[derive(Clone, Debug)]
pub struct InitOptions {
    /// Selected default registry for bare component requests.
    pub default_registry: RegistryNamespace,
    /// Explicitly configured registry sources.
    pub registries: BTreeMap<RegistryNamespace, RegistrySource>,
}

impl Default for InitOptions {
    fn default() -> Self {
        let official: RegistryNamespace = RegistryNamespace::OFFICIAL
            .parse()
            .expect("official namespace is valid");
        Self {
            default_registry: official.clone(),
            registries: BTreeMap::from([(official, RegistrySource::Embedded)]),
        }
    }
}

impl InitOptions {
    /// Adds or replaces one configured registry source.
    pub fn with_registry(mut self, namespace: RegistryNamespace, source: RegistrySource) -> Self {
        self.registries.insert(namespace, source);
        self
    }
}

/// A reviewable initialization plan. Applying it creates only missing consumer
/// directories and a new valid `components.json`; it never overwrites files.
#[derive(Clone, Debug)]
pub struct InitPlan {
    /// Safely discovered consumer package.
    pub project: DioxusProject,
    /// Proposed consumer-owned configuration.
    pub configuration: ComponentsConfiguration,
    /// Project-root configuration destination.
    pub configuration_path: PathBuf,
    /// Missing source/CSS parent directories that apply will create.
    pub directories_to_create: Vec<PathBuf>,
    /// Rust module roots reserved for later marker-region setup.
    pub module_roots: Vec<PathBuf>,
    /// Explicit entrypoint-owned declarations that expose generated roots.
    pub entrypoint_modules: ModuleUpdatePlan,
    /// Generated nested module roots required by the default layout.
    pub module_setup: Vec<ModuleUpdatePlan>,
    /// CSS entry reserved for the later marker-owned theme installer.
    pub css_entry: PathBuf,
    write_configuration: bool,
}

impl InitPlan {
    /// Returns whether applying this plan will make any filesystem changes.
    pub fn has_changes(&self) -> bool {
        self.write_configuration
            || !self.directories_to_create.is_empty()
            || self.entrypoint_modules.has_changes()
            || self.module_setup.iter().any(ModuleUpdatePlan::has_changes)
    }

    /// Applies an already reviewed plan without overwriting consumer files.
    pub fn apply(&self) -> Result<(), InitError> {
        for directory in &self.directories_to_create {
            fs::create_dir_all(directory).map_err(|error| InitError::WriteFailed {
                path: directory.display().to_string(),
                message: error.to_string(),
            })?;
        }
        self.entrypoint_modules
            .apply()
            .map_err(|error| InitError::Module(error.to_string()))?;
        for module in &self.module_setup {
            module
                .apply()
                .map_err(|error| InitError::Module(error.to_string()))?;
        }
        if self.write_configuration {
            let contents = serde_json::to_string_pretty(&self.configuration).map_err(|error| {
                InitError::ConfigurationSerialization {
                    message: error.to_string(),
                }
            })?;
            fs::write(&self.configuration_path, format!("{contents}\n")).map_err(|error| {
                InitError::WriteFailed {
                    path: self.configuration_path.display().to_string(),
                    message: error.to_string(),
                }
            })?;
        }
        Ok(())
    }
}

/// Creates a non-mutating plan for `adico init` from a directory in a consumer
/// project.
pub fn plan_init(start: &Path, options: &InitOptions) -> Result<InitPlan, InitError> {
    let project = discover_dioxus_project(start)?;
    let project_root = project
        .package_manifest_path
        .parent()
        .expect("Cargo manifests have parent directories")
        .to_path_buf();
    let configuration = default_configuration(options.clone());
    configuration
        .validate()
        .map_err(|error| InitError::InvalidConfiguration(Box::new(error)))?;

    let configuration_path = project_root.join("components.json");
    let write_configuration =
        match configuration_path
            .try_exists()
            .map_err(|error| InitError::WriteFailed {
                path: configuration_path.display().to_string(),
                message: error.to_string(),
            })? {
            false => true,
            true if configuration_path.is_file() => {
                let existing = fs::read_to_string(&configuration_path).map_err(|error| {
                    InitError::WriteFailed {
                        path: configuration_path.display().to_string(),
                        message: error.to_string(),
                    }
                })?;
                let existing = ComponentsConfiguration::parse(&existing)
                    .map_err(|error| InitError::InvalidExistingConfiguration(Box::new(error)))?;
                if existing == configuration {
                    false
                } else {
                    return Err(InitError::ExistingConfigurationConflict {
                        path: configuration_path.display().to_string(),
                    });
                }
            }
            true => {
                return Err(InitError::ExistingConfigurationConflict {
                    path: configuration_path.display().to_string(),
                });
            }
        };

    let paths = configuration.paths.clone();
    let entrypoint_modules = plan_entrypoint_module_update(
        &project.entrypoint,
        &[
            ModuleExportRequest {
                module: "components".to_string(),
                reexport: false,
            },
            ModuleExportRequest {
                module: "adico_lib".to_string(),
                reexport: false,
            },
        ],
    )
    .map_err(|error| InitError::Module(error.to_string()))?;
    let module_setup = vec![
        plan_module_update(
            project_root.join(&paths.components).join("mod.rs"),
            &[ModuleExportRequest {
                module: "ui".to_string(),
                reexport: true,
            }],
        )
        .map_err(|error| InitError::Module(error.to_string()))?,
        plan_module_update(project_root.join(&paths.ui).join("mod.rs"), &[])
            .map_err(|error| InitError::Module(error.to_string()))?,
        plan_module_update(project_root.join(&paths.lib).join("mod.rs"), &[])
            .map_err(|error| InitError::Module(error.to_string()))?,
    ];
    let css_entry = project_root.join(&configuration.css.entry);
    let requested_directories = [
        project_root.join(&paths.components),
        project_root.join(&paths.ui),
        project_root.join(&paths.lib),
        project_root.join(&paths.hooks),
        css_entry
            .parent()
            .expect("configured relative CSS entry has a parent")
            .to_path_buf(),
    ];
    let directories_to_create = requested_directories
        .into_iter()
        .filter(|directory| !directory.is_dir())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    Ok(InitPlan {
        project,
        configuration,
        configuration_path,
        directories_to_create,
        module_roots: vec![
            project_root.join(&paths.components),
            project_root.join(&paths.ui),
        ],
        entrypoint_modules,
        module_setup,
        css_entry,
        write_configuration,
    })
}

fn default_configuration(options: InitOptions) -> ComponentsConfiguration {
    ComponentsConfiguration {
        schema: Some("https://adico.dev/schema/components.json/v1".to_string()),
        version: 1,
        style: "default".to_string(),
        theme: ThemeConfiguration {
            tokens: "shadcn".to_string(),
            dark_mode: "class".to_string(),
        },
        paths: ComponentPaths {
            components: "src/components".to_string(),
            ui: "src/components/ui".to_string(),
            lib: "src/adico_lib".to_string(),
            hooks: "src/hooks".to_string(),
        },
        css: CssConfiguration {
            entry: "assets/tailwind.css".to_string(),
            framework: "tailwind".to_string(),
        },
        registries: options.registries,
        default_registry: options.default_registry,
    }
}

/// Initialization failures that occur before or during narrowly scoped setup.
#[derive(Debug, Error)]
pub enum InitError {
    /// The invocation directory cannot be resolved to one Dioxus package.
    #[error(transparent)]
    ProjectDiscovery(#[from] ProjectDiscoveryError),
    /// Proposed configuration is not valid for this adico version.
    #[error("invalid init configuration: {0}")]
    InvalidConfiguration(#[source] Box<adico_registry_core::RegistryError>),
    /// Existing consumer configuration is invalid and cannot be safely changed.
    #[error("existing components.json is invalid: {0}")]
    InvalidExistingConfiguration(#[source] Box<adico_registry_core::RegistryError>),
    /// Existing configuration differs from this plan and is never overwritten.
    #[error(
        "{path} already exists with different settings; review it or choose matching init options"
    )]
    ExistingConfigurationConflict {
        /// Existing configuration path.
        path: String,
    },
    /// A planned directory or config file could not be written.
    #[error("cannot write {path}: {message}")]
    WriteFailed {
        /// Affected path.
        path: String,
        /// Filesystem failure.
        message: String,
    },
    /// Configuration output could not be serialized.
    #[error("cannot serialize components.json: {message}")]
    ConfigurationSerialization {
        /// Serialization failure.
        message: String,
    },
    /// The explicit entrypoint-owned region could not be prepared safely.
    #[error("cannot prepare adico entrypoint module region: {0}")]
    Module(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TemporaryConsumerProject {
        root: PathBuf,
    }

    impl Drop for TemporaryConsumerProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn temporary_consumer_project() -> TemporaryConsumerProject {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is after epoch")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("adico-init-test-{}-{nonce}", std::process::id()));
        fs::create_dir_all(root.join("src")).expect("source directory should be created");
        fs::create_dir_all(root.join("dioxus/src"))
            .expect("local Dioxus fixture should be created");
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"consumer\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[workspace]\n\n[dependencies]\ndioxus = { path = \"dioxus\" }\n",
        )
        .expect("consumer manifest should be written");
        fs::write(root.join("src/main.rs"), "fn main() {}\n")
            .expect("consumer entrypoint should be written");
        fs::write(
            root.join("dioxus/Cargo.toml"),
            "[package]\nname = \"dioxus\"\nversion = \"0.0.1\"\nedition = \"2024\"\n",
        )
        .expect("local Dioxus manifest should be written");
        fs::write(
            root.join("dioxus/src/lib.rs"),
            "//! local test dependency\n",
        )
        .expect("local Dioxus source should be written");
        fs::write(root.join("keep-me.txt"), "consumer-owned bytes\n")
            .expect("sentinel should be written");
        TemporaryConsumerProject { root }
    }

    #[test]
    fn official_init_is_reviewable_idempotent_and_preserves_unrelated_files() {
        let project = temporary_consumer_project();
        let plan = plan_init(&project.root, &InitOptions::default())
            .expect("official init plan should be valid");
        assert!(plan.has_changes());
        assert_eq!(plan.configuration.default_registry.as_str(), "@adico");
        assert!(!project.root.join("components.json").exists());
        plan.apply().expect("official init plan should apply");

        let configuration = ComponentsConfiguration::parse(
            &fs::read_to_string(project.root.join("components.json"))
                .expect("components configuration should exist"),
        )
        .expect("written configuration should validate");
        assert_eq!(configuration.default_registry.as_str(), "@adico");
        assert!(project.root.join("src/components/ui").is_dir());
        assert!(project.root.join("assets").is_dir());
        assert!(!project.root.join("assets/tailwind.css").exists());
        assert_eq!(
            fs::read_to_string(project.root.join("keep-me.txt")).expect("sentinel should remain"),
            "consumer-owned bytes\n"
        );
        assert!(
            !plan_init(&project.root, &InitOptions::default())
                .expect("matching init should be planable")
                .has_changes()
        );
    }

    #[test]
    fn company_default_init_is_explicit_and_does_not_overwrite_unrelated_files() {
        let project = temporary_consumer_project();
        let company: RegistryNamespace = "@awwwkshay".parse().expect("valid namespace");
        let options = InitOptions::default().with_registry(
            company.clone(),
            RegistrySource::Local {
                path: "company-registry".to_string(),
            },
        );
        let options = InitOptions {
            default_registry: company,
            ..options
        };
        let plan = plan_init(&project.root, &options).expect("company init plan should be valid");
        plan.apply().expect("company init plan should apply");

        let configuration = ComponentsConfiguration::parse(
            &fs::read_to_string(project.root.join("components.json"))
                .expect("components configuration should exist"),
        )
        .expect("written configuration should validate");
        assert_eq!(configuration.default_registry.as_str(), "@awwwkshay");
        assert!(matches!(
            configuration.registries.get(&configuration.default_registry),
            Some(RegistrySource::Local { path }) if path == "company-registry"
        ));
        assert!(matches!(
            configuration
                .registries
                .get(&"@adico".parse().expect("valid namespace")),
            Some(RegistrySource::Embedded)
        ));
        assert_eq!(
            fs::read_to_string(project.root.join("keep-me.txt")).expect("sentinel should remain"),
            "consumer-owned bytes\n"
        );
    }
}
