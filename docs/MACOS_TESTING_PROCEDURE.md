# macOS Testing Procedure for Switchboard

This document provides a comprehensive procedure for testing the switchboard CLI tool on macOS, covering both Intel (x86_64) and Apple Silicon (aarch64/ARM64) architectures.

## Overview

This testing procedure validates that switchboard can be successfully installed and operated on macOS systems through automated and manual verification steps.

**Target Platforms:**
- macOS x86_64 (Intel Macs)
- macOS aarch64 (Apple Silicon: M1, M2, M3)

**Test Script:** [`scripts/test-platform-compatibility.sh`](../scripts/test-platform-compatibility.sh)

**Related Documentation:**
- [`PLATFORM_COMPATIBILITY.md`](./PLATFORM_COMPATIBILITY.md) - Detailed platform requirements
- [`INSTALLATION.md`](./INSTALLATION.md) - General installation instructions

---

## Prerequisites

Before proceeding with testing, ensure the following requirements are met.

### Hardware Requirements

#### Minimum System Requirements
- **Processor:** Intel x86_64 OR Apple Silicon (M1/M2/M3)
- **RAM:** 4GB minimum, 8GB recommended for testing
- **Disk Space:** 3GB minimum free space for build artifacts and Docker images

#### Architecture-Specific Hardware

**For Intel Macs (x86_64):**
- Any Intel-based Mac running macOS 10.15 (Catalina) or later
- Test will run natively on x86_64 architecture

**For Apple Silicon Macs (aarch64):**
- Any M1, M2, or M3 Mac running macOS 11.0 (Big Sur) or later
- Test will run natively on aarch64 architecture
- Rosetta 2 installed if you plan to test x86_64 builds (optional):
  ```bash
  softwareupdate --install-rosetta
  ```

### Software Requirements

#### Operating System
- **Minimum:** macOS 10.15 (Catalina)
- **Recommended:** macOS 11.0 (Big Sur) or later
- **Verification:**
  ```bash
  sw_vers -productVersion
  ```

#### Docker Desktop
- **Minimum:** Docker Desktop 4.0+
- **Status:** Must be installed and running
- **Installation:** Download from https://www.docker.com/products/docker-desktop
- **Verification:**
  ```bash
  docker info
  ```

#### Xcode Command Line Tools
- **Status:** Required for Rust compilation
- **Installation:**
  ```bash
  xcode-select --install
  ```
- **Verification:**
  ```bash
  xcode-select -p
  ```
  Expected output: `/Applications/Xcode.app/Contents/Developer` or similar

#### Rust Toolchain
- **Minimum Version:** Rust 1.70.0 or later
- **Recommended:** Latest stable version
- **Installation Methods:**

  **Method 1: rustup (Recommended)**
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

  **Method 2: Homebrew (Alternative)**
  ```bash
  brew install rust
  ```

- **Verification:**
  ```bash
  rustc --version
  cargo --version
  ```

#### Shell and PATH Configuration
- **Supported Shells:** Zsh (default on macOS 10.15+), Bash
- **Requirement:** `~/.cargo/bin` must be in PATH
- **Verification:**
  ```bash
  which cargo
  ```
  Expected output: `/Users/your-username/.cargo/bin/cargo`

### Network Requirements
- Active internet connection for:
  - Downloading Rust toolchain (if not already installed)
  - Downloading Docker images
  - Fetching cargo dependencies
  - Accessing crates.io registry

### Account Permissions
- Standard user account with sudo access (for installing Xcode Command Line Tools)
- Write permissions to `$HOME/.cargo/bin/` directory
- Docker Desktop running with appropriate file sharing permissions

---

## Pre-Test Setup

Complete these setup steps before running the test script.

### Step 1: Verify macOS Version

Check your macOS version meets the minimum requirement:

```bash
sw_vers -productVersion
```

**Expected Output:**
- macOS 10.15.x (Catalina) or higher
- macOS 11.x.x (Big Sur) recommended
- macOS 12.x.x (Monterey) or later optimal

**Action:** If version is below 10.15, upgrade macOS before proceeding.

### Step 2: Detect System Architecture

Identify your Mac's architecture:

```bash
uname -m
```

