use rand::RngCore;
use bip39::{Language, Mnemonic};
use clap::{Parser, ValueEnum};
use coins_bip32::{path::DerivationPath, prelude::*, xkeys::Parent};
use hex::ToHex;
use k256::{elliptic_curve::sec1::ToEncodedPoint, PublicKey, SecretKey};
use rand::rngs::OsRng;
use rayon::prelude::*;
use sha3::{Digest, Keccak256};
use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use zeroize::Zeroize;

/// How to compare the vanity pattern.
#[derive(Copy, Clone, PartialEq, Eq, ValueEnum, Debug)]
enum Mode {
    /// Compare against lowercase hex (no checksum case rules).
    Lower,
    /// Compare against EIP-55 checksummed address string (case-sensitive).
    Checksum,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Ethereum vanity address generator in Rust")]
struct Args {
    /// Prefix to match (without 0x). Examples: "dead", "c0ffee". Optional if suffix is provided.
    #[arg(required_unless_present = "suffix")]
    pattern: Option<String>,

    /// Suffix to match (without 0x). Examples: "beef", "1337".
    #[arg(long, required_unless_present = "pattern")]
    suffix: Option<String>,

    /// Matching mode: lowercase or checksum (EIP-55 case-sensitive)
    #[arg(long, value_enum, default_value_t = Mode::Lower)]
    mode: Mode,

    /// Number of parallel workers (default: num CPUs)
    #[arg(long)]
    threads: Option<usize>,

    /// HD derivation path (default: m/44'/60'/0'/0 for Ethereum)
    #[arg(long, value_name = "PATH", default_value = "m/44'/60'/0'/0")]
    derivation_path: String,

    /// Number of addresses to check per mnemonic (default: 10)
    #[arg(long, default_value_t = 10)]
    addresses_per_mnemonic: u32,

    /// Show progress updates
    #[arg(long, short = 'p')]
    progress: bool,

    /// Progress update interval in seconds
    #[arg(long, default_value_t = 5)]
    progress_interval: u64,
}

fn main() {
    let args = Args::parse();

    // Normalize the target patterns according to selected mode
    let want_prefix = args.pattern.as_ref().map(|p| match args.mode {
        Mode::Lower => p.to_ascii_lowercase(),
        Mode::Checksum => p.clone(),
    });

    let want_suffix = args.suffix.as_ref().map(|s| match args.mode {
        Mode::Lower => s.to_ascii_lowercase(),
        Mode::Checksum => s.clone(),
    });

    if let Some(ref prefix) = want_prefix {
        if !is_valid_hex_prefix(prefix) {
            eprintln!("prefix pattern must be a valid hex string (0-9a-fA-F), no '0x'");
            std::process::exit(1);
        }
    }

    if let Some(ref suffix) = want_suffix {
        if !is_valid_hex_prefix(suffix) {
            eprintln!("suffix pattern must be a valid hex string (0-9a-fA-F), no '0x'");
            std::process::exit(1);
        }
    }

    // Parse derivation path
    let base_path = match DerivationPath::from_str(&args.derivation_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Invalid derivation path: {}", e);
            std::process::exit(1);
        }
    };

    println!("Searching for vanity address by generating random mnemonics...");
    if let Some(ref prefix) = want_prefix {
        println!("Prefix: {}", prefix);
    }
    if let Some(ref suffix) = want_suffix {
        println!("Suffix: {}", suffix);
    }
    println!("Checking first {} addresses per mnemonic", args.addresses_per_mnemonic);
    println!("Derivation path: {}", args.derivation_path);

