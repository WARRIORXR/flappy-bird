#!/usr/bin/env bash
# ==============================================================================
# Flappy Bird Pro - macOS Build Script
# Builds native Apple Silicon (arm64) or Intel (x86_64) macOS binary
# ==============================================================================
set -euo pipefail

echo "==========================================="
echo "  Flappy Bird Pro - Native macOS Builder   "
echo "==========================================="

# Check for Rust toolchain
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found! Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

echo "🦀 Building release binary with Cargo..."
cd "$(dirname "$0")/flappy-bird-rust"
cargo build --release

echo "📋 Copying executable to project root..."
cp target/release/flappy-bird-rust ../flappy-bird-macos
chmod +x ../flappy-bird-macos

echo "==========================================="
echo "✅ Build Complete!"
echo "Run the game using: ./flappy-bird-macos"
echo "==========================================="
