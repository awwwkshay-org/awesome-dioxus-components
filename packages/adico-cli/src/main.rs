//! The `adico` command-line interface.

use std::env;
use std::path::PathBuf;

use adico_cli::init::{InitOptions, plan_init};
use adico_registry_core::{RegistryNamespace, RegistrySource};

fn main() {
    let arguments: Vec<_> = env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        Some("init") => run_init(&arguments[1..]),
        _ => {
            eprintln!(
                "usage:\n  adico init [--default-registry <@namespace>] [--registry <@namespace>=<embedded|relative-path|https-url>] [--dry-run]"
            );
            std::process::exit(2);
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
