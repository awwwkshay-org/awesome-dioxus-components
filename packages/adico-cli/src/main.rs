//! The `adico` command-line interface.

use std::env;
use std::path::PathBuf;

use adico_cli::add::{AddError, RegistryFileReader, plan_component_add, plan_component_add_all};
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
        Some("list") => run_list(&arguments[1..]),
        Some("view") => run_view(&arguments[1..]),
        Some("css") => run_css(&arguments[1..]),
        _ => {
            eprintln!(
                "usage:\n  adico init [--default-registry <@namespace>] [--registry <@namespace>=<embedded|relative-path|https-url>] [--dry-run]\n  adico add <component...> [--dry-run] [--replace]\n  adico list [--registry <@namespace>]\n  adico view <component>\n  adico css build\n  adico css check"
            );
            std::process::exit(2);
        }
    }
}

fn run_list(arguments: &[String]) {
    let namespace = match parse_list_options(arguments) {
        Ok(namespace) => namespace,
        Err(error) => exit_command_error("list", 2, error),
    };
    let (_, configuration, catalog) = match current_project_catalog() {
        Ok(result) => result,
        Err(error) => exit_command_error("list", 1, error),
    };
    let namespace = namespace.unwrap_or(configuration.default_registry);
    let items = match catalog.items_in(&namespace) {
        Ok(items) => items,
        Err(error) => exit_command_error("list", 1, error.to_string()),
    };
    print!("{}", render_registry_list(&namespace, &items));
}

fn run_view(arguments: &[String]) {
    let request = match parse_view_options(arguments) {
        Ok(request) => request,
        Err(error) => exit_command_error("view", 2, error),
    };
    let (_, configuration, catalog) = match current_project_catalog() {
        Ok(result) => result,
        Err(error) => exit_command_error("view", 1, error),
    };
    let plan = match catalog.resolve(&configuration.default_registry, &[request]) {
        Ok(plan) => plan,
        Err(error) => exit_command_error("view", 1, error.to_string()),
    };
    let requested = plan
        .requested
        .first()
        .expect("one request is required by parse_view_options");
    let item = plan
        .items
        .iter()
        .find(|item| item.address == *requested)
        .expect("a resolved requested item is included in its install plan");
    print!("{}", render_registry_item(item));
}

fn parse_list_options(arguments: &[String]) -> Result<Option<RegistryNamespace>, String> {
    let mut namespace = None;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--registry" => {
                index += 1;
                let value = arguments
                    .get(index)
                    .ok_or("--registry requires a configured @namespace")?;
                if namespace.is_some() {
                    return Err("--registry may be provided only once".to_string());
                }
                namespace = Some(
                    value
                        .parse()
                        .map_err(|error: adico_registry_core::RegistryError| error.to_string())?,
                );
            }
            argument => return Err(format!("unknown list option {argument:?}")),
        }
        index += 1;
    }
    Ok(namespace)
}

fn parse_view_options(arguments: &[String]) -> Result<RegistryAddress, String> {
    match arguments {
        [request] if !request.starts_with('-') => {
            RegistryAddress::parse(request).map_err(|error| error.to_string())
        }
        [] => Err("a component address is required".to_string()),
        _ => Err("view accepts exactly one component address".to_string()),
    }
}

fn current_project_catalog() -> Result<(PathBuf, ComponentsConfiguration, RegistryCatalog), String>
{
    let current = env::current_dir()
        .map_err(|error| format!("cannot determine current directory: {error}"))?;
    let project = discover_dioxus_project(&current).map_err(|error| error.to_string())?;
    let root = project
        .package_manifest_path
        .parent()
        .expect("manifest has parent")
        .to_path_buf();
    let configuration = read_components_configuration(&root)?;
    let (catalog, _) = configured_catalog(&root, &configuration)?;
    Ok((root, configuration, catalog))
}

fn read_components_configuration(
    root: &std::path::Path,
) -> Result<ComponentsConfiguration, String> {
    let contents = std::fs::read_to_string(root.join("components.json"))
        .map_err(|_| "valid components.json is required; run adico init first".to_string())?;
    ComponentsConfiguration::parse(&contents)
        .map_err(|error| format!("components.json is invalid: {error}"))
}