**Expected Output:**
- `x86_64` for Intel Macs
- `arm64` for Apple Silicon Macs

**Note:** Record this value for test documentation.

### Step 3: Install Xcode Command Line Tools

If not already installed, install Xcode Command Line Tools:

```bash
xcode-select --install
```

A dialog will appear prompting installation. Click "Install" and wait for completion.

**Verify installation:**
```bash
xcode-select -p
```

**Expected Output:** `/Applications/Xcode.app/Contents/Developer` or similar path

### Step 4: Install Docker Desktop

If not already installed:

1. Download Docker Desktop for macOS from https://www.docker.com/products/docker-desktop
2. Open the downloaded `.dmg` file
3. Drag Docker to the Applications folder
4. Launch Docker Desktop from Applications
5. Wait for Docker Desktop to start (menu bar icon shows "Docker Desktop is running")

**Verify Docker Desktop is running:**
```bash
docker info
```

**Expected Output:** Docker system information (no errors)

### Step 5: Install Rust Toolchain

If not already installed, install Rust via rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the prompts (enter `1` for default installation).

**After installation, reload your shell:**
```bash
source $HOME/.cargo/env
```

**Verify Rust installation:**
```bash
rustc --version
cargo --version
```

**Expected Output:** Version numbers (e.g., `rustc 1.83.0`, `cargo 1.83.0`)

### Step 6: Configure PATH

Ensure Cargo bin directory is in your PATH.

**For Zsh (default on macOS 10.15+):**
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

**For Bash (older macOS versions):**
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bash_profile
source ~/.bash_profile
```

**Verify PATH configuration:**
```bash
which cargo
```

**Expected Output:** `/Users/your-username/.cargo/bin/cargo`

### Step 7: Verify File Permissions

Ensure you have write permissions to the Cargo bin directory:

```bash
ls -ld ~/.cargo/bin
```

**Expected Output:** Directory should be owned by your user

**If permission issues exist:**
```bash
sudo chown -R $USER ~/.cargo
```

### Step 8: Obtain Switchboard Source Code

If you haven't already, clone or download the switchboard repository:

```bash
cd ~
git clone <repository-url> switchboard
cd switchboard
```

**Alternatively, if you have the source archive:**
```bash
cd ~
# Extract and navigate to switchboard directory
```

### Step 9: Verify Test Script Exists

Ensure the test script is present:

```bash
ls -la scripts/test-platform-compatibility.sh
```

**Expected Output:** File should exist with execute permissions

**If execute permissions are missing:**
```bash
chmod +x scripts/test-platform-compatibility.sh
```

### Step 10: Verify Test Configuration File

Ensure a `switchboard.toml` configuration file exists in the project root:

```bash
ls -la switchboard.toml
```

**Expected Output:** Configuration file exists

**If missing, create from sample:**
```bash
cp switchboard.sample.toml switchboard.toml
```

---

## Testing Procedure

Execute the following steps to run the platform compatibility test.

### Step 1: Navigate to Switchboard Directory

Change to the switchboard project directory:

```bash
cd /path/to/switchboard
```

**Verify current directory:**
```bash
pwd
```

### Step 2: Verify Prerequisites Are Met

Run a quick verification of all prerequisites:

```bash
# Check macOS version
sw_vers -productVersion

# Check architecture
uname -m

# Check Docker
docker info > /dev/null 2>&1 && echo "Docker: OK" || echo "Docker: NOT RUNNING"

# Check Xcode tools
xcode-select -p > /dev/null 2>&1 && echo "Xcode: OK" || echo "Xcode: MISSING"

# Check Rust
rustc --version > /dev/null 2>&1 && echo "Rust: OK" || echo "Rust: MISSING"

# Check cargo in PATH
which cargo > /dev/null 2>&1 && echo "Cargo PATH: OK" || echo "Cargo PATH: NOT SET"

# Check test script
ls scripts/test-platform-compatibility.sh > /dev/null 2>&1 && echo "Test script: OK" || echo "Test script: MISSING"

