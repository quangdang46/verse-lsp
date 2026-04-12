#!/bin/bash
# verse-lsp build script for Linux, Windows, macOS

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PLATFORM=$(uname -s)
ARCH=$(uname -m)
VERSION=${1:-latest}

echo "Building verse-lsp for $PLATFORM ($ARCH)"

case "$PLATFORM" in
    Linux*)
        echo "Detected: Linux"
        TARGET="x86_64-unknown-linux-gnu"
        ;;
    Darwin*)
        echo "Detected: macOS"
        if [ "$ARCH" = "arm64" ]; then
            TARGET="aarch64-apple-darwin"
        else
            TARGET="x86_64-apple-darwin"
        fi
        ;;
    MINGW*|CYGWIN*|MSYS*)
        echo "Detected: Windows"
        TARGET="x86_64-pc-windows-gnu"
        ;;
    *)
        echo -e "${RED}Unsupported platform: $PLATFORM${NC}"
        exit 1
        ;;
esac

echo "Target: $TARGET"

# Build
echo -e "${YELLOW}Building release...${NC}"
cargo build --release --target "$TARGET" 2>&1 || {
    echo -e "${RED}Build failed${NC}"
    exit 1
}

# Create output directory
OUT_DIR="dist/$PLATFORM-$ARCH"
mkdir -p "$OUT_DIR"

# Copy binary
BINARY_NAME="verse-lsp"
if [ "$PLATFORM" = "Windows" ]; then
    BINARY_NAME="verse-lsp.exe"
fi

cp "target/$TARGET/release/$BINARY_NAME" "$OUT_DIR/"

echo -e "${GREEN}Build complete!${NC}"
echo "Output: $OUT_DIR/$BINARY_NAME"
echo ""
echo "To install:"
echo "  Linux/macOS: sudo cp $OUT_DIR/$BINARY_NAME /usr/local/bin/"
echo "  Windows: copy $OUT_DIR\\$BINARY_NAME to your PATH"