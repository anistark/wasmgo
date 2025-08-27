use crate::{
    BuildConfig, BuildResult, CommandExecutor, PathResolver, Plugin, PluginCapabilities,
    PluginInfo, PluginResult, PluginSource, PluginType, WasmBuilder,
};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct CargoToml {
    package: CargoPackage,
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    name: String,
    version: String,
    authors: Vec<String>,
    description: String,
    #[allow(dead_code)]
    homepage: Option<String>,
    #[allow(dead_code)]
    repository: Option<String>,
    #[allow(dead_code)]
    license: Option<String>,
    #[allow(dead_code)]
    keywords: Option<Vec<String>>,
    #[allow(dead_code)]
    categories: Option<Vec<String>>,
    metadata: Option<PackageMetadata>,
}

#[derive(Debug, Deserialize)]
struct PackageMetadata {
    #[serde(rename = "wasm-plugin")]
    wasm_plugin: WasmPluginConfig,
}

#[derive(Debug, Deserialize)]
struct WasmPluginConfig {
    name: String,
    extensions: Vec<String>,
    entry_files: Vec<String>,
    capabilities: WasmPluginCapabilities,
    dependencies: WasmPluginDependencies,
}

#[derive(Debug, Deserialize)]
struct WasmPluginCapabilities {
    compile_wasm: bool,
    compile_webapp: bool,
    live_reload: bool,
    optimization: bool,
    custom_targets: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct WasmPluginDependencies {
    tools: Vec<String>,
}

pub struct GoPlugin {
    plugin_info: PluginInfo,
}

impl GoPlugin {
    pub fn new() -> Self {
        let plugin_info = Self::load_plugin_info()
            .expect("Failed to load plugin configuration from Cargo.toml [package.metadata.wasm-plugin] section");

        Self { plugin_info }
    }

    fn load_plugin_info() -> Result<PluginInfo, Box<dyn std::error::Error>> {
        let cargo_config = Self::read_cargo_toml()?;
        Ok(Self::create_plugin_info(cargo_config))
    }

    fn read_cargo_toml() -> Result<CargoToml, Box<dyn std::error::Error>> {
        if let Ok(content) = fs::read_to_string("Cargo.toml") {
            return Ok(toml::from_str(&content)?);
        }

        let embedded_content = include_str!("../Cargo.toml");
        Ok(toml::from_str(embedded_content)?)
    }

    fn create_plugin_info(cargo_config: CargoToml) -> PluginInfo {
        let wasm_plugin = cargo_config
            .package
            .metadata
            .expect("Missing [package.metadata.wasm-plugin] section in Cargo.toml")
            .wasm_plugin;

        let author = cargo_config
            .package
            .authors
            .first()
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());

        let source = Some(PluginSource::CratesIo {
            name: cargo_config.package.name,
            version: cargo_config.package.version.clone(),
        });

        PluginInfo {
            name: wasm_plugin.name,
            version: cargo_config.package.version,
            description: cargo_config.package.description,
            author,
            extensions: wasm_plugin.extensions,
            entry_files: wasm_plugin.entry_files,
            plugin_type: PluginType::External,
            source,
            dependencies: wasm_plugin.dependencies.tools,
            capabilities: PluginCapabilities {
                compile_wasm: wasm_plugin.capabilities.compile_wasm,
                compile_webapp: wasm_plugin.capabilities.compile_webapp,
                live_reload: wasm_plugin.capabilities.live_reload,
                optimization: wasm_plugin.capabilities.optimization,
                custom_targets: wasm_plugin.capabilities.custom_targets,
            },
        }
    }

    fn find_entry_file(&self, project_directory: &str) -> PluginResult<PathBuf> {
        let entry_file_candidates: Vec<&str> = self
            .plugin_info
            .entry_files
            .iter()
            .map(|s| s.as_str())
            .collect();

        for entry_filename in entry_file_candidates.iter() {
            let entry_file_path = Path::new(project_directory).join(entry_filename);
            if entry_file_path.exists() {
                return Ok(entry_file_path);
            }
        }

        if let Ok(directory_entries) = fs::read_dir(project_directory) {
            for directory_entry in directory_entries.flatten() {
                if let Some(file_extension) = directory_entry.path().extension() {
                    if file_extension == "go" {
                        return Ok(directory_entry.path());
                    }
                }
            }
        }

        Err(crate::PluginError::MissingEntryFile {
            candidates: self.plugin_info.entry_files.clone(),
        })
    }
}