# Check config file
ls switchboard.toml > /dev/null 2>&1 && echo "Config file: OK" || echo "Config file: MISSING"
```

**Action:** Resolve any failed checks before proceeding.

### Step 3: Run the Test Script

Execute the platform compatibility test script:

```bash
./scripts/test-platform-compatibility.sh
```

**Expected Behavior:**
- Script will display a "Platform Information" section
- Script will test `cargo install --path .`
- Script will verify binary installation
- Script will test `switchboard --version`
- Script will test `switchboard validate`
- Script will clean up the installed binary
- Script will display a "Test Summary" with results

### Step 4: Monitor Test Execution

Watch the test output for colored indicators:

- **Green (✓):** Test step passed
- **Red (✗):** Test step failed
- **Blue (ℹ):** Informational message

The script will execute the following sequence:

1. **Platform Detection and Logging**
   - Detects operating system (macOS)
   - Detects architecture (x86_64 or aarch64)
   - Logs OS version
   - Logs Rust and Cargo versions
   - Logs working directory

2. **Cargo Install Test**
   - Runs `cargo install --path . --force`
   - Compiles switchboard from local source
   - Installs binary to `~/.cargo/bin/switchboard`

3. **Binary Verification**
   - Verifies binary exists at `~/.cargo/bin/switchboard`
   - Verifies binary is executable

4. **Version Flag Test**
   - Runs `switchboard --version`
   - Verifies version output is displayed

5. **Validate Command Test**
   - Runs `switchboard validate`
   - Verifies configuration file validation works

6. **Cleanup**
   - Removes installed binary via `cargo uninstall switchboard`
   - Verifies cleanup was successful

7. **Test Summary**
   - Displays overall test results
   - Lists all passed tests
   - Confirms platform compatibility

### Step 5: Record Test Results

Document the test execution results:

**Test Information to Record:**
- Test date and time
- macOS version (`sw_vers -productVersion`)
- Architecture (`uname -m`)
- Rust version (`rustc --version`)
- Cargo version (`cargo --version`)
- Test script output
- Overall result (PASS/FAIL)

**Example Test Report:**

```markdown
## macOS Test Report

**Test Date:** 2026-02-16
**Tester:** Your Name

### Environment
- macOS Version: 14.2.1 (Sonoma)
- Architecture: arm64
- Rust Version: 1.83.0
- Cargo Version: 1.83.0
- Docker Desktop: 4.33.0

### Test Results
- Platform Detection: ✓ PASSED
- Cargo Install: ✓ PASSED
- Binary Verification: ✓ PASSED
- Version Flag: ✓ PASSED
- Validate Command: ✓ PASSED
- Cleanup: ✓ PASSED

### Overall Result: ✓ PASSED

### Notes
- Build time: ~4.5 seconds
- Binary size: ~500MB
- All functionality working as expected
```

### Step 6: Optional Manual Verification

For additional confidence, perform these manual verification steps:

**1. Verify Binary Installation:**
```bash
ls -lh ~/.cargo/bin/switchboard
```

**2. Test Version Flag:**
```bash
~/.cargo/bin/switchboard --version
```

**3. Test Help Flag:**
```bash
~/.cargo/bin/switchboard --help
```

**4. Test Validate Command:**
```bash
~/.cargo/bin/switchboard validate
```

**5. Test List Command:**
```bash
~/.cargo/bin/switchboard list
```

**6. Test Docker Integration:**
```bash
~/.cargo/bin/switchboard logs
```

---

## Expected Results

A successful test should produce the following output and results.

### Expected Script Output

When the test script runs successfully, you should see output similar to this:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Switchboard Platform Compatibility Test
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
This script tests cargo install and switchboard functionality
on the current platform: macOS arm64

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Platform Information
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Operating System: macOS
Architecture:      arm64
OS Version:        14.2.1
Rust Version:      rustc 1.83.0
Cargo Version:     cargo 1.83.0
Working Directory: /Users/username/switchboard
✓ Platform detection complete

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Testing cargo install
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ℹ Installing switchboard from local source...
[Build output...]
✓ cargo install completed successfully

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Verifying Binary Installation
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✓ Binary found at: /Users/username/.cargo/bin/switchboard
✓ Binary is executable

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Testing --version Flag
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ℹ Running: switchboard --version
switchboard 0.1.0
✓ switchboard --version works correctly

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Testing validate Command
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ℹ Running: switchboard validate
✓ switchboard validate works correctly

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Cleanup
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ℹ Removing installed binary: /Users/username/.cargo/bin/switchboard
✓ Binary removed successfully

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Test Summary
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✓ All platform compatibility tests passed!

Tests performed:
  ✓ Platform detection and logging
  ✓ cargo install --path .
  ✓ Binary installation verification
  ✓ switchboard --version
  ✓ switchboard validate
  ✓ Binary cleanup

✓ Platform: macOS arm64 is compatible!
```

