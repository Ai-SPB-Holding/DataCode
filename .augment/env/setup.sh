#!/bin/bash
set -e

# Update system packages
sudo apt-get update

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Source the cargo environment
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version

# Navigate to the workspace directory
cd /mnt/persist/workspace

echo "Rust installation verified. Running tests..."