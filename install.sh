#!/bin/bash

# DataCode Installation Script
# This script installs DataCode as a global command

set -e

echo "🧠 DataCode Installation Script"
echo "==============================="
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust/Cargo is not installed"
    echo "💡 Please install Rust first: https://rustup.rs/"
    exit 1
fi

echo "✅ Rust/Cargo found"

# Check if we're in the DataCode directory
if [ ! -f "Cargo.toml" ] || ! grep -q "name = \"data-code\"" Cargo.toml; then
    echo "❌ Error: Please run this script from the DataCode project directory"
    exit 1
fi

echo "✅ DataCode project directory confirmed"

# Build the project in release mode
echo ""
echo "🔨 Building DataCode in release mode..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Error: Failed to build DataCode"
    exit 1
fi

echo "✅ Build completed successfully"

# Install using cargo install
echo ""
echo "📦 Installing DataCode globally..."
cargo install --path . --force

if [ $? -ne 0 ]; then
    echo "❌ Error: Failed to install DataCode"
    exit 1
fi

echo "✅ DataCode installed successfully!"

# Check if cargo bin directory is in PATH
CARGO_BIN_DIR="$HOME/.cargo/bin"
if [[ ":$PATH:" != *":$CARGO_BIN_DIR:"* ]]; then
    echo ""
    echo "⚠️  Warning: Cargo bin directory is not in your PATH"
    echo "📝 Add this line to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
    echo "   export PATH=\"\$HOME/.cargo/bin:\$PATH\""
    echo ""
    echo "🔄 Or run this command now:"
    echo "   export PATH=\"\$HOME/.cargo/bin:\$PATH\""
    echo ""
    echo "🔄 Then reload your shell or run:"
    echo "   source ~/.bashrc  # or ~/.zshrc"
else
    echo "✅ Cargo bin directory is already in PATH"
fi

# Test the installation
echo ""
echo "🧪 Testing installation..."
if command -v datacode &> /dev/null; then
    echo "✅ DataCode command is available!"
    echo ""
    echo "🎉 Installation completed successfully!"
    echo ""
    echo "📚 Usage:"
    echo "  datacode                 # Start interactive REPL (default)"
    echo "  datacode filename.dc     # Execute DataCode file"
    echo "  datacode filename.dc --build_model  # Export tables to SQLite"
    echo "  datacode --websocket     # Start WebSocket server"
    echo "  datacode --websocket --host 0.0.0.0 --port 8899  # Custom host/port"
    echo "  datacode --websocket --use-ve  # Virtual environment mode"
    echo "  datacode --help          # Show help"
    echo ""
    echo "🚀 Try running: datacode --help"
else
    echo "⚠️  DataCode command not found in PATH"
    echo "💡 You may need to restart your terminal or update your PATH"
    echo ""
    echo "🔄 Try running:"
    echo "   export PATH=\"\$HOME/.cargo/bin:\$PATH\""
    echo "   datacode --help"
fi

echo ""
echo "✨ Happy coding with DataCode! ✨"