fn exit_command_error(command: &str, status: i32, error: String) -> ! {
    eprintln!("adico {command}: {error}");
    std::process::exit(status);
}

fn render_registry_list(namespace: &RegistryNamespace, items: &[ResolvedRegistryItem]) -> String {
    let mut output = format!("Available components from {namespace}:\n");
    if items.is_empty() {
        output.push_str("  (none)\n");
        return output;
    }
    for item in items {
        output.push_str(&format!(
            "  {:<24} {:<20} {}\n",
            item.address,
            item.item.item_type.as_str(),
            item.item.description
        ));
    }
    output
}

fn render_registry_item(resolved: &ResolvedRegistryItem) -> String {
    let item = &resolved.item;
    let compatibility = item
        .compatibility
        .as_ref()
        .unwrap_or(&resolved.registry_compatibility);
    let mut output = format!(
        "{}\nType: {}\nDescription: {}\nCompatibility: CLI {}{}\n",
        resolved.address,
        item.item_type.as_str(),
        item.description,
        compatibility.cli,
        compatibility
            .runtime
            .as_deref()
            .map(|runtime| format!(", runtime {runtime}"))
            .unwrap_or_default(),
    );
    output.push_str("Files:\n");
    for file in &item.files {
        output.push_str(&format!(
            "  {} -> {}/{}\n",
            file.source,
            file.target_root.as_str(),
            file.target
        ));
    }
    render_values(
        &mut output,
        "Registry dependencies",
        &item.registry_dependencies,
    );
    let cargo_dependencies = item
        .cargo_dependencies
        .iter()
        .map(|dependency| {
            let package = dependency
                .package
                .as_deref()
                .map(|package| format!(" (package {package})"))
                .unwrap_or_default();
            let features = if dependency.features.is_empty() {
                String::new()
            } else {
                format!(" [{}]", dependency.features.join(", "))
            };
            let default_features = if dependency.default_features {
                String::new()
            } else {
                " (default features disabled)".to_string()
            };
            format!(
                "{} {}{}{}{}",
                dependency.crate_name, dependency.version, package, features, default_features
            )
        })
        .collect::<Vec<_>>();
    render_values(&mut output, "Cargo dependencies", &cargo_dependencies);
    let mut style = Vec::new();
    if item.style.semantic_tokens {
        style.push("semantic tokens".to_string());
    }
    if item.style.radius_token {
        style.push("radius token".to_string());
    }
    style.extend(
        item.style
            .utilities
            .iter()
            .map(|utility| format!("utility {utility}")),
    );
    render_values(&mut output, "Style requirements", &style);
    if let Some(provenance) = &item.provenance {
        output.push_str(&format!("Provenance: {}", provenance.record));
        if let Some(revision) = &provenance.revision {
            output.push_str(&format!(" (revision {revision})"));
        }
        output.push('\n');
    } else {
        output.push_str("Provenance: none declared\n");
    }
    output
}

fn render_values(output: &mut String, label: &str, values: &[String]) {
    output.push_str(&format!("{label}:\n"));
    if values.is_empty() {
        output.push_str("  (none)\n");
    } else {
        for value in values {
            output.push_str(&format!("  {value}\n"));
        }
    }
}

fn run_css(arguments: &[String]) {
    let mode = match arguments.first().map(String::as_str) {
        Some("build") => CssMode::Build,
        Some("check") => CssMode::Check,
        _ => {
            eprintln!("usage:\n  adico css build\n  adico css check");
            std::process::exit(2);
        }
    };
    let current = match env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("adico css: cannot determine current directory: {error}");
            std::process::exit(1);
        }
    };
    let project = match discover_dioxus_project(&current) {
        Ok(project) => project,
        Err(error) => {
            eprintln!("adico css: {error}");
            std::process::exit(1);
        }
    };
    let root = project
        .package_manifest_path
        .parent()
        .expect("manifest has parent");
    let configuration = match read_components_configuration(root) {
        Ok(configuration) => configuration,
        Err(error) => {
            eprintln!("adico css: {error}");
            std::process::exit(1);
        }
    };
    let result = match mode {
        CssMode::Build => adico_cli::css_build::build_project(root, &configuration.css.entry)
            .map(|()| "adico css build complete."),
        CssMode::Check => adico_cli::css_build::check_project(root, &configuration.css.entry)
            .map(|()| "adico css check: assets/tailwind.css is up to date."),
    };
    match result {
        Ok(message) => println!("{message}"),
        Err(error) => {
            eprintln!("adico css: {error}");
            std::process::exit(1);
        }
    }
}

