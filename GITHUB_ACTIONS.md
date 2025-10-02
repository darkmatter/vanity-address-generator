# GitHub Actions Workflows

This repository includes several GitHub Actions workflows for automated building, testing, and releasing.

## Workflows

### 1. **CI** (`.github/workflows/ci.yml`)

Runs on every push to `main` or `develop` branches and on pull requests.

**What it does:**
- Runs tests (`cargo test`)
- Runs clippy linter (`cargo clippy`)
- Checks code formatting (`cargo fmt`)
- Builds for macOS and Linux to ensure compilation works

**Triggered by:**
- Push to `main` or `develop`
- Pull requests
- Manual workflow dispatch

### 2. **Build and Release** (`.github/workflows/release.yml`)

Comprehensive build and release workflow that creates binaries for all platforms.

**What it does:**
- Builds for 5 platforms:
  - macOS Intel (x86_64-apple-darwin)
  - macOS Apple Silicon (aarch64-apple-darwin)
  - Linux x86_64 (x86_64-unknown-linux-gnu)
  - Linux ARM64 (aarch64-unknown-linux-gnu)
  - Windows x86_64 (x86_64-pc-windows-gnu)
- Creates compressed archives (.tar.gz for Unix, .zip for Windows)
- Generates SHA-256 checksums for each binary and archive
- Creates a master CHECKSUMS.txt file with all checksums
- Publishes release with all artifacts when a tag is pushed

**Triggered by:**
- Push of a version tag (e.g., `v1.0.0`)
- Push to `main` branch (build only, no release)
- Pull requests (build only, no release)
- Manual workflow dispatch

**Usage:**
```bash
# Create and push a release tag
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0

# GitHub Actions will automatically:
# 1. Build all targets
# 2. Generate checksums
# 3. Create a GitHub release
# 4. Upload all binaries and checksums
```

### 3. **Publish Checksums** (`.github/workflows/publish-checksums.yml`)

Publishes checksums documentation after a release.

**What it does:**
- Downloads CHECKSUMS.txt from the latest release
- Creates a formatted CHECKSUMS.md documentation file
- Generates a workflow summary with checksum preview
- Uploads checksums as artifacts

**Triggered by:**
- When a release is published
- Manual workflow dispatch

## Artifacts

Each workflow generates artifacts that you can download:

### CI Workflow
- No artifacts (just validation)

### Build and Release Workflow
- `vanity-eth-<target>` - Contains binary archive, checksums for each platform
- `checksums` - Contains the master CHECKSUMS.txt file

### Publish Checksums Workflow
- `checksums-<version>` - Contains CHECKSUMS.txt and CHECKSUMS.md

## Release Process

### Automatic Release (Recommended)

1. **Update version in Cargo.toml** (if applicable)
2. **Commit changes:**
   ```bash
   git add .
   git commit -m "Prepare release v1.0.0"
   git push origin main
   ```

3. **Create and push tag:**
   ```bash
   git tag -a v1.0.0 -m "Release version 1.0.0"
   git push origin v1.0.0
   ```

4. **Wait for GitHub Actions:**
   - Go to Actions tab
   - Wait for "Build and Release" workflow to complete (~10-15 minutes)
   - Check the Releases page for the new release

5. **Verify release:**
   - Download CHECKSUMS.txt
   - Download a binary for your platform
   - Verify checksum matches

### Manual Workflow Trigger

You can also manually trigger workflows:

1. Go to Actions tab in GitHub
2. Select the workflow (e.g., "Build and Release")
3. Click "Run workflow"
4. Select branch and click "Run workflow"

## Release Assets

When a release is created, the following assets are published:

```
vanity-eth-v1.0.0-x86_64-apple-darwin.tar.gz
vanity-eth-v1.0.0-x86_64-apple-darwin.tar.gz.sha256
vanity-eth-v1.0.0-aarch64-apple-darwin.tar.gz
vanity-eth-v1.0.0-aarch64-apple-darwin.tar.gz.sha256
vanity-eth-v1.0.0-x86_64-unknown-linux-gnu.tar.gz
vanity-eth-v1.0.0-x86_64-unknown-linux-gnu.tar.gz.sha256
vanity-eth-v1.0.0-aarch64-unknown-linux-gnu.tar.gz
vanity-eth-v1.0.0-aarch64-unknown-linux-gnu.tar.gz.sha256
vanity-eth-v1.0.0-x86_64-pc-windows-gnu.zip
vanity-eth-v1.0.0-x86_64-pc-windows-gnu.zip.sha256
CHECKSUMS.txt
vanity-eth.exe.sha256 (for Windows binary)
vanity-eth.sha256 (for each Unix binary)
```

## Verifying Checksums

Users can verify downloaded binaries:

```bash
# Download the release and checksums
curl -LO https://github.com/your-org/vanity-eth/releases/download/v1.0.0/vanity-eth-v1.0.0-x86_64-apple-darwin.tar.gz
curl -LO https://github.com/your-org/vanity-eth/releases/download/v1.0.0/CHECKSUMS.txt

# Extract
tar xzf vanity-eth-v1.0.0-x86_64-apple-darwin.tar.gz

# Verify checksum
shasum -a 256 vanity-eth
# Compare output with CHECKSUMS.txt
```

## Troubleshooting

### Workflow Fails

1. **Check the logs:**
   - Go to Actions tab
   - Click on the failed workflow run
   - Click on the failed job
   - Expand the failed step to see error details

2. **Common issues:**
   - **Cross compilation fails:** Ensure Docker is available on runners
   - **Release creation fails:** Check GitHub token permissions
   - **Checksum generation fails:** Ensure all binaries were built successfully

### Re-running a Failed Workflow

1. Go to the failed workflow run
2. Click "Re-run jobs" → "Re-run failed jobs"

### Deleting a Bad Release

1. Go to Releases page
2. Click on the release
3. Click "Delete"
4. Delete the tag: `git push origin :refs/tags/v1.0.0`
5. Fix the issue and create a new release

## Badges

Add these badges to your README.md:

```markdown
[![CI](https://github.com/your-org/vanity-eth/actions/workflows/ci.yml/badge.svg)](https://github.com/your-org/vanity-eth/actions/workflows/ci.yml)
[![Release](https://github.com/your-org/vanity-eth/actions/workflows/release.yml/badge.svg)](https://github.com/your-org/vanity-eth/actions/workflows/release.yml)
[![Latest Release](https://img.shields.io/github/v/release/your-org/vanity-eth)](https://github.com/your-org/vanity-eth/releases/latest)
```

## Security

- Workflows use pinned action versions for security
- GITHUB_TOKEN has minimal required permissions
- Checksums are generated and published for all artifacts
- All downloads should be verified against published checksums
