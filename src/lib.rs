use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use thiserror::Error;

mod builder;

pub use builder::GoBuilder;
pub use builder::GoPlugin as WasmGoPlugin;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Compilation failed: {reason}")]
    CompilationFailed { reason: String },

    #[error("Build tool not found: {tool}")]
    BuildToolNotFound { tool: String },

    #[error("Invalid project structure: {reason}")]
    InvalidProjectStructure { reason: String },

    #[error("Missing entry file. Expected one of: {candidates:?}")]
    MissingEntryFile { candidates: Vec<String> },

    #[error("Output directory creation failed: {path}")]
    OutputDirectoryCreationFailed { path: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type PluginResult<T> = Result<T, PluginError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginSource {
    CratesIo { name: String, version: String },
    Git { url: String, branch: Option<String> },
    Local { path: PathBuf },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginType {
    Builtin,
    External,
    Registry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct PluginCapabilities {
    pub compile_wasm: bool,
    pub compile_webapp: bool,
    pub live_reload: bool,
    pub optimization: bool,
    pub custom_targets: Vec<String>,
}

impl Default for PluginCapabilities {
    fn default() -> Self {
        Self {
            compile_wasm: true,
            compile_webapp: false,
            live_reload: false,
            optimization: false,
            custom_targets: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub extensions: Vec<String>,
    pub entry_files: Vec<String>,
    pub plugin_type: PluginType,
    pub source: Option<PluginSource>,
    pub dependencies: Vec<String>,
    pub capabilities: PluginCapabilities,
}

pub trait Plugin: Send + Sync {
    fn info(&self) -> &PluginInfo;
    fn can_handle_project(&self, project_path: &str) -> bool;
    fn get_builder(&self) -> Box<dyn WasmBuilder>;
}

#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub project_path: String,
    pub output_directory: String,
    pub verbose: bool,
    pub optimization_level: OptimizationLevel,
    pub target_type: TargetType,
}

#[derive(Debug, Clone)]
pub struct BuildResult {
    pub wasm_file_path: String,
    pub js_file_path: Option<String>,
    pub additional_files: Vec<String>,
    pub is_wasm_bindgen: bool,
}

#[derive(Debug, Clone)]
pub enum OptimizationLevel {
    Debug,
    Release,
    Size,
}

#[derive(Debug, Clone)]
pub enum TargetType {
    Standard,
    Web,
}

pub trait WasmBuilder: Send + Sync {
    fn language_name(&self) -> &str;
    fn entry_file_candidates(&self) -> &[&str];
    fn supported_extensions(&self) -> &[&str];
    fn check_dependencies(&self) -> Vec<String>;
    fn validate_project(&self, project_path: &str) -> PluginResult<()>;
    fn build(&self, config: &BuildConfig) -> PluginResult<BuildResult>;
}

pub struct CommandExecutor;

impl CommandExecutor {
    pub fn is_tool_installed(tool_name: &str) -> bool {
        let version_arg = if tool_name == "tinygo" {
            "version"
        } else {
            "--version"
        };

        Command::new(tool_name)
            .arg(version_arg)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub fn execute_command(
        command_name: &str,
        arguments: &[&str],
        working_directory: &str,
        verbose_output: bool,
    ) -> PluginResult<Output> {
        if verbose_output {
            println!(
                "Executing: {} {} in {}",
                command_name,
                arguments.join(" "),
                working_directory
            );
        }

        let output = Command::new(command_name)
            .args(arguments)
            .current_dir(working_directory)
            .output()
            .map_err(PluginError::Io)?;

        if verbose_output {
            println!(
                "Command output: {}",
                String::from_utf8_lossy(&output.stdout)
            );
            if !output.stderr.is_empty() {
                println!(
                    "Command stderr: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }

        Ok(output)
    }

    pub fn copy_to_output_directory(
        source_file_path: &str,
        output_directory: &str,
        language_name: &str,
    ) -> PluginResult<()> {
        let source_path = Path::new(source_file_path);
        if !source_path.exists() {
            return Err(PluginError::CompilationFailed {
                reason: format!("{language_name} build completed but output file was not found"),
            });
        }

        let filename = source_path.file_name().unwrap();
        let destination_path = Path::new(output_directory).join(filename);

        std::fs::copy(source_path, &destination_path).map_err(PluginError::Io)?;

        println!("📁 Copied to: {}", destination_path.display());
        Ok(())
    }
}

pub struct PathResolver;

impl PathResolver {
    pub fn join_paths(base_path: &str, relative_path: &str) -> String {
        Path::new(base_path)
            .join(relative_path)
            .to_string_lossy()
            .to_string()
    }

    pub fn validate_directory_exists(directory_path: &str) -> PluginResult<()> {
        let directory = Path::new(directory_path);
        if !directory.exists() {
            return Err(PluginError::InvalidProjectStructure {
                reason: format!("Directory does not exist: {directory_path}"),
            });
        }
        if !directory.is_dir() {
            return Err(PluginError::InvalidProjectStructure {
                reason: format!("Path is not a directory: {directory_path}"),
            });
        }
        Ok(())
    }

    pub fn ensure_output_directory_exists(directory_path: &str) -> PluginResult<()> {
        fs::create_dir_all(directory_path).map_err(|_| PluginError::OutputDirectoryCreationFailed {
            path: directory_path.to_string(),
        })
    }

    pub fn is_safe_path(path_to_check: &str) -> bool {
        let path = Path::new(path_to_check);
        !path.to_string_lossy().contains("..")
    }
}

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn wasm_plugin_info() -> PluginInfo {
    WasmGoPlugin::new().info().clone()
}

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn wasm_plugin_create() -> Box<dyn Plugin> {
    Box::new(WasmGoPlugin::new())
}
