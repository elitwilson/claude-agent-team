#!/bin/bash
# Build and install the claude-agent-team binary globally.
# Safe to re-run — just rebuilds and overwrites.

set -e

REPO_DIR="$(cd "$(dirname "$0")" && pwd)"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="claude-bros"

echo "Building $BINARY_NAME..."
cargo build --release --manifest-path "$REPO_DIR/Cargo.toml"

echo "Installing to $INSTALL_DIR/$BINARY_NAME..."
cp "$REPO_DIR/target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"

echo "Done. Run: $BINARY_NAME"
