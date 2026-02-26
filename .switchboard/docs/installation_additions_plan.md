# Installation Documentation Additions Plan

**Date:** 2026-02-16  
**Purpose:** Plan for adding platform-specific requirements to INSTALLATION.md based on audit findings  
**Parent Task:** TODO2.md lines 80-83 - Document platform-specific requirements

---

## Overview

This plan identifies specific sections to add to INSTALLATION.md to address gaps identified in the installation documentation audit. The additions are organized by platform and priority, with insertion locations and source material references.

---

## Linux Requirements Additions

### 1. Kernel Version Requirements

**Priority:** High  
**Insertion Location:** After "Docker" subsection (line 371), before "User Permissions" subsection (line 372)  
**Insert Position:** After line 371, before `### User Permissions`

**Content Summary:**
- Minimum kernel version: 4.0+ recommended
- Tested version: Linux kernel 6.1+ on Debian GNU/Linux 12
- Requirements: cgroups support, namespaces support, containerization support

**Source Material:** PLATFORM_COMPATIBILITY.md lines 34-41

---

### 2. Docker API Features Required

**Priority:** Medium  
**Insertion Location:** Within "Docker" subsection, after existing Docker version requirements (line 340)  
**Insert Position:** After line 340, before `- **Docker Daemon:** Must be running and accessible`

**Content Summary:**
- Container lifecycle management (create, start, stop, remove)
- Volume mounting for workspace and configuration directories
- Container logs streaming
- Container status inspection

**Source Material:** PLATFORM_COMPATIBILITY.md lines 42-50

---

### 3. systemd Requirements

**Priority:** Low  
**Insertion Location:** After "User Permissions" subsection (line 381), before "System Packages" subsection (line 382)  
**Insert Position:** After line 381, before `### System Packages`

**Content Summary:**
- Docker service is managed via systemd on most modern Linux distributions
- Commands for starting and enabling Docker daemon

**Source Material:** PLATFORM_COMPATIBILITY.md lines 90-105

---

### 4. Unix-Specific Code Dependencies

**Priority:** Low  
**Insertion Location:** After "Supported Distributions" table (line 421), before "## macOS Requirements" section (line 425)  
**Insert Position:** After line 421 (after the note about Platform Compatibility link), before line 425

**Content Summary:**
- Signal handling: `tokio::signal::unix` for graceful shutdown
- Process checking: `libc::kill()` for process management
- Docker interaction: `bollard` library for Docker API (supports Docker 1.12+)

**Source Material:** PLATFORM_COMPATIBILITY.md lines 119-126

---

### 5. Networking and Filesystem Requirements

**Priority:** Low  
**Insertion Location:** After "Unix-Specific Code Dependencies" section (new addition above)  
**Insert Position:** After the Unix-Specific Code Dependencies section

**Content Summary:**
- Docker socket access details (`/var/run/docker.sock`)
- Socket permissions and docker group requirements
- Volume mounting requirements for workspace and configuration
- Read-only mount support

**Source Material:** PLATFORM_COMPATIBILITY.md lines 127-146

---

## macOS Requirements Additions

### 6. macOS Version Requirements

**Priority:** High  
**Insertion Location:** At the beginning of "macOS Requirements" section (line 425), after "If you are installing on macOS, ensure the following requirements are met before proceeding with installation."  
**Insert Position:** After line 427, before `### Docker Desktop` (line 429)

**Content Summary:**
- Minimum: macOS 10.15 (Catalina) or later
- Recommended: macOS 11 (Big Sur) or later
- Rationale: Docker Desktop 4.0+ compatibility

**Source Material:** PLATFORM_COMPATIBILITY.md lines 151-157

---

### 7. Xcode Command Line Tools

**Priority:** High  
**Insertion Location:** After "Docker Desktop" subsection (line 471), before "### Supported Architectures" subsection (line 460)  
**Insert Position:** After line 459, before `### Supported Architectures`

**Content Summary:**
- **Critical requirement** for Rust toolchain compilation
- Installation command: `xcode-select --install`
- Verification command: `xcode-select -p`
- Warning about linker errors if not installed

**Source Material:** PLATFORM_COMPATIBILITY.md lines 210-233

---

### 8. Rust Installation Methods

**Priority:** Medium  
**Insertion Location:** After "Xcode Command Line Tools" subsection (new addition above)  
**Insert Position:** After the Xcode Command Line Tools section

**Content Summary:**
- Method 1: rustup (recommended) with installation command
- Method 2: Homebrew (alternative) with installation command
- Note about auto-update differences

**Source Material:** PLATFORM_COMPATIBILITY.md lines 242-255

