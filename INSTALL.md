# 📦 DataCode Installation Guide

This guide explains how to install DataCode as a global command on your system.

## 🚀 Quick Installation

### Automatic Installation (Recommended)

1. **Clone the repository:**
   ```bash
   git clone https://github.com/igornet0/DataCode.git
   cd DataCode
   ```

2. **Run the installation script:**
   ```bash
   ./install.sh
   ```
   
   Or using Make:
   ```bash
   make install
   ```

3. **Test the installation:**
   ```bash
   datacode --help
   ```

## 🔧 Manual Installation

### Prerequisites
- **Rust** (1.70 or later) - Install from [rustup.rs](https://rustup.rs/)
- **Git** - For cloning the repository

### Step-by-Step Installation

1. **Clone and build:**
   ```bash
   git clone https://github.com/igornet0/DataCode.git
   cd DataCode
   cargo build --release
   ```

2. **Install globally:**
   ```bash
   cargo install --path . --force
   ```

3. **Verify installation:**
   ```bash
   datacode --version
   ```

## 🎯 Usage After Installation

### Basic Commands
```bash
# Start interactive REPL
datacode

# Execute a DataCode file
datacode filename.dc

# Show help
datacode --help

# Run demonstration
datacode --demo
```

### Example Usage
```bash
# Create a simple DataCode program
echo "print('Hello, DataCode!')" > hello.dc

# Execute it
datacode hello.dc
```

## 🛠️ Development Setup

For development and testing:

```bash
# Clone repository
git clone https://github.com/igornet0/DataCode.git
cd DataCode

# Build in debug mode
make build

# Run tests
make test

# Start REPL in development mode
make run

# Run all examples
make examples
```

## 🗑️ Uninstallation

### Using the uninstall script:
```bash
./uninstall.sh
```

### Or using Make:
```bash
make uninstall
```

### Manual uninstallation:
```bash
cargo uninstall data_code
```

## 🔍 Troubleshooting

### Command not found: datacode

**Problem:** After installation, `datacode` command is not recognized.

**Solution:** Add Cargo's bin directory to your PATH:

```bash
# Add to your shell profile (~/.bashrc, ~/.zshrc, etc.)
export PATH="$HOME/.cargo/bin:$PATH"

# Reload your shell
source ~/.bashrc  # or ~/.zshrc
```

### Permission denied

**Problem:** Permission denied when running installation script.

**Solution:** Make the script executable:
```bash
chmod +x install.sh
./install.sh
```

### Build fails

**Problem:** Compilation errors during build.

**Solution:** 
1. Update Rust: `rustup update`
2. Clean and rebuild: `cargo clean && cargo build --release`
3. Check Rust version: `rustc --version` (should be 1.70+)

## 📁 File Locations

After installation:
- **Executable:** `~/.cargo/bin/datacode`
- **Source:** Your cloned directory
- **Examples:** `examples/` directory in source

## 🌍 Platform Support

DataCode is tested on:
- ✅ **macOS** (Intel & Apple Silicon)
- ✅ **Linux** (Ubuntu, Debian, CentOS, Arch)
- ✅ **Windows** (with WSL or native)

## 📚 Next Steps

After installation:

1. **Try the examples:**
   ```bash
   datacode examples/hello.dc
   datacode examples/functions.dc
   datacode examples/showcase.dc
   ```

2. **Start the interactive REPL:**
   ```bash
   datacode
   ```

3. **Read the documentation:**
   - [README.md](README.md) - Main documentation
   - [examples/](examples/) - Example programs
   - Type `help` in the REPL for syntax reference

## 🆘 Getting Help

- **In REPL:** Type `help` for syntax reference
- **Command line:** `datacode --help`
- **Issues:** [GitHub Issues](https://github.com/igornet0/DataCode/issues)
- **Documentation:** [README.md](README.md)

---

**Happy coding with DataCode! 🧠✨**