### Expected Exit Code

- **Exit Code 0:** All tests passed
- **Exit Code 1:** One or more tests failed

Verify exit code:
```bash
./scripts/test-platform-compatibility.sh
echo $?
```

**Expected Output:** `0`

### Expected Build Times

Approximate build times on macOS:

| Architecture | Typical Build Time | Release Profile |
|--------------|-------------------|-----------------|
| Apple Silicon (aarch64) | 3-5 seconds | --release |
| Intel (x86_64) | 5-8 seconds | --release |

### Expected Binary Location

After successful installation:
- **Location:** `$HOME/.cargo/bin/switchboard`
- **Size:** Approximately 500MB
- **Permissions:** Executable (`-rwxr-xr-x`)

### Expected Functional Behavior

**Version Command:**
```bash
switchboard --version
```
**Expected Output:** `switchboard 0.1.0` (or current version)

**Validate Command:**
```bash
switchboard validate
```
**Expected Output:** Configuration validation message (no errors)

**Help Command:**
```bash
switchboard --help
```
**Expected Output:** Usage information with available commands

### Expected Platform Compatibility

The following should be verified as compatible:

| Platform | Architecture | Status |
|----------|--------------|--------|
| macOS 10.15+ | x86_64 (Intel) | ✓ Compatible |
| macOS 11.0+ | aarch64 (Apple Silicon) | ✓ Compatible |

---

## Troubleshooting

This section addresses common issues and their resolutions.

### Issue: "rustc not found!" or "cargo not found!"

**Symptom:** Test script reports "rustc not found!" or "cargo not found!"

**Cause:** Rust toolchain is not installed or not in PATH

**Resolution:**
1. Install Rust via rustup:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Reload shell environment:
   ```bash
   source $HOME/.cargo/env
   ```

3. Verify installation:
   ```bash
   rustc --version
   cargo --version
   ```

4. Check PATH:
   ```bash
   which cargo
   ```

5. If not in PATH, add to your shell config:
   - **Zsh:** Add to `~/.zshrc`
     ```bash
     echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
     source ~/.zshrc
     ```
   - **Bash:** Add to `~/.bash_profile`
     ```bash
     echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bash_profile
     source ~/.bash_profile
     ```

---

### Issue: "error: linker command failed" or "ld: library not found"

**Symptom:** Compilation fails with linker errors

**Cause:** Xcode Command Line Tools not installed

**Resolution:**
1. Install Xcode Command Line Tools:
   ```bash
   xcode-select --install
   ```

2. A dialog will appear - click "Install"

3. Wait for installation to complete

4. Verify installation:
   ```bash
   xcode-select -p
   ```
   Expected output: `/Applications/Xcode.app/Contents/Developer`

5. If the path is incorrect, reset it:
   ```bash
   sudo xcode-select --reset
   ```

---

### Issue: "Binary not found at: $HOME/.cargo/bin/switchboard"

**Symptom:** Test script reports binary not found after cargo install

**Cause:** Installation failed or binary installed to different location

**Resolution:**
1. Check cargo install directory:
   ```bash
   cargo install --root --print-config
   ```

2. Look for the binary in standard locations:
   ```bash
   find ~/.cargo -name switchboard -type f 2>/dev/null
   ```

3. If binary exists elsewhere, verify PATH includes that location

4. Try reinstalling:
   ```bash
   cargo install --path . --force --verbose
   ```

5. Check for permission errors:
   ```bash
   ls -ld ~/.cargo/bin
   ```

6. Fix permissions if needed:
   ```bash
   sudo chown -R $USER ~/.cargo
   ```

---

### Issue: "Binary exists but is not executable"

