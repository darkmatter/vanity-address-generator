# Quick Build Reference

## Prerequisites

- Rust toolchain: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Docker (for cross-compilation): [Install Docker](https://docs.docker.com/get-docker/)
- Cross (auto-installed by build tools): `cargo install cross --git https://github.com/cross-rs/cross`

## Quick Commands

```bash
# Current platform only
make release

# All platforms
make all-targets

# Specific platform
make linux          # Linux x86_64
make linux-arm      # Linux ARM64
make macos-intel    # macOS Intel
make macos-arm      # macOS Apple Silicon  
make windows        # Windows x86_64

# View all commands
make help
```

## Build Output

After running `make all-targets`, binaries are in `releases/`:

```
releases/
├── vanity-eth-x86_64-apple-darwin
├── vanity-eth-aarch64-apple-darwin
├── vanity-eth-x86_64-unknown-linux-gnu
├── vanity-eth-aarch64-unknown-linux-gnu
├── vanity-eth-x86_64-pc-windows-gnu.exe
└── checksums.txt
```

## Troubleshooting

### Cross not found
```bash
cargo install cross --git https://github.com/cross-rs/cross
```

### Docker not running
Start Docker Desktop or Docker daemon before building.

### Target fails to build
Some targets may not build without cross. Install Docker and cross.

### Permission denied
```bash
chmod +x build-all.sh
```

## Manual Cross Commands

```bash
# Install cross
cargo install cross --git https://github.com/cross-rs/cross

# Build for specific target
cross build --release --target x86_64-unknown-linux-gnu

# Binary location
ls -la target/x86_64-unknown-linux-gnu/release/vanity-eth
```

## CI/CD (GitHub Actions)

Push a tag to automatically build all targets:

```bash
git tag v1.0.0
git push origin v1.0.0
```

Artifacts will be available in GitHub Releases.
