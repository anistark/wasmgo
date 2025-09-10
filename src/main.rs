#[cfg(feature = "cli")]
use clap::{Parser, Subcommand};
use wasmgo::{CompileConfig, OptimizationLevel, Plugin, TargetType, WasmGoPlugin};

#[cfg(feature = "cli")]
#[derive(Parser)]
#[command(name = "wasmgo")]
#[command(about = "Go WebAssembly plugin for Wasmrun")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
enum Commands {
    /// Run a Go WebAssembly project for execution (default command)
    #[command(alias = "r")]
    Run {
        /// Project path containing go.mod or main.go
        #[arg(short, long, default_value = ".", value_name = "PATH")]
        project: String,

        /// Output directory for compiled files
        #[arg(short, long, default_value = "./dist", value_name = "DIR")]
        output: String,

        /// Optimization level for compilation
        #[arg(long, value_enum, default_value = "release")]
        optimization: CliOptimization,

        /// Enable verbose compilation output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Compile a Go project to WebAssembly
    #[command(alias = "c")]
    Compile {
        /// Project path containing go.mod or main.go
        #[arg(short, long, default_value = ".", value_name = "PATH")]
        project: String,

        /// Output directory for compiled files
        #[arg(short, long, default_value = "./dist", value_name = "DIR")]
        output: String,

        /// Optimization level for compilation
        #[arg(long, value_enum, default_value = "release")]
        optimization: CliOptimization,

        /// Target type for compilation
        #[arg(long, value_enum, default_value = "wasm")]
        target: CliTarget,

        /// Enable verbose compilation output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Inspect project structure, dependencies, and frameworks
    #[command(alias = "check")]
    Inspect {
        /// Project path to inspect
        #[arg(short, long, default_value = ".", value_name = "PATH")]
        project: String,
    },

    /// Check if wasmgo can handle the project
    CanHandle {
        /// Project path to check
        #[arg(value_name = "PATH")]
        project: String,
    },

    /// Check dependencies and system requirements
    CheckDeps,

    /// Clean build artifacts
    Clean {
        /// Project path to clean
        #[arg(value_name = "PATH")]
        project: String,
    },

    /// Show plugin information and capabilities
    Info,

    /// Show supported frameworks and project types
    Frameworks,
}

#[cfg(feature = "cli")]
#[derive(clap::ValueEnum, Clone, Debug)]
enum CliOptimization {
    /// Fast compilation with debug symbols
    Debug,
    /// Balanced optimization for production
    Release,
    /// Smallest possible output size
    Size,
}

#[cfg(feature = "cli")]
#[derive(clap::ValueEnum, Clone, Debug)]
enum CliTarget {
    /// Standard WebAssembly module
    Wasm,
    /// Complete web application bundle
    WebApp,
}

#[cfg(feature = "cli")]
impl From<CliOptimization> for OptimizationLevel {
    fn from(opt: CliOptimization) -> Self {
        match opt {
            CliOptimization::Debug => OptimizationLevel::Debug,
            CliOptimization::Release => OptimizationLevel::Release,
            CliOptimization::Size => OptimizationLevel::Size,
        }
    }
}

#[cfg(feature = "cli")]
impl From<CliTarget> for TargetType {
    fn from(target: CliTarget) -> Self {
        match target {
            CliTarget::Wasm => TargetType::Standard,
            CliTarget::WebApp => TargetType::WebApp,
        }
    }
}

#[cfg(feature = "cli")]
fn print_header() {
    println!(
        "🐹 {} v{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    println!("   {}", env!("CARGO_PKG_DESCRIPTION"));
    println!();
}

#[cfg(feature = "cli")]
fn check_project_validity(plugin: &WasmGoPlugin, project: &str) -> bool {
    if !plugin.can_handle_project(project) {
        eprintln!("❌ Error: Not a valid Go project");
        eprintln!("   Looking for go.mod or .go files in: {project}");
        eprintln!("   Make sure you're in a Go project directory");
        return false;
    }
    true
}

#[cfg(feature = "cli")]
fn check_dependencies(plugin: &WasmGoPlugin) -> bool {
    let missing_deps = plugin.get_builder().check_dependencies();
    if !missing_deps.is_empty() {
        eprintln!("❌ Missing required dependencies:");
        for dep in &missing_deps {
            eprintln!("   • {dep}");
        }
        eprintln!();
        eprintln!("💡 Installation suggestions:");
        if missing_deps.iter().any(|d| d.contains("go")) {
            eprintln!("   • Install Go: https://golang.org/dl/");
        }
        if missing_deps.iter().any(|d| d.contains("tinygo")) {
            eprintln!("   • Install TinyGo: https://tinygo.org/getting-started/install/");
        }
        return false;
    }
    true
}

#[cfg(feature = "cli")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let plugin = WasmGoPlugin::new();

    // Default to Run command if no subcommand is provided
    // Note: this would require making command optional in Cli struct
    match cli.command {
        Commands::Run {
            project,
            output,
            optimization,
            verbose,
        } => {
            if verbose {
                print_header();
                println!("🚀 Preparing Go project for execution...");
                println!("📁 Project: {project}");
                println!("📦 Output: {output}");
                println!("🎯 Optimization: {optimization:?}");
                println!();
            }

            if !check_project_validity(&plugin, &project) {
                std::process::exit(1);
            }

            if !check_dependencies(&plugin) {
                std::process::exit(1);
            }

            let builder = plugin.get_builder();
            let compile_config = CompileConfig {
                project_path: project.clone(),
                output_directory: output,
                verbose,
                optimization_level: optimization.into(),
                target_type: TargetType::Standard,
            };

            match builder.compile(&compile_config) {
                Ok(result) => {
                    if verbose {
                        println!("✅ Project ready for execution!");
                        println!("🎯 Entry point: {}", result.wasm_file_path);
                    } else {
                        println!("{}", result.wasm_file_path);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to prepare project for execution: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Compile {
            project,
            output,
            optimization,
            target,
            verbose,
        } => {
            if verbose {
                print_header();
                println!("🔨 Compiling Go project to WebAssembly...");
                println!("📁 Project: {project}");
                println!("📦 Output: {output}");
                println!("🎯 Optimization: {optimization:?}");
                println!("🏗️  Target: {target:?}");
                println!();
            }

            if !check_project_validity(&plugin, &project) {
                std::process::exit(1);
            }

            if !check_dependencies(&plugin) {
                std::process::exit(1);
            }

            let builder = plugin.get_builder();
            let compile_config = CompileConfig {
                project_path: project.clone(),
                output_directory: output,
                verbose,
                optimization_level: optimization.into(),
                target_type: target.into(),
            };

            match builder.compile(&compile_config) {
                Ok(result) => {
                    println!("✅ Compilation completed successfully!");
                    println!("🎯 WASM file: {}", result.wasm_file_path);

                    if let Some(js_path) = result.js_file_path {
                        println!("📄 JS bindings: {js_path}");
                    }

                    if !result.additional_files.is_empty() {
                        println!("📂 Additional files: {}", result.additional_files.len());
                        if verbose {
                            for file in result.additional_files {
                                println!("   • {file}");
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Compilation failed: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Inspect { project } => {
            print_header();
            println!("🔍 Inspecting Go project...");
            println!();

            if plugin.can_handle_project(&project) {
                println!("📊 Project Analysis");
                println!("═══════════════════");

                if let Ok(directory_entries) = std::fs::read_dir(&project) {
                    let go_files: Vec<_> = directory_entries
                        .filter_map(|entry| entry.ok())
                        .filter(|entry| {
                            entry
                                .path()
                                .extension()
                                .map(|extension| extension.to_string_lossy().to_lowercase() == "go")
                                .unwrap_or(false)
                        })
                        .map(|entry| entry.file_name().to_string_lossy().to_string())
                        .collect();

                    if !go_files.is_empty() {
                        println!("📁 Go files: {}", go_files.join(", "));
                    }
                }

                if std::path::Path::new(&project).join("go.mod").exists() {
                    println!("📦 Module: Found go.mod");
                }

                println!("🎯 Type: Go WebAssembly project");
                println!("🔧 Build Tool: TinyGo");

                println!();
                println!("📋 Dependencies");
                println!("═══════════════");

                let missing = plugin.get_builder().check_dependencies();
                if missing.is_empty() {
                    println!("✅ go - Go compiler");
                    println!("✅ tinygo - WebAssembly compiler for Go");
                    println!();
                    println!("🎉 Project is ready to compile!");
                } else {
                    for dep in &missing {
                        println!("❌ {dep}");
                    }
                    println!();
                    println!(
                        "⚠️  Some required dependencies are missing. Install them to proceed."
                    );
                    std::process::exit(1);
                }
            } else {
                eprintln!("❌ Invalid project: Not a Go project");
                eprintln!("   Looking for go.mod or .go files in: {project}");
                std::process::exit(1);
            }
        }

        Commands::CanHandle { project } => {
            if plugin.can_handle_project(&project) {
                println!("✅ Yes, wasmgo can handle this project");
                if std::path::Path::new(&project).join("go.mod").exists() {
                    println!("📁 Found go.mod at: {project}/go.mod");
                } else {
                    println!("📁 Found Go files in: {project}");
                }
            } else {
                println!("❌ No, wasmgo cannot handle this project");
                println!("🔍 Looking for go.mod or .go files in: {project}");
                std::process::exit(1);
            }
        }

        Commands::CheckDeps => {
            print_header();
            println!("🔍 Checking system dependencies...");
            println!();

            let missing = plugin.get_builder().check_dependencies();

            if missing.is_empty() {
                println!("✅ All required dependencies are available!");
                println!();
                println!("📋 Available tools:");
                println!("   ✅ go - Go compiler");
                println!("   ✅ tinygo - WebAssembly compiler for Go");
            } else {
                println!("❌ Missing required dependencies:");
                for dep in &missing {
                    println!("   • {dep}");
                }

                println!();
                println!("💡 Installation suggestions:");
                println!("   • Install Go: https://golang.org/dl/");
                println!("   • Install TinyGo: https://tinygo.org/getting-started/install/");
                println!("   • On macOS with Homebrew: brew install go tinygo");
                println!("   • On Ubuntu/Debian: sudo apt install golang-go && follow TinyGo instructions");

                std::process::exit(1);
            }
        }

        Commands::Clean { project } => {
            println!("🧹 Cleaning project artifacts: {project}");

            // For Go projects, we mainly clean any built WASM files
            let dist_path = std::path::Path::new(&project).join("dist");
            if dist_path.exists() {
                match std::fs::remove_dir_all(&dist_path) {
                    Ok(_) => println!("✅ Cleaned dist directory"),
                    Err(e) => println!("⚠️  Failed to clean dist directory: {e}"),
                }
            }

            println!("✅ Project cleaned successfully!");
        }

        Commands::Info => {
            print_header();
            println!("🔧 Plugin Information");
            println!("═════════════════════");
            let plugin_info = plugin.info();
            println!("Name: {}", plugin_info.name);
            println!("Version: {}", plugin_info.version);
            println!("Description: {}", plugin_info.description);
            println!("Author: {}", plugin_info.author);

            println!();
            println!("🎯 Capabilities");
            println!("═══════════════");
            println!("✅ Standard WASM compilation");
            println!("✅ TinyGo integration");
            println!("✅ Multiple optimization levels");
            println!("✅ Go module support");
            println!();

            println!("📄 Usage");
            println!("════════");
            println!("Primary (via Wasmrun):");
            println!("   wasmrun run ./my-go-project");
            println!("   wasmrun compile ./my-project --optimization size");
            println!();
            println!("Standalone (testing/development):");
            println!("   {} run ./my-project", env!("CARGO_PKG_NAME"));
            println!(
                "   {} compile ./my-project --target webapp",
                env!("CARGO_PKG_NAME")
            );
            println!("   {} inspect ./my-project", env!("CARGO_PKG_NAME"));
        }

        Commands::Frameworks => {
            print_header();
            println!("🌐 Supported Frameworks & Project Types");
            println!("═══════════════════════════════════════");
            println!();

            println!("📦 Project Types:");
            println!("   • Standard WASM    - Basic Go → WebAssembly compilation via TinyGo");
            println!("   • Web Applications - Full Go web apps compiled to WebAssembly");
            println!();

            println!("🔧 Build Tools:");
            println!("   • TinyGo           - Primary WebAssembly compiler for Go");
            println!("   • go               - Standard Go toolchain for dependency management");
            println!();

            println!("🎯 Optimization Levels:");
            println!("   • debug            - Fast compilation, debug symbols");
            println!("   • release          - Balanced optimization");
            println!("   • size             - Smallest possible output");
        }
    }

    Ok(())
}

#[cfg(not(feature = "cli"))]
fn main() {
    println!("Wasmrun Go Plugin v{}", env!("CARGO_PKG_VERSION"));
    println!("This plugin is designed to be used with the Wasmrun WebAssembly runtime.");
    println!("Configuration is stored in Cargo.toml [package.metadata.wasm-plugin] section.");
    println!();
    println!("Install the CLI feature to use this binary standalone:");
    println!("  cargo install wasmgo --features cli");
    println!();
    println!("Or use with Wasmrun:");
    println!("  wasmrun plugin install wasmgo");
    println!("  wasmrun run ./my-go-project");
}
