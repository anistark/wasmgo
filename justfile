# wasmgo justfile

# Default
default:
    @just --list

# Build
build:
    cargo build --release

# Build with CLI feature
build-cli:
    cargo build --features cli

# Build CLI with release optimizations
build-cli-release:
    cargo build --release --features cli

# Run tests
test:
    cargo test

# Check code formatting
check:
    cargo check
    cargo fmt --check
    cargo clippy -- -D warnings

# Format code
format:
    cargo fmt

# Lint code
lint:
    cargo clippy -- -D warnings

# Clean build artifacts
clean:
    cargo clean

# Install the plugin locally (with CLI)
install:
    cargo install --path . --features cli

# Create example Go project for testing
create-example NAME:
    mkdir -p examples/{{NAME}}
    echo 'module {{NAME}}\n\ngo 1.21' > examples/{{NAME}}/go.mod
    echo 'package main\n\nimport "fmt"\n\nfunc main() {\n    fmt.Println("Hello from {{NAME}}!")\n}' > examples/{{NAME}}/main.go

# Test with example project
test-example NAME: build-cli
    ./target/debug/wasmgo check examples/{{NAME}}

# Run plugin info command
info: build-cli
    ./target/debug/wasmgo info

# Check dependencies
deps: build-cli
    ./target/debug/wasmgo deps

# Validate wasm-plugin configuration in Cargo.toml
validate-config: build-cli
    ./target/debug/wasmgo info
    @echo "✅ Wasm plugin configuration is valid"

# Package for release
package: build
    cargo package

# Publish to crates.io (dry run)
prepare-publish: build format lint
    cargo publish --dry-run

# Publish to crates.io
publish: prepare-publish
    @echo "Publishing to crates.io..."
    cargo publish

# Generate documentation
docs:
    cargo doc --open

# Development workflow - build, test, format, lint
dev: format lint test build-cli

# Watch for changes and rebuild
watch:
    cargo watch -x 'build --features cli'
