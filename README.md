# Ethereum Vanity Address Generator

A high-performance Ethereum vanity address generator written in Rust with BIP-39 mnemonic support and progress tracking.

## Features

- 🚀 **Fast parallel generation** using Rayon for multi-core processing
- 🔑 **BIP-39 mnemonic support** for deterministic key generation
- 📊 **Real-time progress tracking** with estimated completion time
- 🔤 **Multiple matching modes**: lowercase or EIP-55 checksum
- 🔐 **Secure key handling** with automatic memory zeroization
- 🌳 **HD wallet derivation** (BIP-32) with custom derivation paths

## Installation

```bash
cargo build --release
```

## Usage

### Basic Usage (Random Keys)

Generate a vanity address with a specific prefix:

```bash
# Find address starting with "dead"
./target/release/vanity-eth dead

# Find address starting with "c0ffee" with progress tracking
./target/release/vanity-eth c0ffee --progress
```

### With BIP-39 Mnemonic

Use a mnemonic phrase for deterministic key generation:

```bash
# Using a 12-word mnemonic
./target/release/vanity-eth dead \
  --mnemonic "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" \
  --progress

# With custom derivation path and passphrase
./target/release/vanity-eth c0ffee \
  --mnemonic "your twelve word mnemonic phrase goes here for deterministic generation" \
  --passphrase "optional-passphrase" \
  --derivation-path "m/44'/60'/0'/0" \
  --start-index 0
```

### Advanced Options

```bash
# Use checksum mode for case-sensitive matching (EIP-55)
./target/release/vanity-eth DeAd --mode checksum

# Specify number of threads
./target/release/vanity-eth beef --threads 8 --progress

# Adjust progress update interval
./target/release/vanity-eth 1337 --progress --progress-interval 10
```

## Command-Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `pattern` | Hex prefix to match (without 0x) | Required |
| `--mode` | Matching mode: `lower` or `checksum` | `lower` |
| `--threads` | Number of parallel workers | Number of CPUs |
| `--mnemonic` | BIP-39 mnemonic phrase | None (random keys) |
| `--passphrase` | BIP-39 passphrase | Empty string |
| `--derivation-path` | HD derivation path | `m/44'/60'/0'/0` |
| `--start-index` | Starting index for HD derivation | `0` |
| `--progress` | Show progress updates | `false` |
| `--progress-interval` | Progress update interval (seconds) | `5` |

## How It Works

### Random Mode
When no mnemonic is provided, the generator creates random private keys and checks if the resulting Ethereum address matches your desired prefix.

### Mnemonic Mode
When a mnemonic is provided:
1. Derives a seed from the mnemonic and optional passphrase
2. Uses BIP-32 HD derivation to generate keys at `derivation_path/index`
3. Increments the index for each attempt
4. Allows resuming from a specific index with `--start-index`

## Performance

The generator uses all available CPU cores by default and can process hundreds of thousands of addresses per second on modern hardware. The actual time to find a match depends on the prefix length:

- 1 character: ~16 attempts on average
- 2 characters: ~256 attempts
- 3 characters: ~4,096 attempts
- 4 characters: ~65,536 attempts
- 5 characters: ~1,048,576 attempts
- Each additional character multiplies difficulty by 16

## Security Notes

- Private keys are automatically zeroized in memory after use
- The generator uses cryptographically secure random number generation
- When using mnemonics, ensure they are generated securely
- Never share your private keys or mnemonic phrases

## Example Output

```
Using BIP-39 mnemonic with derivation path: m/44'/60'/0'/0
Progress: 145,234 attempts | Rate: 28,953 keys/sec | Est. time: 36 seconds

🎉 Found matching address!
  Address (lower):   0xdeadbeef123456789abcdef0123456789abcdef0
  Address (EIP-55):  0xDeAdBeEf123456789aBcDeF0123456789AbCdEf0
  Private key (hex): 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef
  Derivation index:  42
  Full path:         m/44'/60'/0'/0/42
  Matched lowercase prefix: dead

📊 Statistics:
  Total attempts:    165,432
  Time elapsed:      5.71 seconds
  Rate:              28,953 keys/sec
```

## License

MIT
