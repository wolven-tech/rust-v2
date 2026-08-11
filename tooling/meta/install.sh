#!/usr/bin/env bash
set -e

# Meta Orchestrator Installation Script
# This script builds and installs the meta CLI tool
# Can be run from anywhere - it will install from the current directory

echo "🎯 Installing Meta Orchestrator..."
echo ""

# Check for Rust/Cargo
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✅ Rust found: $(rustc --version)"
echo ""

# Check for Bacon (optional but recommended)
if ! command -v bacon &> /dev/null; then
    echo "⚠️  Bacon not found (optional for Rust hot-reload)"
    echo "   Install: cargo install bacon"
    echo ""
fi

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo "📂 Installing from: $SCRIPT_DIR"
echo ""

# Check if we're in a valid meta directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Cargo.toml not found in current directory"
    echo "   Make sure you're running this script from the meta directory"
    exit 1
fi

# Verify this is the meta package
if ! grep -q 'name = "meta"' Cargo.toml; then
    echo "❌ Error: This doesn't appear to be the meta package"
    exit 1
fi

# Build meta
echo "🔨 Building meta orchestrator..."
if cargo build --release; then
    echo "✅ Build successful!"
else
    echo "❌ Build failed"
    exit 1
fi

echo ""

# Install to cargo bin
echo "📦 Installing to ~/.cargo/bin..."
if cargo install --path .; then
    echo "✅ Installed successfully!"
else
    echo "❌ Installation failed"
    exit 1
fi

echo ""
echo "🎉 Meta orchestrator installed!"
echo ""
echo "Verify installation:"
echo "  meta --version"
echo ""
echo "Get started:"
echo "  cd your-monorepo/    # Navigate to your monorepo"
echo "  meta init            # Initialize meta.toml configuration"
echo "  meta doctor          # Validate your setup"
echo "  meta dev             # Start all dev servers with tmux"
echo ""
echo "📝 Note: Requires tmux for multi-process dev mode"
echo "  macOS:  brew install tmux"
echo "  Linux:  apt install tmux  or  yum install tmux"
echo ""
echo "For more information:"
echo "  meta --help"
echo ""
