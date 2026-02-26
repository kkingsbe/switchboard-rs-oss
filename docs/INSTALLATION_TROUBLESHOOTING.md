# Installation Troubleshooting

This guide helps you diagnose and fix common issues when installing switchboard.

## Quick Checklist

Before diving into specific issues, verify the following:

- [ ] Rust 1.70.0 or later is installed
- [ ] Cargo is available (installed with Rust)
- [ ] Docker is installed and the daemon is running
- [ ] Your user has permissions to use Docker
- [ ] You have at least 500MB free disk space
- [ ] Network access to crates.io (or your corporate registry)

## Common Issues

### Rust/Cargo Not Installed

**Symptom:**
```
bash: cargo: command not found
```

**Solution:**

Install Rust using rustup (recommended):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After installation, reload your shell:

```bash
source $HOME/.cargo/env
```

**Verify installation:**
```bash
rustc --version
cargo --version
```

**Note:** switchboard requires Rust 1.70.0 or later.

---

### Wrong Rust Version

**Symptom:**
```
error: package `switchboard v0.1.0` cannot be built because it requires rustc 1.70.0 or newer
```

**Solution:**

Update Rust using rustup:

```bash
rustup update stable
rustup default stable
```

---

### Docker Daemon Not Running

**Symptom:**
```
Error: error while loading shared libraries: libdocker.so: cannot open shared object file
```
OR
```
Error: Cannot connect to the Docker daemon at unix:///var/run/docker.sock
```

**Solution:**

**Linux:**
```bash
sudo systemctl start docker
sudo systemctl enable docker  # Optional: Start on boot
```

**macOS:**
- Open Docker Desktop
- Wait for the Docker icon to show "Docker Desktop is running"

**Windows (WSL):**
- Open Docker Desktop
- Go to Settings → Resources → WSL Integration
- Enable WSL 2 integration

**Verify Docker is running:**
```bash
docker ps
```

---

### Docker Permission Denied

**Symptom:**
```
permission denied while trying to connect to the Docker daemon socket
```

**Solution:**

Add your user to the docker group:

```bash
sudo usermod -aG docker $USER
```

Log out and log back in for changes to take effect, or run:

```bash
newgrp docker
```

**Verify permissions:**
```bash
docker ps  # Should work without sudo
```

---

### Binary Not Found After Installation

**Symptom:**
```
bash: switchboard: command not found
```

**Solution:**

The cargo binary directory may not be in your PATH. Check if the binary exists:

```bash
ls -la ~/.cargo/bin/switchboard
```

If it exists, add cargo bin to your PATH:

**Bash:**
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

**Zsh:**
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

**Verify installation:**
```bash
which switchboard
switchboard --version
```

---

### Cargo Install Permission Denied

**Symptom:**
```
error: could not write to /home/user/.cargo/registry/cache
```

**Solution:**

Check permissions on the cargo directory:

```bash
ls -la ~/.cargo/
```

If ownership is incorrect, fix it:

```bash
sudo chown -R $USER:$USER ~/.cargo/
```

---

### Network/Firewall Blocking Cargo

**Symptom:**
```
error: failed to get `switchboard` as a dependency of package
Caused by:
  failed to fetch source
```

**Solution:**

If you're behind a corporate firewall or proxy, configure cargo:

Create or edit `~/.cargo/config.toml`:

```toml
[http]
proxy = "http://your-proxy-server:port"

[https]
proxy = "https://your-proxy-server:port"
```

Replace `your-proxy-server` and `port` with your actual proxy details.

---

### Insufficient Disk Space

**Symptom:**
```
error: failed to compile `switchboard v0.1.0`
Caused by:
  out of free disk space
```

**Solution:**

Clean cargo cache to free up space:

```bash
cargo clean
```

Or remove the entire cargo cache:

```bash
rm -rf ~/.cargo/registry/cache ~/.cargo/git/db
```

Check available disk space:

```bash
df -h
```

You need at least 500MB free for installation.

---

### Compilation Errors

**Symptom:**
```
error: failed to compile `switchboard v0.1.0`
error[E0433]: failed to resolve: use of undeclared crate or module
```

**Solution:**

This is likely a bug in switchboard. Please report it on GitHub with:

1. Your OS and version
2. Rust version (`rustc --version`)
3. The full error output
4. Steps to reproduce

Workaround: Try installing from a different source or use a pre-built binary if available.

---

## Platform-Specific Issues

### Linux: Docker Installation

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

### macOS: Docker Desktop

1. Download Docker Desktop from https://www.docker.com/products/docker-desktop
2. Install the .dmg file
3. Launch Docker Desktop from Applications
4. Wait for initialization to complete

**Common macOS issue:** If Docker Desktop fails to start, try:
- Checking for macOS updates
- Resetting Docker Desktop factory settings
- Checking virtualization is enabled in System Settings

### Windows: WSL2 + Docker Desktop

1. Install WSL2:
```powershell
wsl --install
```

2. Download and install Docker Desktop for Windows
3. Enable WSL2 integration in Docker Desktop settings
4. Restart Docker Desktop

---

## Uninstalling Switchboard

To remove switchboard:

```bash
cargo uninstall switchboard
```

To remove any cached build artifacts:

```bash
cargo clean
```

---

## Updating Switchboard

To update to the latest version from crates.io:

```bash
cargo install --force switchboard
```

To update from a local git repository:

```bash
git pull
cargo install --force --path .
```

---

## Getting Help

If you're still having issues:

1. Check the [Platform Compatibility](PLATFORM_COMPATIBILITY.md) page for known issues
2. Search existing [GitHub Issues](https://github.com/your-repo/switchboard/issues)
3. Create a new issue with:
   - Your OS and version
   - Rust/Cargo versions
   - Docker version
   - The full error message
   - Steps you've already tried
