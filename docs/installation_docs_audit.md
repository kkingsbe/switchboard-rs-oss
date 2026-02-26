# Installation Documentation Audit

**Date:** 2026-02-16  
**Purpose:** Identify gaps in INSTALLATION.md and README.md regarding platform-specific requirements compared to PLATFORM_COMPATIBILITY.md

---

## Executive Summary

[`INSTALLATION.md`](docs/INSTALLATION.md) provides **good coverage** of platform-specific requirements, particularly Linux package dependencies and macOS Docker Desktop setup. However, several detailed technical specifications from [`PLATFORM_COMPATIBILITY.md`](docs/PLATFORM_COMPATIBILITY.md) are missing or could be enhanced.

[`README.md`](README.md) provides only **basic installation prerequisites** and directs users to INSTALLATION.md for detailed information, which is appropriate for a high-level overview document.

---

## Current Documentation State

### INSTALLATION.md - Platform Requirements Coverage

#### Linux Requirements (Lines 329-421)

**✅ Currently Documented:**
- Docker version requirement: Docker 20.10+
- Docker daemon running and accessible
- Socket access: `/var/run/docker.sock`
- Docker installation commands for Ubuntu/Debian, Fedora, Arch Linux
- Docker daemon start/enable commands with systemctl
- Docker verification with `docker ps`
- User permissions: adding user to `docker` group
- System packages: git, curl, build-essential, procps, file, sudo
- Package installation commands for all three distributions
- Supported distributions table (Debian 12, Ubuntu 20.04+, Fedora 35+, Arch Linux)

**❌ Missing from PLATFORM_COMPATIBILITY.md:**
- **Kernel version requirements:** Minimum kernel 4.0+ recommended, tested on kernel 6.1+
- **Docker API requirements:** Specific features needed (container lifecycle, volume mounting, log streaming, status inspection)
- **systemd requirements:** Explicit mention of systemd for Docker service management
- **Unix-specific code dependencies:** Signal handling (`tokio::signal::unix`), process checking (`libc::kill`), Docker API client (`bollard` library)
- **Networking and filesystem requirements:** Detailed explanation of Docker socket access and volume mounting requirements
- **cgroups and namespaces support:** Requirements for container isolation

#### macOS Requirements (Lines 425-472)

**✅ Currently Documented:**
- Docker Desktop version requirement: Docker Desktop 4.0+
- Docker Desktop installation steps (download .dmg, drag to Applications, launch)
- Docker Desktop verification with `docker info`
- Starting Docker Desktop instructions
- Supported architectures table: x86_64 (Intel) and aarch64 (Apple Silicon)
- Both architectures marked as code audit verified
- Reference to PLATFORM_COMPATIBILITY.md and MACOS_TESTING_PROCEDURE.md

**❌ Missing from PLATFORM_COMPATIBILITY.md:**
- **macOS version requirements:** Minimum macOS 10.15 (Catalina), recommended macOS 11 (Big Sur) or later
- **Xcode Command Line Tools:** Explicit requirement with installation command (`xcode-select --install`) and verification
- **Xcode Command Line Tools importance:** Warning about linker errors if not installed
- **Rust toolchain installation methods:** Both rustup and Homebrew methods documented
- **PATH configuration:** Detailed instructions for Zsh and Bash, shell-specific configuration files
- **File permissions:** macOS-specific permission issues and security considerations
- **Rosetta 2 compatibility:** For running x86_64 builds on Apple Silicon
- **Architecture-specific compilation:** Target specification commands for Apple Silicon
- **Unix-specific code dependencies:** Signal handling and process checking compatibility notes
- **Docker Desktop file sharing:** Settings for volume mounting permissions
- **Quick setup checklist:** Comprehensive pre-installation verification list
- **Quick setup commands:** Consolidated command sequence for macOS setup

#### System Requirements (Lines 473-500)

**✅ Currently Documented:**
- Minimum RAM: 2GB minimum, 4GB+ recommended for multiple concurrent agents
- Memory per agent: 200-500MB for base Node.js 22 runtime and Kilo Code CLI dependencies
- Factors increasing memory requirements (concurrent agents, large workspaces, long-running tasks)
- Production environment recommendation: 1GB additional RAM per agent
- Minimum disk space: 500MB for installation
- Additional space breakdown: Cargo build cache (500MB-1GB), Docker images (~500MB), agent logs (variable), workspace volumes (variable)
- Typical development workflow recommendation: 2GB additional disk space
- Log accumulation warning for disk-constrained environments

