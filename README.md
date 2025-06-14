# Chakra-Go

🐹 Go WebAssembly plugin for [Chakra](https://github.com/anistark/chakra) - compile Go projects to WebAssembly using TinyGo.

[![Crates.io](https://img.shields.io/crates/v/chakra-go.svg)](https://crates.io/crates/chakra-go)
[![Documentation](https://docs.rs/chakra-go/badge.svg)](https://docs.rs/chakra-go)

## Installation

### As Chakra Plugin
```sh
chakra plugin install chakra-go
chakra ./my-go-project
```

### Standalone CLI (Experimental)
```sh
cargo install chakra-go --features cli
chakra-go build ./my-go-project
```

## Requirements

- [Go](https://golang.org/) - The Go programming language
- [TinyGo](https://tinygo.org/) - Go compiler for WebAssembly

## Usage

### With Chakra
```sh
# Run with live reload
chakra ./my-go-project --watch

# Compile with optimization
chakra compile ./my-go-project --optimization size
```

### Standalone CLI (Only for Testing)
```sh
# Check project compatibility
chakra-go check ./my-go-project

# Build project
chakra-go build ./my-go-project

# Check dependencies
chakra-go deps --install
```

## Plugin Configuration

Plugin configuration is stored in `Cargo.toml` under the `[package.metadata.chakra-plugin]` section:

```toml
[package.metadata.chakra-plugin]
name = "go"
extensions = ["go"]
entry_files = ["go.mod", "main.go", "cmd/main.go", "app.go"]

[package.metadata.chakra-plugin.capabilities]
compile_wasm = true
compile_webapp = false
live_reload = true
optimization = true
custom_targets = ["wasm"]

[package.metadata.chakra-plugin.dependencies]
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

chakra
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

This plugin implements the Chakra plugin interface and reads its configuration from `Cargo.toml`. The configuration includes:

- **Extensions**: File extensions the plugin handles
- **Entry Files**: Priority order for entry point detection
- **Capabilities**: What features the plugin supports
- **Dependencies**: Required external tools

#### [License](./LICENSE)
