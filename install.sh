#!/bin/bash

# Installation script for Fuse
# This script builds and installs fuse to your system

set -e

# Check if Rust toolchain is configured
if ! rustc --version &> /dev/null; then
    echo "Rust toolchain not configured. Setting up default stable toolchain..."
    rustup default stable
fi

echo "Building Fuse..."
cargo build --release

if [ ! -f "target/release/fuse" ]; then
    echo "Error: Build failed or binary not found"
    exit 1
fi

# Try to install to system directory (requires sudo)
if command -v sudo &> /dev/null; then
    echo "Installing fuse to /usr/local/bin (requires sudo)..."
    sudo cp target/release/fuse /usr/local/bin/fuse
    sudo chmod +x /usr/local/bin/fuse
    echo "Fuse installed successfully to /usr/local/bin"
    echo "You can now run 'fuse' from anywhere in your terminal!"
else
    # Fallback to user's local bin directory
    LOCAL_BIN="$HOME/.local/bin"
    mkdir -p "$LOCAL_BIN"
    echo "Installing fuse to $LOCAL_BIN..."
    cp target/release/fuse "$LOCAL_BIN/fuse"
    chmod +x "$LOCAL_BIN/fuse"
    echo "Fuse installed successfully to $LOCAL_BIN"
    
    # Check if ~/.local/bin is in PATH
    if [[ ":$PATH:" != *":$LOCAL_BIN:"* ]]; then
        echo ""
        echo "Warning: $LOCAL_BIN is not in your PATH"
        echo "Add this line to your ~/.bashrc or ~/.zshrc:"
        echo "export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
        echo "Then run: source ~/.bashrc  (or source ~/.zshrc)"
    else
        echo "You can now run 'fuse' from anywhere in your terminal!"
    fi
fi