**❌ Missing from PLATFORM_COMPATIBILITY.md:**
- **Detailed RAM requirements table:** Different scenarios (single agent, multiple agents, heavy workloads) with minimum and recommended values
- **RAM usage breakdown:** Detailed breakdown per component (switchboard binary, Docker daemon, base image, Kilo Code CLI runtime, additional tools)
- **Detailed disk space requirements table:** Minimum vs recommended space for each component
- **Disk space breakdown by component:** Specific sizes for switchboard binary, Docker images, build artifacts, logs
- **CPU requirements table:** Minimum and recommended CPU cores for different scenarios
- **CPU architecture support table:** Current support status for x86_64 and aarch64
- **CPU performance notes:** How containers utilize cores, multi-core benefits for concurrent agents
- **Network requirements:** Any network-specific requirements (currently not in PLATFORM_COMPATIBILITY.md either)

---

### README.md - Platform Requirements Coverage

#### Installation Section (Lines 23-110)

**✅ Currently Documented:**
- Docker required and must be running
- Rust toolchain minimum version 1.70.0
- Git required for cloning repository
- Brief reference to Installation Guide for detailed instructions
- Platform-specific links to Docker installation pages
- Basic installation commands for all three installation methods
- Verification command (`switchboard --version`)
- Troubleshooting reference to INSTALLATION_TROUBLESHOOTING.md and PLATFORM_COMPATIBILITY.md

**❌ Missing (Intentionally - Appropriate for README):**
- README.md is appropriately high-level and defers to INSTALLATION.md for platform-specific details
- No Linux package dependencies listed
- No macOS Docker Desktop version requirements
- No system resource requirements (RAM, disk, CPU)
- No distribution-specific installation commands

**Note:** This is **acceptable and appropriate** for a README.md file, which should be concise. The README correctly redirects users to INSTALLATION.md for detailed platform requirements.

---

## Missing Platform Requirements Documentation

### Priority 1: Critical Technical Details

These are missing from INSTALLATION.md but would be valuable for users with technical questions or troubleshooting needs:

1. **Linux Kernel Version Requirements**
   - Minimum: Kernel 4.0+ recommended
   - Tested on: Debian GNU/Linux 12 with Linux kernel 6.1+
   - Requirements: cgroups support, namespaces support, containerization support
   - **Recommendation:** Add as a subsection in "Linux Requirements"

2. **Linux Docker API Features Required**
   - Container lifecycle management (create, start, stop, remove)
   - Volume mounting for workspace and configuration directories
   - Container logs streaming
   - Container status inspection
   - **Recommendation:** Add to "Docker" subsection in "Linux Requirements"

3. **macOS Version Requirements**
   - Minimum: macOS 10.15 (Catalina) or later
   - Recommended: macOS 11 (Big Sur) or later
   - Rationale: Docker Desktop 4.0+ compatibility
   - **Recommendation:** Add to "macOS Requirements" section

4. **macOS Xcode Command Line Tools**
   - **Critical requirement** for Rust toolchain compilation
   - Installation command: `xcode-select --install`
   - Verification command: `xcode-select -p`
   - Warning about linker errors if not installed
   - **Recommendation:** Add to "macOS Requirements" section (high priority)

### Priority 2: Enhanced Setup Instructions

These details improve the user experience by providing comprehensive setup guidance:

5. **macOS Rust Installation Methods**
   - Method 1: rustup (recommended) with installation command
   - Method 2: Homebrew (alternative) with installation command
   - Note about auto-update differences
   - **Recommendation:** Add to "macOS Requirements" section

6. **macOS PATH Configuration Details**
   - Cargo bin location: `~/.cargo/bin`
   - Zsh configuration (default on macOS Catalina+)
   - Bash configuration (older macOS versions)
   - Shell-specific configuration files (`~/.zshrc` vs `~/.bash_profile`)
   - Verification with `which cargo`
   - **Recommendation:** Add to "macOS Requirements" section

7. **macOS File Permissions and Security**
   - Permission requirements for `~/.cargo/bin/`
   - Security warning about not using sudo with cargo install
   - Fixing directory ownership with `chown -R`
   - **Recommendation:** Add to "macOS Requirements" section

8. **macOS Architecture-Specific Considerations**
   - Rosetta 2 for x86_64 on Apple Silicon
   - Installation command: `softwareupdate --install-rosetta`
   - Native aarch64 recommendation for Apple Silicon
   - Compilation target specification: `--target aarch64-apple-darwin`
   - **Recommendation:** Add to "macOS Requirements" section under "Supported Architectures"

9. **macOS Docker Desktop File Sharing**
   - Settings location: Docker Desktop → Settings → Resources → File sharing
   - Verification that project directories are allowed
   - **Recommendation:** Add to "macOS Requirements" section