**Symptom:** Test script reports binary is not executable

**Cause:** Binary permissions incorrect

**Resolution:**
1. Check current permissions:
   ```bash
   ls -lh ~/.cargo/bin/switchboard
   ```

2. Make binary executable:
   ```bash
   chmod +x ~/.cargo/bin/switchboard
   ```

3. Verify executable:
   ```bash
   ls -lh ~/.cargo/bin/switchboard
   ```
   Should show executable permission (`x`)

---

### Issue: cargo install fails with "error: failed to compile"

**Symptom:** Compilation fails during cargo install

**Cause:** Various compilation errors (dependencies, syntax, platform-specific issues)

**Resolution:**
1. Run with verbose output to see details:
   ```bash
   cargo install --path . --force --verbose
   ```

2. Check Rust version compatibility:
   ```bash
   rustc --version
   ```
   Minimum required: Rust 1.70.0

3. Update Rust if needed:
   ```bash
   rustup update stable
   ```

4. Clean build cache:
   ```bash
   cargo clean
   ```

5. Try building directly:
   ```bash
   cargo build --release --verbose
   ```

6. Check for platform-specific errors:
   - If compilation fails at specific dependency, note the dependency name and error
   - Check dependency's platform support

7. Verify Docker Desktop is running (for some dependencies):
   ```bash
   docker info
   ```

---

### Issue: "switchboard validate failed!"

**Symptom:** Validate command test fails

**Cause:** Configuration file missing or invalid

**Resolution:**
1. Check if config file exists:
   ```bash
   ls -la switchboard.toml
   ```

2. If missing, create from sample:
   ```bash
   cp switchboard.sample.toml switchboard.toml
   ```

3. Run validate with verbose output:
   ```bash
   switchboard validate --verbose
   ```

4. Check configuration file syntax:
   ```bash
   cat switchboard.toml
   ```

5. Ensure you're in the project root directory:
   ```bash
   pwd
   ls switchboard.toml
   ```

---

### Issue: Docker-related errors

**Symptom:** Docker commands fail or container errors

**Cause:** Docker Desktop not running or permissions issues

**Resolution:**
1. Verify Docker Desktop is running:
   ```bash
   docker info
   ```

2. Start Docker Desktop if not running:
   ```bash
   open /Applications/Docker.app
   ```

3. Check Docker Desktop status in menu bar

4. Verify Docker Desktop file sharing permissions:
   - Open Docker Desktop
   - Go to Settings → Resources → File sharing
   - Ensure project directories are allowed

5. Restart Docker Desktop:
   - Click Docker menu bar icon
   - Select "Restart Docker Desktop"

---

### Issue: Permission denied errors

**Symptom:** Permission denied when installing or running

**Cause:** Insufficient permissions on ~/.cargo directory

**Resolution:**
1. Check directory ownership:
   ```bash
   ls -ld ~/.cargo
   ls -ld ~/.cargo/bin
   ```

2. Fix ownership:
   ```bash
   sudo chown -R $USER ~/.cargo
   ```

3. **IMPORTANT:** Do not use sudo with cargo install due to security concerns

4. Verify permissions:
   ```bash
   ls -ld ~/.cargo/bin
   ```

---

### Issue: Test script not found or not executable

**Symptom:** Cannot execute test script

**Cause:** Script missing or lacks execute permissions

**Resolution:**
1. Verify script exists:
   ```bash
   ls -la scripts/test-platform-compatibility.sh
   ```

2. Add execute permissions:
   ```bash
   chmod +x scripts/test-platform-compatibility.sh
   ```

3. Verify permissions:
   ```bash
   ls -la scripts/test-platform-compatibility.sh
   ```
   Should show executable permission (`x`)

---

### Issue: Network connectivity issues

**Symptom:** Failures to download dependencies or Docker images

**Cause:** Network connectivity or firewall issues

**Resolution:**
1. Test network connectivity:
   ```bash
   ping -c 3 crates.io
   ping -c 3 github.com
   ```

2. Check if behind a proxy:
   ```bash
   echo $HTTP_PROXY
   echo $HTTPS_PROXY
   ```

3. If using proxy, configure git and cargo:
   ```bash
   git config --global http.proxy $HTTP_PROXY
   git config --global https.proxy $HTTPS_PROXY
   ```

