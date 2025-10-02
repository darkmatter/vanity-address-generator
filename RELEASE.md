# Release Guide

This guide explains how to build and release the vanity address generator for multiple platforms.

## Quick Start

### Build for Current Platform

```bash
# Using Make
make release

# Using Cargo directly
cargo build --release
```

The binary will be at `target/release/vanity-eth` with checksum displayed.

### Build for All Platforms

```bash
# Using the build script
./build-all.sh

# Using Make
make all-targets
```

Binaries will be in the `releases/` directory with a `checksums.txt` file.

## Supported Platforms

| Platform            | Target Triple               | Notes                   |
| ------------------- | --------------------------- | ----------------------- |
| macOS Intel         | `x86_64-apple-darwin`       | Requires macOS to build |
| macOS Apple Silicon | `aarch64-apple-darwin`      | Requires macOS to build |
| Linux x86_64        | `x86_64-unknown-linux-gnu`  | Can cross-compile       |
| Linux ARM64         | `aarch64-unknown-linux-gnu` | Requires `cross`        |
| Windows x86_64      | `x86_64-pc-windows-gnu`     | Requires `cross`        |

## Build Methods

### Method 1: Build Script (Recommended)

The `build-all.sh` script automatically:
- Builds for all supported targets
- Generates SHA-256 checksums
- Organizes binaries in `releases/` directory

```bash
./build-all.sh
```

### Method 2: Makefile

Convenient shortcuts for common tasks:

```bash
make release        # Current platform
make all-targets    # All platforms
make macos-intel    # Specific platform
make checksums      # Generate checksums
```

### Method 3: Manual Cargo

Build for specific targets manually:

```bash
# Add target (first time only)
rustup target add x86_64-unknown-linux-gnu

# Build
cargo build --release --target x86_64-unknown-linux-gnu

# Generate checksum
shasum -a 256 target/x86_64-unknown-linux-gnu/release/vanity-eth
```

### Method 4: Using Cross

For cross-compilation (especially Linux ARM and Windows):

```bash
# Install cross (first time only)
cargo install cross

# Build
cross build --release --target aarch64-unknown-linux-gnu
cross build --release --target x86_64-pc-windows-gnu
```

## GitHub Actions (CI/CD)

The repository includes a GitHub Actions workflow (`.github/workflows/release.yml`) that:
- Automatically builds for all platforms
- Generates checksums
- Creates GitHub releases with binaries

### Trigger a Release

1. **Tag a version:**
   ```bash
   git tag -a v1.0.0 -m "Release v1.0.0"
   git push origin v1.0.0
   ```

2. **Or manually trigger:**
   - Go to Actions tab in GitHub
   - Select "Build and Release" workflow
   - Click "Run workflow"

3. **Download artifacts:**
   - From the Actions run, or
   - From the Releases page (if tagged)

## Platform-Specific Notes

### macOS

- Intel and ARM builds require macOS
- Can build both architectures on Apple Silicon Macs
- Use `lipo` to create universal binaries:
  ```bash
  lipo -create \
    target/x86_64-apple-darwin/release/vanity-eth \
    target/aarch64-apple-darwin/release/vanity-eth \
    -output vanity-eth-universal
  ```

### Linux

- x86_64 can be built on most platforms
- ARM64 requires `cross` or native ARM machine
- Binaries are statically linked with `gnu` toolchain

### Windows

- Requires `cross` for cross-compilation from macOS/Linux
- Produces `.exe` files
- Consider `x86_64-pc-windows-msvc` target for native Windows builds

## Verifying Builds

### Check Binary

```bash
# Verify it runs
./target/release/vanity-eth --help

# Check target architecture
file ./target/release/vanity-eth

# On macOS
otool -L ./target/release/vanity-eth

# On Linux
ldd ./target/release/vanity-eth
```

### Verify Checksum

```bash
# Generate checksum
shasum -a 256 target/release/vanity-eth

# Compare with documented checksum
# Should match the SHA-256 in README.md or checksums.txt
```

## Distribution

### Create Archive

```bash
# For Unix (macOS, Linux)
tar czf vanity-eth-v1.0.0-x86_64-apple-darwin.tar.gz \
  -C target/x86_64-apple-darwin/release vanity-eth

# For Windows
cd target/x86_64-pc-windows-gnu/release
zip vanity-eth-v1.0.0-x86_64-pc-windows-gnu.zip vanity-eth.exe
```

### Update README

After building, update the README.md with current checksums:

```bash
# Build release
make release

# Get checksum
shasum -a 256 target/release/vanity-eth

# Update README.md with the new checksum
```

## Troubleshooting

### Target Not Installed

```bash
rustup target add <target-triple>
```

### Cross-Compilation Fails

```bash
# Install cross
cargo install cross --git https://github.com/cross-rs/cross

# Use cross instead of cargo
cross build --release --target <target-triple>
```

### Linker Errors

- Install platform-specific toolchain
- On macOS: `xcode-select --install`
- On Linux: `sudo apt-get install build-essential`

### Permission Denied

```bash
chmod +x build-all.sh
```

## Release Checklist

- [ ] Update version in `Cargo.toml`
- [ ] Update `CHANGELOG.md` (if exists)
- [ ] Run tests: `cargo test`
- [ ] Build all targets: `./build-all.sh`
- [ ] Verify checksums
- [ ] Update README.md with checksums
- [ ] Commit changes
- [ ] Create and push tag: `git tag v1.0.0 && git push origin v1.0.0`
- [ ] Wait for GitHub Actions to complete
- [ ] Verify release artifacts on GitHub
- [ ] Test binaries on different platforms