enum CssMode {
    Build,
    Check,
}

/// Best-effort compile-after-mutate step for `init`/`add`: failures here
/// (most likely a first-run network fetch of the standalone Tailwind CLI
/// failing) are printed but never abort the command, since the component
/// install/scaffold itself already succeeded and is more important than an
/// automatic CSS refresh a human can always retry with `adico css build`.
fn build_css_best_effort(
    command: &str,
    root: &std::path::Path,
    configuration: &ComponentsConfiguration,
) {
    let input = root.join(&configuration.css.entry);
    if !input.is_file() {
        return;
    }
    if let Err(error) = adico_cli::css_build::build_project(root, &configuration.css.entry) {
        eprintln!(
            "adico {command}: warning: could not compile Tailwind CSS ({error}); run `adico css build` once this is resolved."
        );
    }
}

/// Names the exact one-line fix when a consumer's entrypoint does not yet
/// link the compiled stylesheet -- printed instead of silently reporting
/// success while the project cannot render styled output.
fn warn_if_entrypoint_missing_stylesheet(command: &str, entrypoint: &std::path::Path) {
    let contents = std::fs::read_to_string(entrypoint).unwrap_or_default();
    if !contents.contains("assets/tailwind.css") {
        eprintln!(
            "adico {command}: note: {} does not yet link the compiled stylesheet. Add inside your root component's rsx! block:\n  document::Stylesheet {{ href: asset!(\"/assets/tailwind.css\") }}\n(and enable the `document` Dioxus feature, plus `const TAILWIND_CSS: Asset = asset!(\"/assets/tailwind.css\");`, if not already present).",
            entrypoint.display()
        );
    }
}