    if let Some(t) = args.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(t)
            .build_global()
            .ok();
    }

    let found = Arc::new(AtomicBool::new(false));
    let attempts = Arc::new(AtomicU64::new(0));
    let start_time = Instant::now();

    // Progress tracking thread
    if args.progress {
        let found_clone = Arc::clone(&found);
        let attempts_clone = Arc::clone(&attempts);
        let interval = Duration::from_secs(args.progress_interval);
        let pattern_len = want_prefix.as_ref().map(|p| p.len()).unwrap_or(0) + want_suffix.as_ref().map(|s| s.len()).unwrap_or(0);
        let addresses_per_mnemonic = args.addresses_per_mnemonic;

        std::thread::spawn(move || {
            let mut last_attempts = 0u64;
            while !found_clone.load(Ordering::Relaxed) {
                std::thread::sleep(interval);
                let current_attempts = attempts_clone.load(Ordering::Relaxed);
                let elapsed = start_time.elapsed().as_secs();
                let rate = if elapsed > 0 {
                    (current_attempts - last_attempts) / args.progress_interval
                } else {
                    0
                };

                let mnemonics_checked = current_attempts / addresses_per_mnemonic as u64;

                // Estimate probability and time (adjusted for multiple addresses per mnemonic)
                let probability = 16_f64.powi(pattern_len as i32);
                let estimated_seconds = if rate > 0 {
                    probability / rate as f64
                } else {
                    f64::INFINITY
                };

                eprintln!(
                    "Progress: {} mnemonics ({} addresses) | Rate: {} addr/sec | Est. time: {}",
                    format_number(mnemonics_checked),
                    format_number(current_attempts),
                    format_number(rate),
                    format_duration(estimated_seconds)
                );
                last_attempts = current_attempts;
            }
        });
    }

    // Use an unbounded parallel iterator that keeps generating mnemonics until we find a match
    (0u64..u64::MAX)
        .into_par_iter()
        .any(|_| {
            if found.load(Ordering::Relaxed) {
                return true;
            }

            // Generate a random mnemonic
            let mut rng = OsRng;
            let mut entropy = [0u8; 16]; // 12 words = 16 bytes of entropy
            rng.fill_bytes(&mut entropy);
            let mnemonic = Mnemonic::from_entropy(&entropy).expect("failed to generate mnemonic");
            let seed = mnemonic.to_seed("");

            // Check the first N addresses derived from this mnemonic
            for index in 0..args.addresses_per_mnemonic {
                if found.load(Ordering::Relaxed) {
                    return true;
                }

                // Update attempts counter for each address checked
                attempts.fetch_add(1, Ordering::Relaxed);

                let (sk, addr_bytes, derivation_index) = gen_key_from_seed(&seed, &base_path, index);

                let addr_lower = hex::encode(addr_bytes);
                let checksummed = to_eip55(&addr_lower);

                let matches = match args.mode {
                    Mode::Lower => {
                        let prefix_match = want_prefix.as_ref().map_or(true, |p| addr_lower.starts_with(p));
                        let suffix_match = want_suffix.as_ref().map_or(true, |s| addr_lower.ends_with(s));
                        prefix_match && suffix_match
                    }
                    Mode::Checksum => {
                        let addr_checksum = checksummed.strip_prefix("0x").unwrap();
                        let prefix_match = want_prefix.as_ref().map_or(true, |p| addr_checksum.starts_with(p));
                        let suffix_match = want_suffix.as_ref().map_or(true, |s| addr_checksum.ends_with(s));
                        prefix_match && suffix_match
                    }
                };

                if matches {
                    let elapsed = start_time.elapsed();
                    let total_attempts = attempts.load(Ordering::Relaxed);

                    // Print result and stop everyone else
                    let sk_hex = sk.to_bytes().encode_hex::<String>();
                    // Zeroize secret key bytes after we've copied the hex out
                    let mut sk_bytes = sk.to_bytes();
                    sk_bytes.zeroize();

                    println!("\n🎉 Found matching address!");
                    println!("  Address (lower):   0x{}", addr_lower);
                    println!("  Address (EIP-55):  {}", checksummed);
                    println!("  Mnemonic:          {}", mnemonic.to_string());
                    println!("  Derivation index:  {}", derivation_index.unwrap());
                    println!("  Full path:         {}/{}", args.derivation_path, derivation_index.unwrap());
                    println!("  Private key (hex): 0x{}", sk_hex);

                    // Optional: show the matched prefix/suffix to sanity-check
                    match args.mode {
                        Mode::Lower => {
                            if let Some(ref prefix) = want_prefix {
                                println!(
                                    "  Matched lowercase prefix: {}",
                                    &addr_lower[..prefix.len().min(addr_lower.len())]
                                );
                            }
                            if let Some(ref suffix) = want_suffix {
                                let start = addr_lower.len().saturating_sub(suffix.len());
                                println!(
                                    "  Matched lowercase suffix: {}",
                                    &addr_lower[start..]
                                );
                            }
                        }
                        Mode::Checksum => {
                            let chk = checksummed.strip_prefix("0x").unwrap();
                            if let Some(ref prefix) = want_prefix {
                                println!(
                                    "  Matched checksum prefix: {}",
                                    &chk[..prefix.len().min(chk.len())]
                                );
                            }
                            if let Some(ref suffix) = want_suffix {
                                let start = chk.len().saturating_sub(suffix.len());
                                println!(
                                    "  Matched checksum suffix: {}",
                                    &chk[start..]
                                );
                            }
                        }
                    }

                    println!("\n📊 Statistics:");
                    println!("  Mnemonics checked: {}", format_number(total_attempts / args.addresses_per_mnemonic as u64));
                    println!("  Total addresses:   {}", format_number(total_attempts));
                    println!("  Time elapsed:      {:.2} seconds", elapsed.as_secs_f64());
                    println!("  Rate:              {} addr/sec", format_number((total_attempts as f64 / elapsed.as_secs_f64()) as u64));

                    found.store(true, Ordering::SeqCst);
                    return true;
                }
            }

            false
        });
}

