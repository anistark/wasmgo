# Wasm Go

🐹 Go WebAssembly plugin for [Wasmrun](https://github.com/anistark/wasmrun) - compile Go projects to WebAssembly using TinyGo.

[![Crates.io](https://img.shields.io/crates/v/wasmgo.svg)](https://crates.io/crates/wasmgo)
[![Documentation](https://docs.rs/wasmgo/badge.svg)](https://docs.rs/wasmgo)

## Installation

### As Wasm Plugin in Wasmrun
```sh
wasmrun plugin install wasmgo
wasmrun ./my-go-project
```

### Standalone CLI (Experimental)
```sh
cargo install wasmgo --features cli
wasmgo build ./my-go-project
```

## Requirements

- [Go](https://golang.org/) - The Go programming language
- [TinyGo](https://tinygo.org/) - Go compiler for WebAssembly

## Usage

### With Wasmrun
```sh
# Run with live reload
wasmrun ./my-go-project --watch

# Compile with optimization
wasmrun compile ./my-go-project --optimization size
```

### Standalone CLI (Only for Testing)
```sh
# Check project compatibility
wasmgo check ./my-go-project

# Build project
wasmgo build ./my-go-project

# Check dependencies
wasmgo deps --install
```

## Plugin Configuration

Plugin configuration is stored in `Cargo.toml` under the `[package.metadata.wasm-plugin]` section:

```toml
[package.metadata.wasm-plugin]
name = "go"
extensions = ["go"]
entry_files = ["go.mod", "main.go", "cmd/main.go", "app.go"]

[package.metadata.wasm-plugin.capabilities]
compile_wasm = true
compile_webapp = false
live_reload = true
optimization = true
custom_targets = ["wasm"]

[package.metadata.wasm-plugin.dependencies]
tools = ["tinygo", "go"]
```

This follows Rust ecosystem conventions for tool-specific metadata.

## Project Structure

Supports standard Go project layouts:

```sh
my-go-project/
├── go.mod
├── main.go
└── ...
```

Or with cmd directory:

```sh
my-go-project/
├── go.mod
├── cmd/
│   └── main.go
└── ...
```

## Example

```sh
mkdir hello-wasm && cd hello-wasm
go mod init hello-wasm

cat > main.go << 'EOF'
package main

import "fmt"

func main() {
    fmt.Println("Hello, WebAssembly from Go!")
}
EOF

wasmrun
```

## Development

```sh
# Build
just build

# Test with example
just create-example hello
just test-example hello

# Validate configuration
just validate-config

# Install locally
just install
```

## Plugin Architecture

This plugin implements the Wasm plugin interface and reads its configuration from `Cargo.toml`. The configuration includes:

- **Extensions**: File extensions the plugin handles
- **Entry Files**: Priority order for entry point detection
- **Capabilities**: What features the plugin supports
- **Dependencies**: Required external tools

#### [License](./LICENSE)
