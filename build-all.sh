#!/bin/bash

# Build script for multiple targets
# This script builds the vanity address generator for multiple platforms
# and generates SHA-256 checksums for each binary

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Building vanity-eth for multiple targets...${NC}\n"

# Define targets
TARGETS=(
    "x86_64-apple-darwin"           # macOS Intel
    "aarch64-apple-darwin"          # macOS Apple Silicon
    "x86_64-unknown-linux-gnu"      # Linux x86_64
    "aarch64-unknown-linux-gnu"     # Linux ARM64
    "x86_64-pc-windows-gnu"         # Windows x86_64
)

# Create releases directory
RELEASE_DIR="releases"
mkdir -p "$RELEASE_DIR"

# Check if cross is installed
if command -v cross &> /dev/null; then
    BUILD_CMD="cross"
    echo -e "${GREEN}Using 'cross' for cross-compilation${NC}\n"
else
    echo -e "${BLUE}Note: 'cross' not found, attempting to install...${NC}"
    if cargo install cross --git https://github.com/cross-rs/cross; then
        BUILD_CMD="cross"
        echo -e "${GREEN}Successfully installed 'cross'${NC}\n"
    else
        BUILD_CMD="cargo"
        echo -e "${GREEN}Using 'cargo' for compilation (some targets may fail)${NC}"
        echo -e "To enable full cross-compilation support, install Docker and run: cargo install cross\n"
    fi
fi

# Build for each target
for TARGET in "${TARGETS[@]}"; do
    echo -e "${BLUE}Building for $TARGET...${NC}"

    # Check if target is installed (only needed for cargo, cross handles this automatically)
    if [ "$BUILD_CMD" = "cargo" ]; then
        if ! rustup target list | grep -q "^$TARGET (installed)"; then
            echo "Installing target $TARGET..."
            rustup target add "$TARGET" || echo "Warning: Could not add target $TARGET"
        fi
    fi

    # Build
    if $BUILD_CMD build --release --target "$TARGET"; then
        # Determine binary name
        if [[ "$TARGET" == *"windows"* ]]; then
            BINARY_NAME="vanity-eth.exe"
        else
            BINARY_NAME="vanity-eth"
        fi

        SOURCE_PATH="target/$TARGET/release/$BINARY_NAME"
        DEST_NAME="vanity-eth-$TARGET"

        if [[ "$TARGET" == *"windows"* ]]; then
            DEST_NAME="$DEST_NAME.exe"
        fi

        DEST_PATH="$RELEASE_DIR/$DEST_NAME"

        # Copy binary to releases directory
        if [ -f "$SOURCE_PATH" ]; then
            cp "$SOURCE_PATH" "$DEST_PATH"
            echo -e "${GREEN}✓ Built: $DEST_NAME${NC}"

            # Generate checksum
            if command -v shasum &> /dev/null; then
                CHECKSUM=$(shasum -a 256 "$DEST_PATH" | cut -d' ' -f1)
            elif command -v sha256sum &> /dev/null; then
                CHECKSUM=$(sha256sum "$DEST_PATH" | cut -d' ' -f1)
            else
                CHECKSUM="N/A"
            fi

            echo "  SHA-256: $CHECKSUM"
            echo "$CHECKSUM  $DEST_NAME" >> "$RELEASE_DIR/checksums.txt"
        else
            echo "  Warning: Binary not found at $SOURCE_PATH"
        fi
    else
        echo -e "  Warning: Build failed for $TARGET"
    fi

    echo ""
done

echo -e "${GREEN}Build complete!${NC}"
echo -e "Binaries are in the '$RELEASE_DIR' directory"
echo -e "Checksums saved to '$RELEASE_DIR/checksums.txt'\n"

# Display checksums
if [ -f "$RELEASE_DIR/checksums.txt" ]; then
    echo -e "${BLUE}Checksums:${NC}"
    cat "$RELEASE_DIR/checksums.txt"
fi
