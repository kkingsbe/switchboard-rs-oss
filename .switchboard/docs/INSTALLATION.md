# Installation Guide

This guide provides comprehensive instructions for installing Switchboard using different methods. Choose the installation method that best fits your needs and environment.

## Table of Contents

- [Installing from crates.io](#installing-from-cratesio)
- [Platform-Specific Requirements](#platform-specific-requirements)
- [Installing from Source](#installing-from-source)
- [Installing from Binaries](#installing-from-binaries)
- [Linux Requirements](#linux-requirements)
- [macOS Requirements](#macos-requirements)
- [System Requirements](#system-requirements)
- [Post-Installation Setup](#post-installation-setup)
- [Troubleshooting](#troubleshooting)

---

## Installing from crates.io

This is the recommended and simplest way to install Switchboard. The `cargo install` command downloads and compiles the latest published version from crates.io.

### Quick Installation

```bash
cargo install switchboard
```

This installs the latest published version of Switchboard from crates.io.

### Prerequisites

Before installing from crates.io, ensure you have:

- **Rust toolchain (1.70 or later)** - [Install Rust](https://rustup.rs/)
- **Cargo** (comes with Rust)
- **Docker Desktop running** (required for container execution)

#### Installing Rust

If you don't have Rust installed, use the official installer:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After installation, reload your shell:

```bash
source $HOME/.cargo/env
```

Verify your Rust version:

```bash
rustc --version
cargo --version
```

You should see Rust 1.70.0 or later.

#### Installing Docker

Docker is required to run agent containers. Install Docker Desktop for your platform:

- **macOS**: https://www.docker.com/products/docker-desktop
- **Linux**: https://docs.docker.com/engine/install/
- **Windows**: https://www.docker.com/products/docker-desktop

Ensure Docker is running before proceeding:

```bash
docker ps
```

## Platform-Specific Requirements

Before installing Switchboard, ensure your system meets the platform-specific requirements outlined below.

### Linux Requirements

#### Package Dependencies
- **Docker**: docker-ce or docker.io (20.10+)
  - The Docker daemon must be running and accessible
  - Docker container support is required for agent execution

#### System Requirements
- **Minimum RAM**: 512MB
- **Disk Space**: 100MB for switchboard binary + Docker overhead

> **Note:** The Docker daemon must be running before installing or using Switchboard. For detailed installation instructions and supported distributions, see [Linux Requirements](#linux-requirements) below.

### macOS Requirements

#### Docker
- **Docker Desktop for Mac** is required (Docker CLI alone is insufficient)
- **Minimum Version**: Docker Desktop 4.0+

#### System Requirements
- **macOS Version**: macOS 10.15 (Catalina) or later
- **Minimum RAM**: 4GB for Docker Desktop
- **Disk Space**: 100MB for switchboard binary + Docker Desktop overhead

> **Note:** Docker Desktop must be running and accessible before installing or using Switchboard. For detailed installation instructions, see [macOS Requirements](#macos-requirements) below.

### General System Requirements

#### Hardware
- **CPU**: Any modern 64-bit processor (x86_64 or ARM64)
- **RAM**: 512MB minimum (4GB recommended for Docker)
- **Disk Space**: 100MB for switchboard binary + Docker container storage requirements

#### Network
- **Internet connection** required for:
  - Downloading Docker images
  - Fetching dependencies during installation

---

### Version Selection

#### Install a specific version

```bash
cargo install switchboard --version 0.1.0
```

#### Install the latest version

```bash
cargo install switchboard
```

#### View available versions

Visit https://crates.io/crates/switchboard to see all published versions.

### Installation Location

By default, `cargo install` places the switchboard binary in:

- **Unix-like systems (Linux/macOS)**: `~/.cargo/bin/switchboard`
- **Windows**: `%USERPROFILE%\.cargo\bin\switchboard.exe`

Ensure this directory is in your PATH. See [PATH Configuration](#path-configuration) below.

### PATH Configuration

After installation, you may need to add the Cargo bin directory to your PATH if it's not already there.

#### Linux/macOS (Bash)

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

#### Linux/macOS (Zsh)

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

#### Windows (PowerShell)

```powershell
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$env:USERPROFILE\.cargo\bin", "User")
```

Restart your terminal after setting the PATH.

### Updating

To update to the latest version:

```bash
cargo install switchboard --force
```

The `--force` flag overwrites the previously installed binary with the new version.

### Verification

After installation, verify that switchboard is installed correctly:

```bash
switchboard --version
```

You should see output like:
```
switchboard 0.1.0
```

Test basic functionality:

```bash
switchboard validate switchboard.sample.toml
```

### Uninstalling

To remove switchboard:

```bash
cargo uninstall switchboard
```

This removes the binary from your Cargo bin directory. Cached build artifacts can be cleaned with:

```bash
cargo clean
```

### Troubleshooting

If you encounter issues during installation:

- **Command not found**: Ensure `~/.cargo/bin` is in your PATH
- **Permission denied**: Check permissions on `~/.cargo/` directory
- **Network errors**: Verify internet access and check firewall/proxy settings
- **Docker errors**: Ensure Docker daemon is running
- **Compilation errors**: Verify Rust version is 1.70.0 or later

For detailed troubleshooting, see [Installation Troubleshooting](./INSTALLATION_TROUBLESHOOTING.md).

### Network Requirements

- Internet access required to download from crates.io
- Cargo caches downloads locally, so subsequent installations are faster
- If behind a corporate firewall, you may need to configure a proxy in `~/.cargo/config.toml`:

```toml
[http]
proxy = "http://your-proxy-server:port"

[https]
proxy = "https://your-proxy-server:port"
```

---

## Installing from Source

If you prefer to build from source or need to modify the code, you can install Switchboard directly from the repository.

### Prerequisites

- **Rust toolchain (1.70 or later)**
- **Git**
- **Docker Desktop running**

### Installation Steps

1. **Clone the repository:**

```bash
git clone https://github.com/your-org/switchboard.git && cd switchboard
```

2. **Install from source:**

```bash
cargo install --path .
```

This installs the `switchboard` binary globally on your system. The `cargo install` command builds the project in release mode by default, which provides optimized binaries.

3. **Verify installation:**

```bash
switchboard --version
```

### Development Build

For development purposes, you can build a debug version instead:

```bash
cargo build
```

The debug binary will be located at `target/debug/switchboard`.

### Updating from Source

To update to the latest version from the repository:

```bash
git pull
cargo install --force --path .
```

### Uninstalling

```bash
cargo uninstall switchboard
```

---

## Installing from Binaries

Pre-compiled binaries for Linux, macOS, and Windows are available for download from the [GitHub Releases](https://github.com/your-org/switchboard/releases) page.

*Note: Binary downloads are available starting with version 0.1.0 and later.*

### Download and Install

#### Linux

```bash
# Download the latest binary
curl -L https://github.com/your-org/switchboard/releases/latest/download/switchboard-linux-x86_64 -o switchboard

# Make it executable
chmod +x switchboard

# Move to a directory in your PATH
sudo mv switchboard /usr/local/bin/
```

#### macOS

```bash
# Download the latest binary
curl -L https://github.com/your-org/switchboard/releases/latest/download/switchboard-darwin-x86_64 -o switchboard

# Make it executable
chmod +x switchboard

# Move to a directory in your PATH
sudo mv switchboard /usr/local/bin/
```

#### Windows (PowerShell)

```powershell
# Download the latest binary
Invoke-WebRequest -Uri "https://github.com/your-org/switchboard/releases/latest/download/switchboard-windows-x86_64.exe" -OutFile "switchboard.exe"

# Move to a directory in your PATH (e.g., create a folder and add to PATH)
mkdir -p $env:USERPROFILE\bin
Move-Item switchboard.exe $env:USERPROFILE\bin\
```

Then add `$env:USERPROFILE\bin` to your PATH in System Properties.

### Verification

After installation, verify the binary:

```bash
switchboard --version
```

### Uninstalling

#### Linux/macOS

```bash
sudo rm /usr/local/bin/switchboard
```

#### Windows

```powershell
Remove-Item $env:USERPROFILE\bin\switchboard.exe
```

---

## Linux Requirements

If you are installing on Linux, ensure the following requirements are met before proceeding with installation.

### Package Dependencies

#### Docker

**Docker is required** to run agent containers and is the primary runtime dependency for Switchboard.

- **Minimum Version:** Docker 20.10+
- **Docker Daemon:** Must be running and accessible
- **Socket Access:** Access to `/var/run/docker.sock` is required

Install Docker by distribution:

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install docker-ce docker-ce-cli containerd.io
```

**Fedora:**
```bash
sudo dnf install docker
```

**Arch Linux:**
```bash
sudo pacman -S docker
```

Start the Docker daemon:
```bash
sudo systemctl start docker
sudo systemctl enable docker  # Optional: start on boot
```

Verify Docker is running:
```bash
docker ps
```

#### User Permissions

The user running Switchboard must be in the `docker` group to execute Docker commands without sudo:

```bash
sudo usermod -aG docker $USER
```

Log out and log back in for the group change to take effect.

#### System Packages

The following system packages are required for Switchboard's Docker containers:

- `git` - For repository operations
- `curl` - For downloading dependencies and CLI tools
- `build-essential` - For compilation tasks (includes gcc, make, etc.)
- `procps` - For process monitoring (includes ps, kill commands)
- `file` - For file type detection
- `sudo` - For elevated operations within containers

These packages are typically installed by default. If missing, install via your distribution's package manager:

**Ubuntu/Debian:**
```bash
sudo apt-get install git curl build-essential procps file sudo
```

**Fedora:**
```bash
sudo dnf install git curl @development-tools procps-ng file sudo
```

**Arch Linux:**
```bash
sudo pacman -S git curl base-devel procps-ng file sudo
```

### Supported Distributions

Switchboard has been tested and verified on the following Linux distributions:

| Distribution | Version | Status | Notes |
|--------------|---------|--------|-------|
| **Debian** | GNU/Linux 12 (bookworm) | ✓ Tested | Primary test environment, Linux kernel 6.1+ |
| **Ubuntu** | Ubuntu 20.04+ | ✓ Expected | Based on Debian 12 compatibility |
| **Fedora** | Fedora 35+ | ✓ Expected | Compatible with standard Docker packages |
| **Arch Linux** | Rolling release | ✓ Expected | Compatible with standard Docker packages |

> **Note:** Other modern Linux distributions with Docker 20.10+ support should work, though they may not have been tested. For comprehensive testing results, see [Platform Compatibility](./PLATFORM_COMPATIBILITY.md).

---

## macOS Requirements

If you are installing on macOS, ensure the following requirements are met before proceeding with installation.

### Docker Desktop

**Docker Desktop is required** to run agent containers on macOS. Unlike Linux, macOS does not support Docker Engine directly and must use Docker Desktop.

- **Minimum Version:** Docker Desktop 4.0+
- **Status:** Must be installed and running before executing switchboard commands
- **Download:** Available from https://www.docker.com/products/docker-desktop

#### Installation

1. Download Docker Desktop for macOS from https://www.docker.com/products/docker-desktop
2. Open the downloaded `.dmg` file
3. Drag Docker to the Applications folder
4. Launch Docker Desktop from Applications

#### Verification

Docker Desktop must be running before proceeding:

```bash
docker info
```

Expected output: Docker system information (no errors)

#### Starting Docker Desktop

- Launch from Applications folder
- Or run: `open /Applications/Docker.app`
- Wait for the Docker menu bar icon to show "Docker Desktop is running"

### Supported Architectures

Switchboard is compatible with both macOS architectures:

| Architecture | Status | Notes |
|--------------|--------|-------|
| **x86_64** (Intel) | ✓ Code audit verified | Tested on Debian GNU/Linux 12; macOS testing pending |
| **aarch64** (Apple Silicon) | ✓ Code audit verified | M1/M2/M3 Macs supported; macOS testing pending |

Both architectures have been verified for compatibility through code audit. Actual testing on macOS hardware is pending. For comprehensive testing results and to contribute test results, see [Platform Compatibility](./PLATFORM_COMPATIBILITY.md) and [macOS Testing Procedure](./MACOS_TESTING_PROCEDURE.md).

---

## System Requirements

Switchboard has specific hardware and network requirements for optimal operation, particularly for running Docker containers with agent workloads.

### Minimum RAM

- **Minimum:** 2GB RAM
- **Recommended:** 4GB+ RAM for multiple concurrent agents

Each Docker container running the Kilo Code CLI consumes additional memory. The base Node.js 22 runtime and Kilo Code CLI dependencies require approximately 200-500MB per agent. Memory requirements increase with:

- Concurrent agent workloads
- Large workspace volumes mounted into containers
- Long-running compilation or analysis tasks

For production environments with multiple active agents, allocate at least 1GB of additional RAM per agent beyond the minimum 2GB system requirement.

### Minimum Disk Space

- **Installation:** 500MB free disk space minimum
- **Additional space requirements:**
  - Cargo build cache: 500MB-1GB during compilation
  - Docker images: ~500MB for the node:22-slim base image plus Kilo Code CLI dependencies
  - Agent logs: Variable based on agent activity, stored in `.switchboard/logs/`
  - Workspace volumes: Dependent on project size and mounted directories

For typical development workflows with multiple projects, plan for at least 2GB of additional disk space beyond the minimum installation requirement. Logs and Docker images accumulate over time, so periodic cleanup may be necessary in disk-constrained environments.

### Network Requirements

- **Internet connectivity required** for initial installation and operation:
  - Access to crates.io for cargo install
  - Access to npm registry for Kilo Code CLI dependencies
  - Access to Docker registry for pulling container images (if not cached)
  - Access to GitHub for agent repository operations (if required)

- **Corporate/proxy environments:**
  If operating behind a corporate firewall or proxy, configure proxy settings in `~/.cargo/config.toml`:

  ```toml
  [http]
  proxy = "http://your-proxy-server:port"

  [https]
  proxy = "https://your-proxy-server:port"
  ```

- **Docker registry access:**
  Docker must have network access to pull images. If your environment uses a private Docker registry, configure Docker to use it:

  ```bash
  docker login <your-registry-url>
  ```

Switchboard itself does not open any network ports; it communicates with Docker via the local Docker socket (`/var/run/docker.sock` on Linux, or the equivalent on macOS).

---

## Post-Installation Setup

After installing Switchboard, you need to configure it before running agents.

### Quick Start

1. **Create a configuration file:**

```bash
cd ~/my-project
```

Create `switchboard.toml` with a simple agent:

```toml
version = "0.1.0"

[[agent]]
name = "hello-world"
schedule = "0 */6 * * *"           # Every 6 hours
prompt = "Hello! Please review the README.md file and suggest improvements."
```

2. **Validate your configuration:**

```bash
switchboard validate
```

3. **Build the Docker image:**

```bash
switchboard build
```

4. **Run a single agent:**

```bash
switchboard run hello-world
```

5. **Start the scheduler:**

```bash
switchboard up
```

For detailed setup instructions, including API key configuration, model selection, and MCP server setup, see [Setup Guide](./setup.md).

---

## Troubleshooting

### Common Installation Issues

#### Rust/Cargo Not Found

```
bash: cargo: command not found
```

**Solution:** Install Rust using rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Wrong Rust Version

```
error: package `switchboard v0.1.0` cannot be built because it requires rustc 1.70.0 or newer
```

**Solution:** Update Rust:

```bash
rustup update stable
rustup default stable
```

#### Docker Daemon Not Running

```
Error: Cannot connect to the Docker daemon at unix:///var/run/docker.sock
```

**Solution:** Start Docker:

- **macOS:** Open Docker Desktop
- **Linux:** `sudo systemctl start docker`
- **Windows:** Start Docker Desktop

#### Binary Not Found After Installation

```
bash: switchboard: command not found
```

**Solution:** Add Cargo bin to your PATH (see [PATH Configuration](#path-configuration) above).

### Getting Help

If you encounter issues not covered here:

1. Check the [Installation Troubleshooting Guide](./INSTALLATION_TROUBLESHOOTING.md) for detailed solutions
2. Review the [Platform Compatibility](./PLATFORM_COMPATIBILITY.md) documentation for known platform-specific issues
3. Search existing [GitHub Issues](https://github.com/your-repo/switchboard/issues)
4. Create a new issue with:
   - Your OS and version
   - Rust/Cargo versions
   - Docker version
   - The full error message
   - Steps you've already tried

### Additional Resources

- [Setup Guide](./setup.md) - Configuration and setup instructions
- [Platform Compatibility](./PLATFORM_COMPATIBILITY.md) - Platform-specific requirements
- [README.md](../README.md) - Project overview and quick start
- [CRATES_IO_PUBLISHING.md](./CRATES_IO_PUBLISHING.md) - For developers publishing to crates.io
