# DataCode Makefile
# Convenient commands for building, testing, and installing DataCode

.PHONY: help build test run install uninstall clean dev release examples

# Default target
help:
	@echo "🧠 DataCode - Available Commands"
	@echo "================================"
	@echo ""
	@echo "Development:"
	@echo "  make build      - Build DataCode in debug mode"
	@echo "  make test       - Run all tests"
	@echo "  make run        - Start DataCode REPL"
	@echo "  make dev        - Build and run in development mode"
	@echo ""
	@echo "Release:"
	@echo "  make release    - Build DataCode in release mode"
	@echo "  make install    - Install DataCode as global command"
	@echo "  make uninstall  - Remove DataCode global command"
	@echo ""
	@echo "Examples:"
	@echo "  make examples   - Run all example files"
	@echo ""
	@echo "Maintenance:"
	@echo "  make clean      - Clean build artifacts"
	@echo ""
	@echo "Usage after installation:"
	@echo "  datacode                 # Start REPL"
	@echo "  datacode filename.dc     # Execute file"
	@echo "  datacode --help          # Show help"

# Build in debug mode
build:
	@echo "🔨 Building DataCode (debug mode)..."
	cargo build

# Build in release mode
release:
	@echo "🔨 Building DataCode (release mode)..."
	cargo build --release

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test

# Start REPL
run:
	@echo "🚀 Starting DataCode REPL..."
	cargo run

# Development mode (build + run)
dev: build run

# Install as global command
install:
	@echo "📦 Installing DataCode globally..."
	@chmod +x install.sh
	@./install.sh

# Uninstall global command
uninstall:
	@echo "🗑️  Uninstalling DataCode..."
	@chmod +x uninstall.sh
	@./uninstall.sh

# Run example files
examples:
	@echo "📚 Running DataCode examples..."
	@echo ""
	@echo "🔹 Running hello.dc:"
	@cargo run examples/hello.dc
	@echo ""
	@echo "🔹 Running functions.dc:"
	@cargo run examples/functions.dc
	@echo ""
	@echo "🔹 Running showcase.dc:"
	@cargo run examples/showcase.dc

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Check code formatting and linting
check:
	@echo "🔍 Checking code..."
	cargo check
	cargo clippy
	cargo fmt --check

# Format code
format:
	@echo "✨ Formatting code..."
	cargo fmt

# Show project info
info:
	@echo "🧠 DataCode Project Information"
	@echo "==============================="
	@echo "Name: DataCode"
	@echo "Version: $(shell grep '^version' Cargo.toml | cut -d'"' -f2)"
	@echo "Language: Rust"
	@echo "License: MIT"
	@echo ""
	@echo "📁 Project Structure:"
	@echo "  src/           - Source code"
	@echo "  examples/      - Example .dc files"
	@echo "  tests/         - Test files"
	@echo ""
	@echo "🔧 Available targets: build, test, run, install, examples"
