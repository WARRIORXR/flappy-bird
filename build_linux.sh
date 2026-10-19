#!/usr/bin/env bash
# ==============================================================================
# Flappy Bird Pro - Linux Build Script
# Builds native 64-bit Linux executable with full OpenGL / ALSA sound support
# ==============================================================================
set -euo pipefail

echo "==========================================="
echo "  Flappy Bird Pro - Native Linux Builder   "
echo "==========================================="

# Check for Rust toolchain
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found! Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Detect package manager and suggest/install graphics and audio libraries
if command -v apt-get &> /dev/null; then
    echo "📦 Ubuntu/Debian detected. Checking build dependencies..."
    echo "Installing required dev libraries (ALSA, X11, OpenGL)..."
    sudo apt-get update -qq || true
    sudo apt-get install -y --no-install-recommends \
        libasound2-dev \
        libx11-dev \
        libxi-dev \
        libgl1-mesa-dev \
        libxcursor-dev \
        libxrandr-dev \
        libxinerama-dev \
        build-essential \
        pkg-config
elif command -v dnf &> /dev/null; then
    echo "📦 Fedora/RHEL detected. Checking build dependencies..."
    sudo dnf install -y alsa-lib-devel libX11-devel libXi-devel mesa-libGL-devel
elif command -v pacman &> /dev/null; then
    echo "📦 Arch Linux detected. Checking build dependencies..."
    sudo pacman -S --needed --noconfirm alsa-lib libx11 libxi mesa
fi

echo "🦀 Building release binary with Cargo..."
cd "$(dirname "$0")/flappy-bird-rust"
cargo build --release

echo "📋 Copying executable to project root..."
cp target/release/flappy-bird-rust ../flappy-bird-linux
chmod +x ../flappy-bird-linux

echo "==========================================="
echo "✅ Build Complete!"
echo "Run the game using: ./flappy-bird-linux"
echo "==========================================="