impl Plugin for GoPlugin {
    fn info(&self) -> &PluginInfo {
        &self.plugin_info
    }

    fn can_handle_project(&self, project_directory: &str) -> bool {
        let go_module_path = PathResolver::join_paths(project_directory, "go.mod");
        if Path::new(&go_module_path).exists() {
            return true;
        }

        if let Ok(directory_entries) = fs::read_dir(project_directory) {
            for directory_entry in directory_entries.flatten() {
                if let Some(file_extension) = directory_entry.path().extension() {
                    let extension_lowercase = file_extension.to_string_lossy().to_lowercase();
                    if self
                        .plugin_info
                        .extensions
                        .iter()
                        .any(|ext| ext == &extension_lowercase)
                    {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn get_builder(&self) -> Box<dyn WasmBuilder> {
        Box::new(GoPlugin::new())
    }
}

impl WasmBuilder for GoPlugin {
    fn language_name(&self) -> &str {
        "Go"
    }

    fn entry_file_candidates(&self) -> &[&str] {
        &["main.go", "cmd/main.go", "app.go", "go.mod"]
    }

    fn supported_extensions(&self) -> &[&str] {
        &["go"]
    }

    fn check_dependencies(&self) -> Vec<String> {
        let mut missing_dependencies = Vec::new();

        for tool in &self.plugin_info.dependencies {
            if !CommandExecutor::is_tool_installed(tool) {
                let install_hint = match tool.as_str() {
                    "tinygo" => format!("{tool} (install from https://tinygo.org)"),
                    "go" => format!("{tool} (Go compiler)"),
                    _ => tool.clone(),
                };
                missing_dependencies.push(install_hint);
            }
        }

        missing_dependencies
    }

    fn validate_project(&self, project_directory: &str) -> PluginResult<()> {
        PathResolver::validate_directory_exists(project_directory)?;
        let _ = self.find_entry_file(project_directory)?;
        Ok(())
    }

    fn build(&self, build_configuration: &BuildConfig) -> PluginResult<BuildResult> {
        if !CommandExecutor::is_tool_installed("tinygo") {
            return Err(crate::PluginError::BuildToolNotFound {
                tool: "tinygo".to_string(),
            });
        }

        let entry_file_path = self.find_entry_file(&build_configuration.project_path)?;

        PathResolver::ensure_output_directory_exists(&build_configuration.output_directory)?;

        let output_filename = entry_file_path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string()
            + ".wasm";

        println!("🔨 Building with TinyGo...");

        let output_path = Path::new(&build_configuration.output_directory).join(&output_filename);

        let build_command_output = CommandExecutor::execute_command(
            "tinygo",
            &[
                "build",
                "-o",
                &output_path.to_string_lossy(),
                "-target=wasm",
                ".",
            ],
            &build_configuration.project_path,
            build_configuration.verbose,
        )?;

        if !build_command_output.status.success() {
            return Err(crate::PluginError::CompilationFailed {
                reason: format!(
                    "Build failed: {}",
                    String::from_utf8_lossy(&build_command_output.stderr)
                ),
            });
        }

        if !output_path.exists() {
            return Err(crate::PluginError::CompilationFailed {
                reason: "TinyGo build completed but WASM file was not created".to_string(),
            });
        }

        Ok(BuildResult {
            wasm_file_path: output_path.to_string_lossy().to_string(),
            js_file_path: None,
            additional_files: vec![],
            is_wasm_bindgen: false,
        })
    }
}

impl Default for GoPlugin {
    fn default() -> Self {
        Self::new()
    }
}

pub type GoBuilder = GoPlugin;