/// Generate a key from BIP-39 seed with HD derivation
fn gen_key_from_seed(seed: &[u8; 64], base_path: &DerivationPath, index: u32) -> (SecretKey, [u8; 20], Option<u32>) {
    use hmac::Mac;
    use sha2::Sha512;

    // BIP-32 master key derivation
    type HmacSha512 = hmac::Hmac<Sha512>;
    let mut mac = HmacSha512::new_from_slice(b"Bitcoin seed").expect("valid key");
    mac.update(seed);
    let result = mac.finalize();
    let bytes = result.into_bytes();

    // Split into private key and chain code
    let (key_bytes, chain_code_bytes) = bytes.split_at(32);

    // Create the signing key for the master
    let master_key = k256::ecdsa::SigningKey::from_slice(key_bytes).expect("valid master key");

    // Convert chain code bytes to [u8; 32]
    let mut chain_code_array = [0u8; 32];
    chain_code_array.copy_from_slice(chain_code_bytes);
    let chain_code = ChainCode::from(chain_code_array);

    let master_xkey_info = XKeyInfo {
        depth: 0,
        parent: KeyFingerprint::from([0u8; 4]),
        index: 0,
        chain_code,
        hint: Hint::Legacy,
    };
    let master = XPriv::new(master_key, master_xkey_info);

    // Derive to base path then to specific index
    let derived = master
        .derive_path(base_path)
        .expect("valid derivation")
        .derive_child(index)
        .expect("valid child derivation");

    // Convert to k256 SecretKey
    // The XPriv contains a k256::ecdsa::SigningKey which we can access via AsRef
    let signing_key: &k256::ecdsa::SigningKey = derived.as_ref();
    let sk = SecretKey::from_slice(&signing_key.to_bytes()).expect("valid secret key");
    let pk = PublicKey::from_secret_scalar(&sk.to_nonzero_scalar());

    // Generate address (same as gen_key_and_address)
    let enc = pk.to_encoded_point(false);
    let uncompressed = enc.as_bytes();
    debug_assert_eq!(uncompressed[0], 0x04);

    let mut hasher = Keccak256::new();
    hasher.update(&uncompressed[1..]);
    let digest = hasher.finalize();

    let mut addr = [0u8; 20];
    addr.copy_from_slice(&digest[12..]);

    (sk, addr, Some(index))
}

/// EIP-55 checksum: given a lowercase hex address without 0x, produce "0x..." with mixed case.
fn to_eip55(addr_lower: &str) -> String {
    let addr_lower = addr_lower.to_ascii_lowercase();
    let mut hasher = Keccak256::new();
    hasher.update(addr_lower.as_bytes());
    let hash = hasher.finalize();

    let mut out = String::with_capacity(42);
    out.push_str("0x");

    for (i, c) in addr_lower.chars().enumerate() {
        if c.is_ascii_hexdigit() {
            // For letters a-f, uppercase if corresponding hash nibble >= 8
            if c.is_ascii_alphabetic() {
                // high nibble for even i, low nibble for odd i
                let byte = hash[i / 2];
                let nibble = if i % 2 == 0 { byte >> 4 } else { byte & 0x0f };
                if nibble >= 8 {
                    out.push(c.to_ascii_uppercase());
                } else {
                    out.push(c);
                }
            } else {
                out.push(c);
            }
        }
    }

    out
}

/// Validate that the pattern is a valid hex string (no 0x).
fn is_valid_hex_prefix(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Format large numbers with commas
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let mut count = 0;

    for c in s.chars().rev() {
        if count == 3 {
            result.push(',');
            count = 0;
        }
        result.push(c);
        count += 1;
    }

    result.chars().rev().collect()
}

/// Format duration in human-readable form
fn format_duration(seconds: f64) -> String {
    if seconds.is_infinite() {
        return "calculating...".to_string();
    }

    let secs = seconds as u64;
    if secs < 60 {
        format!("{} seconds", secs)
    } else if secs < 3600 {
        format!("{} minutes", secs / 60)
    } else if secs < 86400 {
        format!("{:.1} hours", seconds / 3600.0)
    } else if secs < 2592000 {
        format!("{:.1} days", seconds / 86400.0)
    } else if secs < 31536000 {
        format!("{:.1} months", seconds / 2592000.0)
    } else {
        format!("{:.1} years", seconds / 31536000.0)
    }
}
