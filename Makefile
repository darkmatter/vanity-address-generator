.PHONY: help build release install clean test all-targets checksums

# Default target
help:
	@echo "Vanity Ethereum Address Generator - Build Commands"
	@echo ""
	@echo "Available targets:"
	@echo "  make build         - Build debug version for current platform"
	@echo "  make release       - Build optimized release for current platform"
	@echo "  make all-targets   - Build release for all platforms using cross"
	@echo "  make checksums     - Generate checksums for release binaries"
	@echo "  make install       - Install to ~/.cargo/bin"
	@echo "  make test          - Run tests"
	@echo "  make clean         - Clean build artifacts"
	@echo ""
	@echo "Platform-specific targets (using cross):"
	@echo "  make macos-intel   - Build for macOS Intel (x86_64-apple-darwin)"
	@echo "  make macos-arm     - Build for macOS Apple Silicon (aarch64-apple-darwin)"
	@echo "  make linux         - Build for Linux x86_64 (x86_64-unknown-linux-gnu)"
	@echo "  make linux-arm     - Build for Linux ARM64 (aarch64-unknown-linux-gnu)"
	@echo "  make windows       - Build for Windows x86_64 (x86_64-pc-windows-gnu)"
	@echo ""
	@echo "Note: 'cross' will be automatically installed if not present"

# Build debug version
build:
	cargo build

# Build release version
release:
	cargo build --release
	@echo ""
	@echo "Release binary built at: target/release/vanity-eth"
	@shasum -a 256 target/release/vanity-eth || sha256sum target/release/vanity-eth

# Build for all targets using cross
all-targets:
	export PATH=$(HOME)/.cargo/bin:$(PATH)
	@if ! command -v cross &> /dev/null; then \
		echo "Installing cross for cross-compilation..."; \
		cargo install cross --git https://github.com/cross-rs/cross; \
	fi
	@echo "Building for all targets using cross..."
	@cross build --release --target x86_64-apple-darwin
	@cross build --release --target aarch64-apple-darwin
	@cross build --release --target x86_64-unknown-linux-gnu
	@cross build --release --target aarch64-unknown-linux-gnu
	@cross build --release --target x86_64-pc-windows-gnu
	@echo ""
	@echo "Generating checksums..."
	@mkdir -p releases
	@for target in x86_64-apple-darwin aarch64-apple-darwin x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-pc-windows-gnu; do \
		if [ "$$target" = "x86_64-pc-windows-gnu" ]; then \
			binary="vanity-eth.exe"; \
			dest="releases/vanity-eth-$$target.exe"; \
		else \
			binary="vanity-eth"; \
			dest="releases/vanity-eth-$$target"; \
		fi; \
		if [ -f "target/$$target/release/$$binary" ]; then \
			cp "target/$$target/release/$$binary" "$$dest"; \
			shasum -a 256 "$$dest" | tee -a releases/checksums.txt; \
		fi; \
	done
	@echo ""
	@echo "All binaries built and checksums generated in releases/"

# Generate checksums for current release
checksums:
	@if [ -f target/release/vanity-eth ]; then \
		shasum -a 256 target/release/vanity-eth || sha256sum target/release/vanity-eth; \
	else \
		echo "No release binary found. Run 'make release' first."; \
	fi

# Install to cargo bin
install:
	cargo install --path .

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean
	rm -rf releases/

# Convenience targets for specific platforms (using cross)
macos-intel:
	cross build --release --target x86_64-apple-darwin

macos-arm:
	cross build --release --target aarch64-apple-darwin

linux:
	cross build --release --target x86_64-unknown-linux-gnu

linux-arm:
	cross build --release --target aarch64-unknown-linux-gnu

windows:
	cross build --release --target x86_64-pc-windows-gnu
