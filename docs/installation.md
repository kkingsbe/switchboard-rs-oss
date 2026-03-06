# Installation Guide

This guide covers how to install Switchboard on your system.

## Prerequisites

Before installing Switchboard, ensure you have the following:

### Required Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| **Docker** | 20.10+ | Container runtime for skill isolation |
| **Rust** | 1.70+ | Required for `cargo install` |
| **Cargo** | Latest | Comes with Rust toolchain |

### Verifying Prerequisites

```bash
# Check Docker is running
docker --version

# Check Rust is installed
rustc --version

# Check Cargo is available
cargo --version
```

## Installation Methods

### Method 1: Install from Source (Recommended)

```bash
cargo install --path .
```

This builds and installs the latest version from the local source code.

### Method 2: Install from Crates.io

```bash
cargo install switchboard
```

### Method 3: Build Manually

```bash
git clone https://github.com/switchboard-ai/switchboard.git
cd switchboard
cargo build --release
# Binary available at target/release/switchboard
```

## Verification

After installation, verify everything is working:

```bash
switchboard --version
```

You should see output similar to:
```
switchboard 0.1.0
```

## Quick Troubleshooting

### Issue 1: "command not found" after installation

**Cause:** The Cargo bin directory is not in your PATH.

**Solution:** Add Cargo's bin directory to your PATH:

```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$PATH:$HOME/.cargo/bin"

# Reload your shell
source ~/.bashrc  # or source ~/.zshrc
```

### Issue 2: Docker daemon not running

**Symptom:** `Error: Docker daemon is not running`

**Solution:** Start Docker Desktop or the Docker daemon:

```bash
# macOS
open -a Docker

# Linux
sudo systemctl start docker
sudo systemctl enable docker  # Auto-start on boot
```

### Issue 3: Compilation fails with missing dependencies

**Symptom:** `error: failed to run custom build command for openssl-sys`

**Solution:** Install development headers:

```bash
# Ubuntu/Debian
sudo apt-get install pkg-config libssl-dev

# macOS
brew install openssl

# Fedora/RHEL
sudo dnf install pkg-config openssl-devel
```

### Issue 4: Permission denied when running Docker

**Symptom:** `permission denied while trying to connect to the Docker daemon`

**Solution:** Add your user to the docker group:

```bash
sudo usermod -aG docker $USER
# Log out and back in for changes to take effect
```

### Issue 5: Old Rust version

**Symptom:** `error: package `switchboard` requires Rust version 1.70 or newer`

**Solution:** Update your Rust toolchain:

```bash
rustup update
rustup default stable
```

## Next Steps

- [Configuration](configuration.md) - Configure Switchboard for your needs
- [Troubleshooting](troubleshooting.md) - Detailed troubleshooting guide