### Priority 3: Comprehensive System Resources

These details provide more granular system requirement specifications:

10. **Detailed RAM Requirements Table**
    - Scenarios: Single Agent, Multiple Agents (2-4), Heavy Workloads (5+)
    - Minimum and recommended RAM for each scenario
    - **Recommendation:** Add to "System Requirements" section

11. **Detailed RAM Usage Breakdown**
    - switchboard binary: ~50-100MB
    - Docker daemon: ~200-500MB
    - Docker base image: ~50-100MB per container
    - Kilo Code CLI runtime: ~100-300MB per container
    - Additional tools: ~50-100MB per container
    - **Recommendation:** Add to "System Requirements" section

12. **Detailed Disk Space Requirements Table**
    - Minimum and recommended space per component
    - Components: switchboard binary, Docker base image, additional tools, Cargo build cache, agent logs, workspace files
    - **Recommendation:** Add to "System Requirements" section

13. **CPU Requirements Table**
    - Scenarios: Single Agent, Multiple Agents (2-4), Heavy Workloads (5+)
    - Minimum and recommended CPU cores
    - **Recommendation:** Add new "CPU Requirements" subsection to "System Requirements"

14. **CPU Architecture Support Table**
    - x86_64: Supported, tested on Debian GNU/Linux 12
    - aarch64: Supported, code audit verified, testing pending
    - **Recommendation:** Add to "System Requirements" section

15. **CPU Performance Notes**
    - Container isolation and core utilization
    - Multi-core benefits for concurrent agents
    - **Recommendation:** Add to "System Requirements" section

### Priority 4: Technical Dependencies

These details are valuable for advanced users and troubleshooting:

16. **Linux Unix-Specific Code Dependencies**
    - Signal handling: `tokio::signal::unix` for graceful shutdown
    - Process checking: `libc::kill()` for process management
    - Docker interaction: `bollard` library for Docker API (supports Docker 1.12+)
    - **Recommendation:** Add to "Linux Requirements" section

17. **Linux Networking and Filesystem Requirements**
    - Docker socket access details
    - Socket permissions and docker group requirements
    - Volume mounting requirements for workspace and configuration
    - Read-only mount support
    - **Recommendation:** Add to "Linux Requirements" section

18. **macOS Unix-Specific Code Dependencies**
    - Same as Linux, with compatibility notes for macOS
    - `tokio::signal::unix` compatible with macOS
    - `libc::kill()` compatible with macOS
    - `bollard` library works with Docker Desktop for macOS
    - **Recommendation:** Add to "macOS Requirements" section

19. **macOS Docker Desktop Integration Details**
    - Docker Desktop manages Docker daemon on macOS
    - bollard library connects to Docker Desktop automatically
    - Volume mounting permissions and file sharing settings
    - **Recommendation:** Add to "macOS Requirements" section

### Priority 5: Setup Checklists and Quick References

These improve user onboarding and setup verification:

20. **macOS Quick Setup Checklist**
    - macOS 10.15+ installed
    - Docker Desktop 4.0+ installed and running
    - Xcode Command Line Tools installed
    - Rust toolchain installed (1.70.0+)
    - `~/.cargo/bin` added to PATH
    - Proper file permissions for `~/.cargo/bin`
    - **Recommendation:** Add to "macOS Requirements" section

21. **macOS Quick Setup Commands**
    - Consolidated command sequence for setup
    - Xcode Command Line Tools installation
    - Rust installation via rustup
    - PATH configuration
    - Installation and verification
    - **Recommendation:** Add to "macOS Requirements" section

---

## Recommendations

### For INSTALLATION.md

#### High Priority Changes

1. **Add macOS version requirements** to the macOS Requirements section
   - Specify minimum macOS 10.15 (Catalina)
   - Recommend macOS 11 (Big Sur) or later

2. **Add Xcode Command Line Tools requirement** to macOS Requirements
   - This is a **critical** requirement that could cause installation failures if missing
   - Include installation command and verification
   - Explain the impact (linker errors without it)

3. **Add Linux kernel version requirements** to Linux Requirements
   - Specify minimum kernel 4.0+
   - Note cgroups and namespaces support requirements

4. **Add detailed system requirements tables** to System Requirements section
   - RAM requirements by usage scenario
   - Disk space requirements by component
   - CPU requirements by usage scenario

#### Medium Priority Changes

5. **Add macOS-specific setup instructions**
   - Rust installation methods (rustup vs Homebrew)
   - PATH configuration details (Zsh vs Bash)
   - File permissions and security considerations
   - Architecture-specific considerations (Rosetta 2)