4. Check firewall settings
5. Try using VPN if necessary
6. Verify DNS resolution:
   ```bash
   nslookup crates.io
   ```

---

### Issue: Rosetta 2 issues on Apple Silicon

**Symptom:** Trying to run x86_64 builds on Apple Silicon

**Cause:** Rosetta 2 not installed

**Resolution:**
1. Install Rosetta 2:
   ```bash
   softwareupdate --install-rosetta
   ```

2. Accept license agreement if prompted

3. **Note:** Native aarch64 builds are recommended for Apple Silicon
   - The test script automatically detects architecture
   - Native builds perform better than translated x86_64

---

### Issue: Slow build times

**Symptom:** Compilation takes much longer than expected

**Cause:** Insufficient resources or first-time compilation

**Resolution:**
1. Check available RAM:
   ```bash
   sysctl hw.memsize
   ```

2. Close unnecessary applications

3. Ensure sufficient disk space:
   ```bash
   df -h ~
   ```

4. First-time compilation is always slower due to dependency building

5. Subsequent compilations will be faster

6. Expected build times:
   - Apple Silicon: 3-5 seconds
   - Intel: 5-8 seconds

---

### Issue: Script exits with error but no clear message

**Symptom:** Test script exits with code 1 but output is unclear

**Cause:** Script may be in strict mode (`set -euo pipefail`)

**Resolution:**
1. Run script with bash debugging:
   ```bash
   bash -x ./scripts/test-platform-compatibility.sh
   ```

2. This shows each command as it executes

3. Look for the last command before failure

4. Check script output for colored error messages (✗)

5. Review the specific test section that failed

---

## Test Checklist

Use this checklist to track completion of each testing step. Print or copy this section for manual testing.

### Pre-Test Setup Checklist

#### Environment Verification
- [ ] macOS version is 10.15 (Catalina) or later
  - Command: `sw_vers -productVersion`
  - Verified version: \_\_\_\_\_\_\_\_\_\_\_

- [ ] System architecture identified (x86_64 or arm64)
  - Command: `uname -m`
  - Architecture: \_\_\_\_\_\_\_\_\_\_\_

#### Docker Setup
- [ ] Docker Desktop installed (version 4.0+)
  - Installed version: \_\_\_\_\_\_\_\_\_\_\_

- [ ] Docker Desktop is currently running
  - Verified with: `docker info`

- [ ] Docker Desktop file sharing configured
  - Project directory accessible: [ ] Yes [ ] No

#### Xcode Tools
- [ ] Xcode Command Line Tools installed
  - Command: `xcode-select -p`
  - Path: \_\_\_\_\_\_\_\_\_\_\_

#### Rust Toolchain
- [ ] Rust installed (version 1.70.0+)
  - Command: `rustc --version`
  - Version: \_\_\_\_\_\_\_\_\_\_\_

- [ ] Cargo installed
  - Command: `cargo --version`
  - Version: \_\_\_\_\_\_\_\_\_\_\_

#### PATH Configuration
- [ ] ~/.cargo/bin is in PATH
  - Command: `which cargo`
  - Location: \_\_\_\_\_\_\_\_\_\_\_

#### File Permissions
- [ ] Write permissions on ~/.cargo/bin
  - Command: `ls -ld ~/.cargo/bin`
  - Owner: \_\_\_\_\_\_\_\_\_\_\_

#### Source Code
- [ ] switchboard source code obtained
  - Location: \_\_\_\_\_\_\_\_\_\_\_

- [ ] Test script exists and is executable
  - Command: `ls -la scripts/test-platform-compatibility.sh`

- [ ] Configuration file exists
  - Command: `ls -la switchboard.toml`

---

### Test Execution Checklist

#### Script Execution
- [ ] Navigated to switchboard directory
  - Working directory: \_\_\_\_\_\_\_\_\_\_\_

- [ ] All prerequisites verified
  - All checks passed: [ ] Yes [ ] No

- [ ] Test script executed
  - Command: `./scripts/test-platform-compatibility.sh`

---

### Test Results Verification