---

### 9. PATH Configuration Details

**Priority:** Medium  
**Insertion Location:** After "Rust Installation Methods" subsection (new addition above)  
**Insert Position:** After the Rust Installation Methods section

**Content Summary:**
- Cargo bin location: `~/.cargo/bin`
- Zsh configuration (default on macOS Catalina+)
- Bash configuration (older macOS versions)
- Shell-specific configuration files (`~/.zshrc` vs `~/.bash_profile`)
- Verification with `which cargo`

**Source Material:** PLATFORM_COMPATIBILITY.md lines 256-294

---

### 10. File Permissions

**Priority:** Medium  
**Insertion Location:** After "PATH Configuration Details" subsection (new addition above)  
**Insert Position:** After the PATH Configuration Details section

**Content Summary:**
- Permission requirements for `~/.cargo/bin/`
- Security warning about not using sudo with cargo install
- Fixing directory ownership with `chown -R`

**Source Material:** PLATFORM_COMPATIBILITY.md lines 295-320

---

### 11. Architecture-Specific Considerations

**Priority:** Medium  
**Insertion Location:** Within "### Supported Architectures" subsection, after the table (line 467), before the note line (line 469)  
**Insert Position:** After line 467, before line 469

**Content Summary:**
- Rosetta 2 for x86_64 on Apple Silicon
- Installation command: `softwareupdate --install-rosetta`
- Native aarch64 recommendation for Apple Silicon
- Compilation target specification: `--target aarch64-apple-darwin`

**Source Material:** PLATFORM_COMPATIBILITY.md lines 321-347

---

### 12. Docker Desktop File Sharing

**Priority:** Low  
**Insertion Location:** After "Architecture-Specific Considerations" subsection (new addition above)  
**Insert Position:** After the Architecture-Specific Considerations section

**Content Summary:**
- Settings location: Docker Desktop → Settings → Resources → File sharing
- Verification that project directories are allowed

**Source Material:** PLATFORM_COMPATIBILITY.md lines 368-379

---

### 13. Quick Setup Checklist

**Priority:** Low  
**Insertion Location:** After "Docker Desktop File Sharing" subsection (new addition above), before "## System Requirements" section (line 473)  
**Insert Position:** After the Docker Desktop File Sharing section, before line 473

**Content Summary:**
- macOS 10.15+ installed
- Docker Desktop 4.0+ installed and running
- Xcode Command Line Tools installed
- Rust toolchain installed (1.70.0+)
- `~/.cargo/bin` added to PATH
- Proper file permissions for `~/.cargo/bin`

**Source Material:** PLATFORM_COMPATIBILITY.md lines 382-392

---

### 14. Quick Setup Commands

**Priority:** Low  
**Insertion Location:** After "Quick Setup Checklist" subsection (new addition above)  
**Insert Position:** After the Quick Setup Checklist section

**Content Summary:**
- Consolidated command sequence for setup
- Xcode Command Line Tools installation
- Rust installation via rustup
- PATH configuration
- Installation and verification

**Source Material:** PLATFORM_COMPATIBILITY.md lines 393-422

---

## System Requirements Additions

### 15. Detailed RAM Requirements Table

**Priority:** High  
**Insertion Location:** Within "### Minimum RAM" subsection, after the existing RAM requirements text (line 488), before "### Minimum Disk Space" subsection (line 490)  
**Insert Position:** After line 488, before `### Minimum Disk Space`

**Content Summary:**
- RAM requirements table by usage scenario:
  - Single Agent: 2GB minimum, 4GB recommended
  - Multiple Agents (2-4): 4GB minimum, 8GB recommended
  - Heavy Workloads (5+ agents): 8GB minimum, 16GB+ recommended

**Source Material:** PLATFORM_COMPATIBILITY.md lines 428-435

---

### 16. RAM Usage Breakdown

**Priority:** Medium  
**Insertion Location:** Within "### Minimum RAM" subsection, after the RAM requirements table (new addition above)  
**Insert Position:** After the Detailed RAM Requirements Table section

**Content Summary:**
- switchboard binary: ~50-100MB
- Docker daemon: ~200-500MB
- Docker base image: ~50-100MB per container
- Kilo Code CLI runtime: ~100-300MB per container
- Additional tools: ~50-100MB per container

**Source Material:** PLATFORM_COMPATIBILITY.md lines 436-445

---

### 17. Detailed Disk Space Requirements Table

**Priority:** High  
**Insertion Location:** Within "### Minimum Disk Space" subsection, after existing disk space requirements (line 499), before "## Post-Installation Setup" section (line 501)  
**Insert Position:** After line 499, before `## Post-Installation Setup`

