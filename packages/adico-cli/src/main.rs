//! The `adico` command-line interface.

use std::env;
use std::path::PathBuf;

use adico_cli::add::{AddError, RegistryFileReader, plan_component_add};
use adico_cli::init::{InitOptions, plan_init};
use adico_cli::project::discover_dioxus_project;
use adico_registry_core::{
    ComponentsConfiguration, EmbeddedRegistry, LoadedRegistry, RegistryAddress, RegistryCatalog,
    RegistryLocation, RegistryNamespace, RegistrySource, RegistrySourceLoader,
    ResolvedRegistryItem,
};

fn main() {
    let arguments: Vec<_> = env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        Some("init") => run_init(&arguments[1..]),
        Some("add") => run_add(&arguments[1..]),
        _ => {
            eprintln!(
                "usage:\n  adico init [--default-registry <@namespace>] [--registry <@namespace>=<embedded|relative-path|https-url>] [--dry-run]\n  adico add <component...> [--dry-run]"
            );
            std::process::exit(2);
        }
    }
}

fn run_add(arguments: &[String]) {
    let (requests, dry_run) = match parse_add_options(arguments) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("adico add: {error}");
            std::process::exit(2);
        }
    };
    let current = match env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("adico add: cannot determine current directory: {error}");
            std::process::exit(1);
        }
    };
    let project = match discover_dioxus_project(&current) {
        Ok(project) => project,
        Err(error) => {
            eprintln!("adico add: {error}");
            std::process::exit(1);
        }
    };
    let root = project
        .package_manifest_path
        .parent()
        .expect("manifest has parent");
    let configuration = match std::fs::read_to_string(root.join("components.json"))
        .ok()
        .and_then(|contents| ComponentsConfiguration::parse(&contents).ok())
    {
        Some(configuration) => configuration,
        None => {
            eprintln!("adico add: valid components.json is required; run adico init first");
            std::process::exit(1);
        }
    };
    let (catalog, reader) = match configured_catalog(root, &configuration) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("adico add: {error}");
            std::process::exit(1);
        }
    };
    let plan = match plan_component_add(
        &catalog,
        root,
        &project.package_manifest_path,
        &configuration,
        &requests,
        &reader,
    ) {
        Ok(plan) => plan,
        Err(error) => {
            eprintln!("adico add: {error}");
            std::process::exit(1);
        }
    };
    println!(
        "adico add plan: {}",
        plan.install
            .items
            .iter()
            .map(|item| item.address.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    if dry_run {
        return;
    }
    if let Err(error) = plan.apply() {
        eprintln!("adico add: {error}");
        std::process::exit(1);
    }
    println!("adico add complete.");
}

fn parse_add_options(arguments: &[String]) -> Result<(Vec<RegistryAddress>, bool), String> {
    let mut dry_run = false;
    let mut requests = Vec::new();
    for argument in arguments {
        if argument == "--dry-run" {
            dry_run = true;
        } else if argument.starts_with('-') {
            return Err(format!("unknown add option {argument:?}"));
        } else {
            requests.push(RegistryAddress::parse(argument).map_err(|error| error.to_string())?);
        }
    }
    if requests.is_empty() {
        return Err("at least one component is required".to_string());
    }
    Ok((requests, dry_run))
}

fn configured_catalog(
    project_root: &std::path::Path,
    configuration: &ComponentsConfiguration,
) -> Result<(RegistryCatalog, ConfiguredRegistryReader), String> {
    let official_manifest = include_bytes!("../../../registry/registry.json");
    let loader = RegistrySourceLoader::new(EmbeddedRegistry::new(official_manifest, project_root));
    let mut catalog = RegistryCatalog::new();
    for (namespace, configured_source) in &configuration.registries {
        if matches!(configured_source, RegistrySource::Embedded) {
            let official = LoadedRegistry::from_embedded_manifest(
                official_manifest,
                "embedded official registry",
            )
            .map_err(|error| error.to_string())?;
            if &official.manifest.namespace != namespace {
                return Err(format!(
                    "embedded registry is {}; it cannot be configured as {}",
                    official.manifest.namespace, namespace
                ));
            }
            catalog
                .insert(official)
                .map_err(|error| error.to_string())?;
            continue;
        }
        let source = match configured_source {
            RegistrySource::Local { path } => RegistrySource::Local {
                path: project_root.join(path).display().to_string(),
            },
            source => source.clone(),
        };
        let registry = loader
            .load(namespace, &source)
            .map_err(|error| error.to_string())?;
        catalog
            .insert(registry)
            .map_err(|error| error.to_string())?;
    }
    Ok((catalog, ConfiguredRegistryReader { loader }))
}

struct ConfiguredRegistryReader {
    loader: RegistrySourceLoader,
}

impl RegistryFileReader for ConfiguredRegistryReader {
    fn read(&self, item: &ResolvedRegistryItem, source: &str) -> Result<Vec<u8>, AddError> {
        match (&item.location, source) {
            (RegistryLocation::Embedded { .. }, "lib/cn.rs") => {
                Ok(include_bytes!("../../../registry/lib/cn.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/button.rs") => {
                Ok(include_bytes!("../../../registry/ui/button.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/dialog.rs") => {
                Ok(include_bytes!("../../../registry/ui/dialog.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, _) => Err(AddError::ReadFailed {
                path: format!("{} from {}", source, item.location),
                message: "this adico binary does not embed the requested registry source"
                    .to_string(),
            }),
            _ => self
                .loader
                .read_resolved_source(item, source)
                .map_err(|error| AddError::ReadFailed {
                    path: format!("{} from {}", source, item.location),
                    message: error.to_string(),
                }),
        }
    }
}

fn run_init(arguments: &[String]) {
    let (options, dry_run) = match parse_init_options(arguments) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("adico init: {error}");
            std::process::exit(2);
        }
    };
    let current_directory = match env::current_dir() {
        Ok(directory) => directory,
        Err(error) => {
            eprintln!("adico init: cannot determine current directory: {error}");
            std::process::exit(1);
        }
    };
    let plan = match plan_init(&current_directory, &options) {
        Ok(plan) => plan,
        Err(error) => {
            eprintln!("adico init: {error}");
            std::process::exit(1);
        }
    };
    println!("adico init plan for {}", plan.project.package_name);
    println!(
        "  default registry: {}",
        plan.configuration.default_registry
    );
    if plan.directories_to_create.is_empty() {
        println!("  directories: already prepared");
    } else {
        for directory in &plan.directories_to_create {
            println!("  create directory: {}", directory.display());
        }
    }
    if plan.has_changes() {
        println!("  components.json: {}", plan.configuration_path.display());
    } else {
        println!("  components.json: already matches this plan");
    }
    println!(
        "  module roots reserved: {}",
        display_paths(&plan.module_roots)
    );
    println!("  CSS entry reserved: {}", plan.css_entry.display());
    if dry_run {
        return;
    }
    if let Err(error) = plan.apply() {
        eprintln!("adico init: {error}");
        std::process::exit(1);
    }
    println!("adico is ready.");
}

fn parse_init_options(arguments: &[String]) -> Result<(InitOptions, bool), String> {
    let mut options = InitOptions::default();
    let mut default_registry = None;
    let mut dry_run = false;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--dry-run" => dry_run = true,
            "--default-registry" => {
                index += 1;
                let value = arguments
                    .get(index)
                    .ok_or("--default-registry requires an @namespace value")?;
                default_registry = Some(
                    value
                        .parse::<RegistryNamespace>()
                        .map_err(|error| error.to_string())?,
                );
            }
            "--registry" => {
                index += 1;
                let value = arguments
                    .get(index)
                    .ok_or("--registry requires @namespace=<embedded|relative-path|https-url>")?;
                let (namespace, source) = parse_registry_assignment(value)?;
                options.registries.insert(namespace, source);
            }
            argument => return Err(format!("unknown init option {argument:?}")),
        }
        index += 1;
    }
    if let Some(default_registry) = default_registry {
        options.default_registry = default_registry;
    }
    Ok((options, dry_run))
}

fn parse_registry_assignment(value: &str) -> Result<(RegistryNamespace, RegistrySource), String> {
    let (namespace, source) = value
        .split_once('=')
        .ok_or("--registry must use @namespace=<embedded|relative-path|https-url>")?;
    let namespace = namespace
        .parse()
        .map_err(|error: adico_registry_core::RegistryError| error.to_string())?;
    let source = match source {
        "embedded" => RegistrySource::Embedded,
        source if source.starts_with("https://") || source.starts_with("http://") => {
            RegistrySource::Https {
                url: source.to_string(),
            }
        }
        source if !source.is_empty() => RegistrySource::Local {
            path: PathBuf::from(source).display().to_string(),
        },
        _ => return Err("registry source must not be empty".to_string()),
    };
    Ok((namespace, source))
}

fn display_paths(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use adico_registry_core::{ComponentPaths, CssConfiguration, ThemeConfiguration};
    use sha2::{Digest, Sha256};
    use std::collections::BTreeMap;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn checksum(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    #[test]
    fn bare_requests_use_the_project_selected_local_registry() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("valid time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "adico-configured-registry-test-{}-{nonce}",
            std::process::id()
        ));
        let registry = root.join("company-registry");
        fs::create_dir_all(registry.join("ui")).expect("registry directory should exist");
        let source = b"pub fn company_button() {}\n";
        fs::write(registry.join("ui/button.rs"), source).expect("registry source should exist");
        fs::write(
            registry.join("registry.json"),
            format!(
                r#"{{"formatVersion":1,"namespace":"@awwwkshay","name":"company","compatibility":{{"cli":">=0.1.0"}},"items":[{{"name":"button","type":"registry:ui","description":"company button","files":[{{"source":"ui/button.rs","targetRoot":"ui","target":"button.rs","checksum":"{}"}}]}}]}}"#,
                checksum(source)
            ),
        )
        .expect("registry manifest should exist");
        let company: RegistryNamespace = "@awwwkshay".parse().expect("valid namespace");
        let configuration = ComponentsConfiguration {
            schema: None,
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
            registries: BTreeMap::from([
                (
                    "@adico".parse().expect("valid namespace"),
                    RegistrySource::Embedded,
                ),
                (
                    company.clone(),
                    RegistrySource::Local {
                        path: "company-registry".to_string(),
                    },
                ),
            ]),
            default_registry: company,
        };
        let (catalog, _) = configured_catalog(&root, &configuration).expect("catalog should load");
        let plan = catalog
            .resolve(
                &configuration.default_registry,
                &[RegistryAddress::parse("button").expect("valid request")],
            )
            .expect("bare company item should resolve");
        assert_eq!(plan.items[0].address.to_string(), "@awwwkshay/button");
        fs::remove_dir_all(root).expect("temporary project should be removable");
    }
}
