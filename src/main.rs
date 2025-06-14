use chakra_go::{BuildConfig, ChakraGoPlugin, OptimizationLevel, Plugin, TargetType};
#[cfg(feature = "cli")]
use clap::{Parser, Subcommand};

#[cfg(feature = "cli")]
#[derive(Parser)]
#[command(name = "chakra-go")]
#[command(about = "Go WebAssembly plugin for Chakra")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
enum Commands {
    Info,
    Check {
        path: String,
    },
    Build {
        path: String,
        #[arg(short, long, default_value = ".")]
        output: String,
        #[arg(short, long)]
        verbose: bool,
        #[arg(short = 'O', long, default_value = "release")]
        optimization: String,
    },
    Deps {
        #[arg(short, long)]
        install: bool,
    },
    Config,
}

#[cfg(feature = "cli")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let plugin = ChakraGoPlugin::new();

    match cli.command {
        Commands::Info => {
            let plugin_info = plugin.info();
            println!("Plugin: {}", plugin_info.name);
            println!("Version: {}", plugin_info.version);
            println!("Description: {}", plugin_info.description);
            println!("Author: {}", plugin_info.author);
            println!("Extensions: {}", plugin_info.extensions.join(", "));
            println!("Entry files: {}", plugin_info.entry_files.join(", "));
            println!("Dependencies: {}", plugin_info.dependencies.join(", "));

            println!("\nCapabilities:");
            println!("  Compile WASM: {}", plugin_info.capabilities.compile_wasm);
            println!(
                "  Compile Web App: {}",
                plugin_info.capabilities.compile_webapp
            );
            println!("  Live Reload: {}", plugin_info.capabilities.live_reload);
            println!("  Optimization: {}", plugin_info.capabilities.optimization);
            println!(
                "  Custom Targets: {}",
                plugin_info.capabilities.custom_targets.join(", ")
            );
        }

        Commands::Check { path } => {
            if plugin.can_handle_project(&path) {
                println!("✅ Can handle project at: {}", path);

                if let Ok(directory_entries) = std::fs::read_dir(&path) {
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
                        println!("Detected Go files: {}", go_files.join(", "));
                    }
                }

                if std::path::Path::new(&path).join("go.mod").exists() {
                    println!("Found go.mod file");
                }
            } else {
                println!("❌ Cannot handle project at: {}", path);
                println!("Expected: Go files (.go) or go.mod");
                std::process::exit(1);
            }
        }

        Commands::Build {
            path,
            output,
            verbose,
            optimization,
        } => {
            let builder = plugin.get_builder();

            let optimization_level = match optimization.as_str() {
                "debug" => OptimizationLevel::Debug,
                "release" => OptimizationLevel::Release,
                "size" => OptimizationLevel::Size,
                _ => {
                    eprintln!(
                        "Invalid optimization level: {}. Use: debug, release, size",
                        optimization
                    );
                    std::process::exit(1);
                }
            };

            let output_directory = if output == "." { path.clone() } else { output };

            let build_configuration = BuildConfig {
                project_path: path.clone(),
                output_directory,
                verbose,
                optimization_level,
                target_type: TargetType::Standard,
            };

            println!("Building Go project: {}", path);

            match builder.build(&build_configuration) {
                Ok(build_result) => {
                    println!("✅ Build successful!");
                    println!("WASM file: {}", build_result.wasm_file_path);
                    if let Some(js_file_path) = build_result.js_file_path {
                        println!("JS file: {}", js_file_path);
                    }
                    if !build_result.additional_files.is_empty() {
                        println!(
                            "Additional files: {}",
                            build_result.additional_files.join(", ")
                        );
                    }
                }
                Err(error) => {
                    eprintln!("❌ Build failed: {}", error);
                    std::process::exit(1);
                }
            }
        }

        Commands::Deps { install } => {
            let builder = plugin.get_builder();
            let missing_dependencies = builder.check_dependencies();

            if missing_dependencies.is_empty() {
                println!("✅ All dependencies are installed!");
            } else {
                println!("❌ Missing dependencies:");
                for dependency in &missing_dependencies {
                    println!("  - {}", dependency);
                }

                if install {
                    println!("\nInstallation instructions:");
                    println!("1. Install Go: https://golang.org/dl/");
                    println!("2. Install TinyGo: https://tinygo.org/getting-started/install/");
                    println!("\nOn macOS with Homebrew:");
                    println!("  brew install go tinygo");
                    println!("\nOn Ubuntu/Debian:");
                    println!("  sudo apt install golang-go");
                    println!("  # Follow TinyGo installation instructions from https://tinygo.org");
                }

                std::process::exit(1);
            }
        }

        Commands::Config => {
            let plugin_info = plugin.info();
            println!("🔧 Chakra-Go Plugin Configuration");
            println!("Source: Cargo.toml [package.metadata.chakra-plugin]");
            println!();
            println!("Plugin Name: {}", plugin_info.name);
            println!("Extensions: {}", plugin_info.extensions.join(", "));
            println!("Entry Files: {}", plugin_info.entry_files.join(", "));
            println!("Dependencies: {}", plugin_info.dependencies.join(", "));
            println!();
            println!("Capabilities:");
            println!(
                "  • WASM Compilation: {}",
                plugin_info.capabilities.compile_wasm
            );
            println!(
                "  • Web App Compilation: {}",
                plugin_info.capabilities.compile_webapp
            );
            println!("  • Live Reload: {}", plugin_info.capabilities.live_reload);
            println!(
                "  • Optimization: {}",
                plugin_info.capabilities.optimization
            );
            println!(
                "  • Custom Targets: [{}]",
                plugin_info.capabilities.custom_targets.join(", ")
            );
            println!();
            println!("💡 To modify configuration, edit the [package.metadata.chakra-plugin] section in Cargo.toml");
        }
    }

    Ok(())
}

#[cfg(not(feature = "cli"))]
fn main() {
    println!("Chakra Go Plugin v{}", env!("CARGO_PKG_VERSION"));
    println!("This plugin is designed to be used with the Chakra WebAssembly runtime.");
    println!("Configuration is stored in Cargo.toml [package.metadata.chakra-plugin] section.");
    println!();
    println!("Install the CLI feature to use this binary standalone:");
    println!("  cargo install chakra-go --features cli");
    println!();
    println!("Or use with Chakra:");
    println!("  chakra plugin install chakra-go");
    println!("  chakra run ./my-go-project");
}
