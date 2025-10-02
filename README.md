# Ethereum Vanity Address Generator

A high-performance Ethereum vanity address generator written in Rust that searches through BIP-39 mnemonics to find matching addresses.

## Features

- 🚀 **Fast parallel generation** using Rayon for multi-core processing
- 🔑 **BIP-39 mnemonic generation** - generates random mnemonics and checks multiple addresses per mnemonic
- 📊 **Real-time progress tracking** with estimated completion time
- 🔤 **Multiple matching modes**: lowercase or EIP-55 checksum
- 🎯 **Prefix and/or suffix matching** - find addresses with specific beginnings and/or endings
- 🔐 **Secure key handling** with automatic memory zeroization
- 🌳 **HD wallet derivation** (BIP-32) with custom derivation paths

## Installation

Binaries are available but it is recommended to build from source to avoid phishing attacks

### From Source

```bash
cargo build --release
```

### Building for Multiple Targets

We provide build tools that use `cross` for seamless cross-compilation:

```bash
# Build for all platforms (installs cross automatically if needed)
make all-targets

# Or use the build script
./build-all.sh
```

This will create binaries for:
- macOS (Intel): `x86_64-apple-darwin`
- macOS (Apple Silicon): `aarch64-apple-darwin`
- Linux (x86_64): `x86_64-unknown-linux-gnu`
- Linux (ARM64): `aarch64-unknown-linux-gnu`
- Windows (x86_64): `x86_64-pc-windows-gnu`

All binaries and their checksums will be in the `releases/` directory.

**Note:** `cross` requires Docker to be installed. If you don't have Docker, the build script will fall back to `cargo` (some targets may not build).

### Manual Cross-Compilation

The easiest way to cross-compile is using `cross`:

```bash
# Install cross (first time only)
cargo install cross --git https://github.com/cross-rs/cross

# Build for any target
cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target aarch64-unknown-linux-gnu
cross build --release --target x86_64-pc-windows-gnu
```

Or for a specific target using Make:

```bash
make linux        # Linux x86_64
make linux-arm    # Linux ARM64
make macos-intel  # macOS Intel
make macos-arm    # macOS Apple Silicon
make windows      # Windows x86_64
```

### Binary Checksum

**Release Binary:** `target/release/vanity-eth`  
**SHA-256:** `35ec5dce77e89c1c11041da4b8992447912482510187cf1bb618de9e7002b632`

Verify the checksum:
```bash
shasum -a 256 target/release/vanity-eth
```

## Usage

### Basic Usage

Generate a vanity address by searching through random mnemonics:

```bash
# Find address starting with "dead"
./target/release/vanity-eth dead

# Find address starting with "c0ffee" with progress tracking
./target/release/vanity-eth c0ffee --progress
```

### Suffix Matching

Search for addresses with specific endings:

```bash
# Find address ending with "beef"
./target/release/vanity-eth --suffix beef --progress

# Find address with both prefix and suffix
./target/release/vanity-eth dead --suffix 1337 --progress
```

### Advanced Options

```bash
# Use checksum mode for case-sensitive matching (EIP-55)
./target/release/vanity-eth DeAd --mode checksum

# Check more addresses per mnemonic (default is 10)
./target/release/vanity-eth c0ffee --addresses-per-mnemonic 20 --progress

# Specify number of threads
./target/release/vanity-eth beef --threads 8 --progress

# Custom derivation path
./target/release/vanity-eth 1337 --derivation-path "m/44'/60'/1'/0" --progress

# Adjust progress update interval
./target/release/vanity-eth abc --progress --progress-interval 10
```

## Command-Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `pattern` | Hex prefix to match (without 0x) | Required unless `--suffix` provided |
| `--suffix` | Hex suffix to match (without 0x) | Required unless `pattern` provided |
| `--mode` | Matching mode: `lower` or `checksum` | `lower` |
| `--threads` | Number of parallel workers | Number of CPUs |
| `--derivation-path` | HD derivation path | `m/44'/60'/0'/0` |
| `--addresses-per-mnemonic` | Number of addresses to check per mnemonic | `10` |
| `--progress` | Show progress updates | `false` |
| `--progress-interval` | Progress update interval (seconds) | `5` |

## How It Works

The generator searches for vanity addresses by:
1. Generating random BIP-39 mnemonics (12 words)
2. For each mnemonic, deriving the first N addresses (default: 10) using HD derivation
3. Checking if any of these addresses match your prefix and/or suffix criteria
4. When a match is found, outputting the mnemonic, derivation index, and private key

This approach is more efficient than checking random keys because each mnemonic yields multiple addresses to check, effectively expanding the search space.

## Performance

The generator uses all available CPU cores by default and can process hundreds of thousands of addresses per second on modern hardware. By checking 10 addresses per mnemonic (default), you effectively 10x your search efficiency.

The actual time to find a match depends on the pattern length:

- 1 character: ~16 attempts on average
- 2 characters: ~256 attempts
- 3 characters: ~4,096 attempts
- 4 characters: ~65,536 attempts
- 5 characters: ~1,048,576 attempts
- Each additional character multiplies difficulty by 16

**Note:** When matching both prefix AND suffix, the difficulty is the product of both patterns (e.g., 3-char prefix + 3-char suffix = 4,096 × 4,096 = ~16.7 million attempts).

## Security Notes

- Private keys are automatically zeroized in memory after use
- The generator uses cryptographically secure random number generation
- When using mnemonics, ensure they are generated securely
- Never share your private keys or mnemonic phrases

## Example Output

```
Searching for vanity address by generating random mnemonics...
Prefix: dead
Checking first 10 addresses per mnemonic
Derivation path: m/44'/60'/0'/0
Progress: 14,523 mnemonics (145,230 addresses) | Rate: 28,953 addr/sec | Est. time: 36 seconds

🎉 Found matching address!
  Address (lower):   0xdeadbeef123456789abcdef0123456789abcdef0
  Address (EIP-55):  0xDeAdBeEf123456789aBcDeF0123456789AbCdEf0
  Mnemonic:          abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
  Derivation index:  4
  Full path:         m/44'/60'/0'/0/4
  Private key (hex): 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef
  Matched lowercase prefix: dead

📊 Statistics:
  Mnemonics checked: 16,543
  Total addresses:   165,430
  Time elapsed:      5.71 seconds
  Rate:              28,953 addr/sec
```

## License

MIT