#### Individual Test Steps
- [ ] Platform detection test passed
  - OS detected: \_\_\_\_\_\_\_\_\_\_\_
  - Architecture detected: \_\_\_\_\_\_\_\_\_\_\_

- [ ] Cargo install test passed
  - Build time: \_\_\_\_\_\_\_\_\_\_\_ seconds

- [ ] Binary verification test passed
  - Binary location: \_\_\_\_\_\_\_\_\_\_\_

- [ ] Version flag test passed
  - Version output: \_\_\_\_\_\_\_\_\_\_\_

- [ ] Validate command test passed
  - Configuration validated: [ ] Yes [ ] No

- [ ] Cleanup test passed
  - Binary removed: [ ] Yes [ ] No

#### Overall Result
- [ ] All tests passed
  - Exit code: \_\_\_\_\_

- [ ] Platform confirmed compatible
  - Platform: \_\_\_\_\_\_\_\_\_\_\_

---

### Manual Verification Checklist (Optional)

#### Post-Test Verification
- [ ] Binary location verified
  - Command: `ls -lh ~/.cargo/bin/switchboard`

- [ ] Version command tested
  - Command: `~/.cargo/bin/switchboard --version`

- [ ] Help command tested
  - Command: `~/.cargo/bin/switchboard --help`

- [ ] Validate command tested
  - Command: `~/.cargo/bin/switchboard validate`

- [ ] List command tested
  - Command: `~/.cargo/bin/switchboard list`

- [ ] Docker integration verified
  - Command: `~/.cargo/bin/switchboard logs`

---

### Test Documentation Checklist

#### Test Report
- [ ] Test date and time recorded
  - Date/Time: \_\_\_\_\_\_\_\_\_\_\_

- [ ] Tester name recorded
  - Tester: \_\_\_\_\_\_\_\_\_\_\_

- [ ] Environment details documented
  - macOS version: \_\_\_\_\_\_\_\_\_\_\_
  - Architecture: \_\_\_\_\_\_\_\_\_\_\_
  - Rust version: \_\_\_\_\_\_\_\_\_\_\_
  - Cargo version: \_\_\_\_\_\_\_\_\_\_\_
  - Docker Desktop version: \_\_\_\_\_\_\_\_\_\_\_

- [ ] Test results documented
  - Overall result: [ ] PASS [ ] FAIL

- [ ] Any issues or anomalies noted
  - Notes: \_\_\_\_\_\_\_\_\_\_\_

- [ ] Performance metrics recorded
  - Build time: \_\_\_\_\_\_\_\_\_\_\_ seconds
  - Binary size: \_\_\_\_\_\_\_\_\_\_\_ MB

---

### Completion Checklist

#### Post-Test Cleanup
- [ ] Binary removed by script
  - Verify: `ls ~/.cargo/bin/switchboard` (should fail)

- [ ] No leftover artifacts
  - Verify: `.switchboard` directory clean if applicable

#### Results Submission
- [ ] Test report saved
  - Location: \_\_\_\_\_\_\_\_\_\_\_

- [ ] Test results communicated
  - Recipient: \_\_\_\_\_\_\_\_\_\_\_

---

## Additional Resources

### Documentation Links
- [`PLATFORM_COMPATIBILITY.md`](./PLATFORM_COMPATIBILITY.md) - Detailed platform requirements
- [`INSTALLATION.md`](./INSTALLATION.md) - General installation instructions
- [`INSTALLATION_TROUBLESHOOTING.md`](./INSTALLATION_TROUBLESHOOTING.md) - Installation troubleshooting guide
- [`troubleshooting.md`](./troubleshooting.md) - General troubleshooting guide

### Command References
- Rust installation: https://rustup.rs/
- Docker Desktop for Mac: https://www.docker.com/products/docker-desktop
- Cargo documentation: https://doc.rust-lang.org/cargo/

### Reporting Issues

If you encounter issues not covered in this troubleshooting section:

1. Document the error message
2. Record your environment details (macOS version, architecture, Rust version)
3. Note the step where the error occurred
4. Run the test script with debug output:
   ```bash
   bash -x ./scripts/test-platform-compatibility.sh
   ```
5. Collect the complete output
6. Report the issue through the project's issue tracker

---