6. **Add technical dependency details**
   - Unix-specific code dependencies (signal handling, process checking, bollard library)
   - Docker API feature requirements
   - Networking and filesystem requirements

7. **Add setup checklists and quick references**
   - macOS quick setup checklist
   - macOS quick setup commands
   - Similar Linux setup reference if beneficial

#### Low Priority Changes

8. **Enhance Docker integration details**
   - macOS Docker Desktop file sharing settings
   - Volume mounting permission details

9. **Add CPU architecture support table**
   - Status for x86_64 and aarch64
   - Testing status notes

### For README.md

**No changes recommended.** README.md appropriately provides a high-level overview and defers to INSTALLATION.md for detailed platform requirements. This is correct for a README document.

---

## Comparison Summary

| Topic | INSTALLATION.md | PLATFORM_COMPATIBILITY.md | Gap |
|-------|----------------|---------------------------|-----|
| **Linux Docker version** | ✅ 20.10+ | ✅ 20.10+ | None |
| **Linux Docker installation commands** | ✅ Yes | ✅ Yes | None |
| **Linux user permissions** | ✅ docker group | ✅ docker group | None |
| **Linux system packages** | ✅ git, curl, build-essential, procps, file, sudo | ✅ git, curl, build-essential, procps, file, sudo | None |
| **Linux supported distributions** | ✅ Debian, Ubuntu, Fedora, Arch | ✅ Debian, Ubuntu, Fedora, Arch | None |
| **Linux kernel version** | ❌ Missing | ✅ 4.0+ recommended, 6.1+ tested | ❌ Gap |
| **Linux Docker API features** | ❌ Missing | ✅ Detailed list | ❌ Gap |
| **Linux systemd requirements** | ❌ Implied only | ✅ Explicit | ❌ Gap |
| **Linux Unix dependencies** | ❌ Missing | ✅ tokio, libc, bollard | ❌ Gap |
| **Linux cgroups/namespaces** | ❌ Missing | ✅ Required | ❌ Gap |
| **macOS Docker Desktop version** | ✅ 4.0+ | ✅ 4.0+ | None |
| **macOS Docker Desktop install** | ✅ Yes | ✅ Yes | None |
| **macOS supported architectures** | ✅ x86_64, aarch64 | ✅ x86_64, aarch64 | None |
| **macOS version requirement** | ❌ Missing | ✅ 10.15+ | ❌ Gap |
| **macOS Xcode Command Line Tools** | ❌ Missing | ✅ Required | ❌ **Critical Gap** |
| **macOS Rust installation methods** | ❌ Missing | ✅ rustup, Homebrew | ❌ Gap |
| **macOS PATH configuration** | ❌ Missing | ✅ Zsh, Bash details | ❌ Gap |
| **macOS file permissions** | ❌ Missing | ✅ Detailed | ❌ Gap |
| **macOS architecture considerations** | ❌ Missing | ✅ Rosetta 2, native builds | ❌ Gap |
| **macOS Docker Desktop file sharing** | ❌ Missing | ✅ Settings location | ❌ Gap |
| **System RAM requirements** | ⚠️ Basic only | ✅ Detailed table | ❌ Gap |
| **System disk space** | ⚠️ Basic breakdown | ✅ Detailed table | ❌ Gap |
| **System CPU requirements** | ❌ Missing | ✅ Table by scenario | ❌ Gap |
| **System CPU architectures** | ❌ Missing | ✅ Table with status | ❌ Gap |
| **Quick setup checklists** | ❌ Missing | ✅ macOS checklist | ❌ Gap |
| **Quick setup commands** | ❌ Missing | ✅ macOS commands | ❌ Gap |

Legend: ✅ Present, ⚠️ Partial, ❌ Missing

---

## Conclusion

[`INSTALLATION.md`](docs/INSTALLATION.md) provides solid foundation documentation for platform-specific requirements but would benefit from incorporating several detailed technical specifications from [`PLATFORM_COMPATIBILITY.md`](docs/PLATFORM_COMPATIBILITY.md). The most critical gaps are:

1. **macOS version requirements** - Users need to know if their macOS version is supported
2. **Xcode Command Line Tools requirement** - **Critical** for Rust compilation
3. **Detailed system requirements tables** - RAM, disk, CPU by usage scenario

The moderate priority gaps include comprehensive macOS setup instructions and technical dependency details, which would improve user experience and reduce installation friction.

The low priority gaps (Docker integration details, CPU architecture table) provide additional value for advanced users but are not essential for basic installation.

**Overall Assessment:** INSTALLATION.md is well-structured and covers the essentials. The identified gaps represent opportunities to enhance the documentation to be more comprehensive and user-friendly, particularly for macOS users.