**Content Summary:**
- Disk space requirements table by component:
  - switchboard binary: 500MB minimum, 500MB recommended
  - Docker base image: ~200MB minimum, ~200MB recommended
  - Additional tools/packages: ~100MB minimum, ~500MB recommended
  - Cargo build cache: ~500MB minimum, ~1GB recommended
  - Agent logs: ~10-100MB minimum, ~100MB+ recommended
  - Workspace/project files: Varies by user
  - Total minimum (single agent): ~1.3GB minimum, ~2.3GB recommended

**Source Material:** PLATFORM_COMPATIBILITY.md lines 446-457

---

### 18. Disk Space Breakdown by Component

**Priority:** Medium  
**Insertion Location:** Within "### Minimum Disk Space" subsection, after the disk space requirements table (new addition above)  
**Insert Position:** After the Detailed Disk Space Requirements Table section

**Content Summary:**
- switchboard Binary: Size ~500MB, location, includes compiled binary
- Docker Images: Base image size, additional tools, runtime tools, total per image
- Build Artifacts: Cargo build cache, release binary, cleanup command
- Logs: Location, growth rate, monitoring recommendation

**Source Material:** PLATFORM_COMPATIBILITY.md lines 458-480

---

### 19. CPU Requirements Table

**Priority:** Medium  
**Insertion Location:** New subsection within "System Requirements" section, after "### Minimum Disk Space" subsection  
**Insert Position:** After line 499 (after all disk space content), before "## Post-Installation Setup" section (line 501)

**Content Summary:**
- CPU requirements table by usage scenario:
  - Single Agent: 2 cores minimum, 4+ cores recommended
  - Multiple Agents (2-4): 4 cores minimum, 6-8 cores recommended
  - Heavy Workloads (5+ agents): 6 cores minimum, 8+ cores recommended

**Source Material:** PLATFORM_COMPATIBILITY.md lines 481-488

---

### 20. CPU Architecture Support Table

**Priority:** Low  
**Insertion Location:** After "CPU Requirements Table" subsection (new addition above)  
**Insert Position:** After the CPU Requirements Table section

**Content Summary:**
- x86_64: Supported, tested on Debian GNU/Linux 12
- aarch64: Supported, code audit verified, testing pending

**Source Material:** PLATFORM_COMPATIBILITY.md lines 489-495

---

### 21. CPU Performance Notes

**Priority:** Low  
**Insertion Location:** After "CPU Architecture Support Table" subsection (new addition above)  
**Insert Position:** After the CPU Architecture Support Table section

**Content Summary:**
- Container isolation and core utilization
- Multi-core benefits for concurrent agents

**Source Material:** PLATFORM_COMPATIBILITY.md lines 496-505

---

## Summary of Additions by Priority

### High Priority (5 additions)
1. Linux Kernel Version Requirements
2. macOS Version Requirements
3. Xcode Command Line Tools
4. Detailed RAM Requirements Table
5. Detailed Disk Space Requirements Table

### Medium Priority (8 additions)
6. Docker API Features Required
7. Rust Installation Methods
8. PATH Configuration Details
9. File Permissions
10. Architecture-Specific Considerations
11. RAM Usage Breakdown
12. Disk Space Breakdown by Component
13. CPU Requirements Table

### Low Priority (8 additions)
14. systemd Requirements
15. Unix-Specific Code Dependencies
16. Networking and Filesystem Requirements
17. Docker Desktop File Sharing
18. Quick Setup Checklist
19. Quick Setup Commands
20. CPU Architecture Support Table
21. CPU Performance Notes

---

## Implementation Notes

- **Total additions:** 21 new sections/subsections
- **Estimated impact:** Adds approximately 100-150 lines to INSTALLATION.md
- **Sections requiring line references:** All insertion positions are documented above
- **Consistency notes:** Ensure all new subsections follow the same heading style and formatting as existing sections
- **Cross-references:** Maintain links to PLATFORM_COMPATIBILITY.md where appropriate
- **Priority approach:** Implement High Priority additions first, then Medium, then Low

---

## Parent Task Alignment

The additions in this plan align with TODO2.md lines 80-83:

- [x] **Linux: package dependencies (Docker)** - Covered by additions 1, 2, 3, 4, 5
- [x] **macOS: Docker Desktop requirement** - Covered by additions 6, 7, 8, 9, 10, 11, 12, 13, 14
- [x] **System requirements (minimum RAM, disk space)** - Covered by additions 15, 16, 17, 18, 19, 20, 21