fn run_add(arguments: &[String]) {
    let (request, dry_run, replace) = match parse_add_options(arguments) {
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
    let plan = match match request {
        AddRequest::Items(requests) => plan_component_add(
            &catalog,
            root,
            &project.package_manifest_path,
            &configuration,
            &requests,
            &reader,
            replace,
        ),
        AddRequest::All => plan_component_add_all(
            &catalog,
            root,
            &project.package_manifest_path,
            &configuration,
            &reader,
            replace,
        ),
    } {
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
    build_css_best_effort("add", root, &configuration);
    warn_if_entrypoint_missing_stylesheet("add", &project.entrypoint);
    println!("adico add complete.");
}

enum AddRequest {
    Items(Vec<RegistryAddress>),
    All,
}

fn parse_add_options(arguments: &[String]) -> Result<(AddRequest, bool, bool), String> {
    let mut dry_run = false;
    let mut replace = false;
    let mut requests = Vec::new();
    let mut all = false;
    for argument in arguments {
        if argument == "--dry-run" {
            dry_run = true;
        } else if argument == "--replace" {
            replace = true;
        } else if argument == "--all" {
            all = true;
        } else if argument.starts_with('-') {
            return Err(format!("unknown add option {argument:?}"));
        } else {
            requests.push(RegistryAddress::parse(argument).map_err(|error| error.to_string())?);
        }
    }
    if all && !requests.is_empty() {
        return Err("--all cannot be combined with named components".to_string());
    }
    if all {
        return Ok((AddRequest::All, dry_run, replace));
    }
    if requests.is_empty() {
        return Err("at least one component is required".to_string());
    }
    Ok((AddRequest::Items(requests), dry_run, replace))
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
            (RegistryLocation::Embedded { .. }, "ui/select.rs") => {
                Ok(include_bytes!("../../../registry/ui/select.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/badge.rs") => {
                Ok(include_bytes!("../../../registry/ui/badge.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/card.rs") => {
                Ok(include_bytes!("../../../registry/ui/card.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/input.rs") => {
                Ok(include_bytes!("../../../registry/ui/input.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/item.rs") => {
                Ok(include_bytes!("../../../registry/ui/item.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/pagination.rs") => {
                Ok(include_bytes!("../../../registry/ui/pagination.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/skeleton.rs") => {
                Ok(include_bytes!("../../../registry/ui/skeleton.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/textarea.rs") => {
                Ok(include_bytes!("../../../registry/ui/textarea.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/sheet.rs") => {
                Ok(include_bytes!("../../../registry/ui/sheet.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/tooltip.rs") => {
                Ok(include_bytes!("../../../registry/ui/tooltip.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/popover.rs") => {
                Ok(include_bytes!("../../../registry/ui/popover.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/hover_card.rs") => {
                Ok(include_bytes!("../../../registry/ui/hover_card.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/dropdown_menu.rs") => {
                Ok(include_bytes!("../../../registry/ui/dropdown_menu.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/context_menu.rs") => {
                Ok(include_bytes!("../../../registry/ui/context_menu.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/menubar.rs") => {
                Ok(include_bytes!("../../../registry/ui/menubar.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/combobox.rs") => {
                Ok(include_bytes!("../../../registry/ui/combobox.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/calendar.rs") => {
                Ok(include_bytes!("../../../registry/ui/calendar.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/date_picker.rs") => {
                Ok(include_bytes!("../../../registry/ui/date_picker.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/sidebar.rs") => {
                Ok(include_bytes!("../../../registry/ui/sidebar.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/aspect_ratio.rs") => {
                Ok(include_bytes!("../../../registry/ui/aspect_ratio.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/label.rs") => {
                Ok(include_bytes!("../../../registry/ui/label.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/progress.rs") => {
                Ok(include_bytes!("../../../registry/ui/progress.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/avatar.rs") => {
                Ok(include_bytes!("../../../registry/ui/avatar.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/checkbox.rs") => {
                Ok(include_bytes!("../../../registry/ui/checkbox.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/collapsible.rs") => {
                Ok(include_bytes!("../../../registry/ui/collapsible.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/switch.rs") => {
                Ok(include_bytes!("../../../registry/ui/switch.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/toggle.rs") => {
                Ok(include_bytes!("../../../registry/ui/toggle.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/accordion.rs") => {
                Ok(include_bytes!("../../../registry/ui/accordion.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/radio_group.rs") => {
                Ok(include_bytes!("../../../registry/ui/radio_group.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/tabs.rs") => {
                Ok(include_bytes!("../../../registry/ui/tabs.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/toggle_group.rs") => {
                Ok(include_bytes!("../../../registry/ui/toggle_group.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/scroll_area.rs") => {
                Ok(include_bytes!("../../../registry/ui/scroll_area.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/alert_dialog.rs") => {
                Ok(include_bytes!("../../../registry/ui/alert_dialog.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/toast.rs") => {
                Ok(include_bytes!("../../../registry/ui/toast.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/slider.rs") => {
                Ok(include_bytes!("../../../registry/ui/slider.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/toolbar.rs") => {
                Ok(include_bytes!("../../../registry/ui/toolbar.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/virtual_list.rs") => {
                Ok(include_bytes!("../../../registry/ui/virtual_list.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/tag_group.rs") => {
                Ok(include_bytes!("../../../registry/ui/tag_group.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/drag_and_drop_list.rs") => {
                Ok(include_bytes!("../../../registry/ui/drag_and_drop_list.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/color_picker.rs") => {
                Ok(include_bytes!("../../../registry/ui/color_picker.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/mode_toggle.rs") => {
                Ok(include_bytes!("../../../registry/ui/mode_toggle.rs").to_vec())
            }
            (RegistryLocation::Embedded { .. }, "ui/theme_switcher.rs") => {
                Ok(include_bytes!("../../../registry/ui/theme_switcher.rs").to_vec())
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
    let root = plan
        .project
        .package_manifest_path
        .parent()
        .expect("manifest has parent");
    build_css_best_effort("init", root, &plan.configuration);
    warn_if_entrypoint_missing_stylesheet("init", &plan.project.entrypoint);
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
    fn discovery_uses_default_and_explicit_configured_sources_without_mutation() {
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
        let company_items = catalog
            .items_in(&configuration.default_registry)
            .expect("company items should list");
        assert_eq!(
            render_registry_list(&configuration.default_registry, &company_items),
            "Available components from @awwwkshay:\n  @awwwkshay/button registry:ui          company button\n"
        );
        let official: RegistryNamespace = "@adico".parse().expect("valid namespace");
        let official_items = catalog
            .items_in(&official)
            .expect("official items should list");
        assert_eq!(
            official_items
                .iter()
                .map(|item| item.address.to_string())
                .collect::<Vec<_>>(),
            vec![
                "@adico/accordion".to_string(),
                "@adico/alert-dialog".to_string(),
                "@adico/aspect-ratio".to_string(),
                "@adico/avatar".to_string(),
                "@adico/badge".to_string(),
                "@adico/button".to_string(),
                "@adico/calendar".to_string(),
                "@adico/card".to_string(),
                "@adico/checkbox".to_string(),
                "@adico/cn".to_string(),
                "@adico/collapsible".to_string(),
                "@adico/color-picker".to_string(),
                "@adico/combobox".to_string(),
                "@adico/context-menu".to_string(),
                "@adico/date-picker".to_string(),
                "@adico/dialog".to_string(),
                "@adico/drag-and-drop-list".to_string(),
                "@adico/dropdown-menu".to_string(),
                "@adico/hover-card".to_string(),
                "@adico/input".to_string(),
                "@adico/item".to_string(),
                "@adico/label".to_string(),
                "@adico/menubar".to_string(),
                "@adico/mode-toggle".to_string(),
                "@adico/pagination".to_string(),
                "@adico/popover".to_string(),
                "@adico/progress".to_string(),
                "@adico/radio-group".to_string(),
                "@adico/scroll-area".to_string(),
                "@adico/select".to_string(),
                "@adico/sheet".to_string(),
                "@adico/sidebar".to_string(),
                "@adico/skeleton".to_string(),
                "@adico/slider".to_string(),
                "@adico/switch".to_string(),
                "@adico/tabs".to_string(),
                "@adico/tag-group".to_string(),
                "@adico/textarea".to_string(),
                "@adico/theme-switcher".to_string(),
                "@adico/toast".to_string(),
                "@adico/toggle".to_string(),
                "@adico/toggle-group".to_string(),
                "@adico/toolbar".to_string(),
                "@adico/tooltip".to_string(),
                "@adico/virtual-list".to_string(),
            ]
        );
        let official_plan = catalog
            .resolve(
                &configuration.default_registry,
                &[RegistryAddress::parse("@adico/dialog").expect("valid official request")],
            )
            .expect("official dialog should resolve");
        let dialog = official_plan
            .items
            .iter()
            .find(|item| item.address.to_string() == "@adico/dialog")
            .expect("official dialog should be present");
        let details = render_registry_item(dialog);
        assert!(details.contains("@adico/dialog\nType: registry:ui"));
        assert!(details.contains("Registry dependencies:\n  cn"));
        assert!(details.contains("Cargo dependencies:\n  dioxus =0.7.9"));
        assert!(
            details.contains("Provenance: provenance/records/adico-primitives-dialog-select.json")
        );
        assert!(
            !root.join("src").exists(),
            "discovery must not create consumer source"
        );
        assert!(
            !root.join("components.json").exists(),
            "discovery must not create consumer configuration"
        );
        fs::remove_dir_all(root).expect("temporary project should be removable");
    }

    #[test]
    fn list_and_view_options_reject_ambiguous_or_mutating_forms() {
        let selected = parse_list_options(&["--registry".to_string(), "@adico".to_string()])
            .expect("one namespace should parse")
            .expect("the namespace is supplied");
        assert_eq!(selected.as_str(), "@adico");
        assert!(parse_list_options(&["button".to_string()]).is_err());
        assert!(parse_view_options(&[]).is_err());
        assert!(parse_view_options(&["button".to_string(), "dialog".to_string()]).is_err());
    }

    #[test]
    fn add_all_selects_the_registry_wide_install_path() {
        let (request, dry_run, replace) =
            parse_add_options(&["--all".to_string(), "--dry-run".to_string()])
                .expect("add-all options should parse");
        assert!(matches!(request, AddRequest::All));
        assert!(dry_run);
        assert!(!replace);
        let (_, _, replace) = parse_add_options(&["button".to_string(), "--replace".to_string()])
            .expect("replace should parse");
        assert!(replace);
        assert!(parse_add_options(&["--all".to_string(), "button".to_string()]).is_err());
    }
}