## Appendix A: Quick Reference Commands

### Environment Detection
```bash
# Check macOS version
sw_vers -productVersion

# Check architecture
uname -m

# Check all macOS version details
sw_vers
```

### Docker Verification
```bash
# Check Docker info
docker info

# Check Docker version
docker --version

# Test Docker run
docker run --rm hello-world
```

### Rust Verification
```bash
# Check Rust version
rustc --version

# Check Cargo version
cargo --version

# Check Rustup version
rustup --version

# Update Rust toolchain
rustup update stable
```

### PATH Configuration
```bash
# Check current PATH
echo $PATH

# Check if cargo is in PATH
which cargo

# Check cargo config directory
ls -la ~/.cargo
```

### File Permissions
```bash
# Check directory permissions
ls -ld ~/.cargo/bin

# Check binary permissions
ls -lh ~/.cargo/bin/switchboard

# Fix ownership (if needed)
sudo chown -R $USER ~/.cargo
```

### Test Script Operations
```bash
# Run test script
./scripts/test-platform-compatibility.sh

# Run with debug output
bash -x ./scripts/test-platform-compatibility.sh

# Check script permissions
ls -la scripts/test-platform-compatibility.sh

# Add execute permissions
chmod +x scripts/test-platform-compatibility.sh
```

---

## Appendix B: Test Script Flow Diagram

```
START
  │
  ├─> Detect Platform (OS, Architecture)
  │     ├─> macOS detected? → YES
  │     └─> Log platform info
  │
  ├─> Verify Prerequisites
  │     ├─> Rust installed?
  │     ├─> Cargo installed?
  │     └─> Docker running?
  │
  ├─> Run cargo install --path . --force
  │     ├─> SUCCESS → Continue
  │     └─> FAIL → Exit with error
  │
  ├─> Verify Binary Installation
  │     ├─> Binary exists at ~/.cargo/bin/switchboard?
  │     └─> Binary is executable?
  │
  ├─> Test switchboard --version
  │     ├─> SUCCESS → Continue
  │     └─> FAIL → Exit with error
  │
  ├─> Test switchboard validate
  │     ├─> SUCCESS → Continue
  │     └─> FAIL → Exit with error
  │
  ├─> Cleanup (cargo uninstall switchboard)
  │     └─> Remove binary
  │
  └─> Display Test Summary
        ├─> List all passed tests
        └─> Confirm platform compatibility

END
```

---

## Appendix C: Example Test Session

```
$ cd ~/switchboard
$ ./scripts/test-platform-compatibility.sh

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Switchboard Platform Compatibility Test
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
This script tests cargo install and switchboard functionality
on the current platform: macOS arm64

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Platform Information
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Operating System: macOS
Architecture:      arm64
OS Version:        14.2.1
Rust Version:      rustc 1.83.0
Cargo Version:     cargo 1.83.0
Working Directory: /Users/johnsmith/switchboard
✓ Platform detection complete

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Testing cargo install
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ℹ Installing switchboard from local source...
    Compiling switchboard v0.1.0 (/Users/johnsmith/switchboard)
    Finished release [optimized] target(s) in 4.12s
    Replacing /Users/johnsmith/.cargo/bin/switchboard
✓ cargo install completed successfully

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Verifying Binary Installation
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✓ Binary found at: /Users/johnsmith/.cargo/bin/switchboard
✓ Binary is executable

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Testing --version Flag
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ℹ Running: switchboard --version
switchboard 0.1.0
✓ switchboard --version works correctly

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Testing validate Command
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ℹ Running: switchboard validate
Configuration is valid
✓ switchboard validate works correctly

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Cleanup
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ℹ Removing installed binary: /Users/johnsmith/.cargo/bin/switchboard
✓ Binary removed successfully

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Test Summary
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✓ All platform compatibility tests passed!

Tests performed:
  ✓ Platform detection and logging
  ✓ cargo install --path .
  ✓ Binary installation verification
  ✓ switchboard --version
  ✓ switchboard validate
  ✓ Binary cleanup

✓ Platform: macOS arm64 is compatible!
```

---

**Document Version:** 1.0  
**Last Updated:** 2026-02-16  
**Maintained By:** Switchboard Development Team
