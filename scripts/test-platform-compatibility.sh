#!/bin/bash
#
# Platform Compatibility Test Script for Switchboard
# 
# This script tests that switchboard can be installed via cargo install
# and works correctly on the current platform.
#
# Supported platforms:
#   - Linux (x86_64)
#   - macOS (x86_64)
#   - macOS (aarch64/Apple Silicon)
#
# Exit codes:
#   - 0: All tests passed
#   - 1: Any test failed
#
# Usage: ./scripts/test-platform-compatibility.sh

set -euo pipefail  # Exit on error, undefined variables, and pipe failures

# ============================================================================
# COLORS FOR OUTPUT
# ============================================================================
readonly GREEN='\033[0;32m'
readonly RED='\033[0;31m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m' # No Color

# ============================================================================
# HELPER FUNCTIONS
# ============================================================================

# Print a success message
success_msg() {
    echo -e "${GREEN}✓ $1${NC}"
}

# Print an error message
error_msg() {
    echo -e "${RED}✗ $1${NC}"
}

# Print an info message
info_msg() {
    echo -e "${BLUE}ℹ $1${NC}"
}

# Print a section header
section_header() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# ============================================================================
# PLATFORM DETECTION
# ============================================================================

# Detect the operating system
detect_os() {
    case "$(uname -s)" in
        Linux*)
            echo "Linux"
            ;;
        Darwin*)
            echo "macOS"
            ;;
        *)
            echo "Unknown"
            ;;
    esac
}

# Detect the system architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64)
            echo "x86_64"
            ;;
        aarch64|arm64)
            echo "aarch64"
            ;;
        *)
            echo "Unknown"
            ;;
    esac
}

# Get OS version details
get_os_version() {
    local os_name
    os_name="$(detect_os)"
    
    case "$os_name" in
        Linux)
            if command_exists lsb_release; then
                lsb_release -a 2>/dev/null | grep "Description" | cut -f2-
            elif [ -f /etc/os-release ]; then
                grep "PRETTY_NAME" /etc/os-release | cut -d'"' -f2
            else
                echo "Linux (version unknown)"
            fi
            ;;
        macOS)
            sw_vers -productVersion 2>/dev/null || echo "macOS (version unknown)"
            ;;
        *)
            echo "Unknown OS"
            ;;
    esac
}

# ============================================================================
# PLATFORM INFORMATION LOGGING
# ============================================================================

# Log detailed platform information
log_platform_info() {
    section_header "Platform Information"
    
    local os_name
    local arch_name
    local os_version
    
    os_name="$(detect_os)"
    arch_name="$(detect_arch)"
    os_version="$(get_os_version)"
    
    echo "Operating System: $os_name"
    echo "Architecture:      $arch_name"
    echo "OS Version:        $os_version"
    
    # Log Rust/Cargo versions
    if command_exists rustc; then
        echo "Rust Version:      $(rustc --version)"
    else
        error_msg "rustc not found!"
        return 1
    fi
    
    if command_exists cargo; then
        echo "Cargo Version:     $(cargo --version)"
    else
        error_msg "cargo not found!"
        return 1
    fi
    
    # Log current directory
    echo "Working Directory: $(pwd)"
    
    success_msg "Platform detection complete"
    return 0
}

# ============================================================================
# CARGO INSTALL TEST
# ============================================================================

# Run cargo install --path . to install switchboard from local source
test_cargo_install() {
    section_header "Testing cargo install"
    
    info_msg "Installing switchboard from local source..."
    
    # Run cargo install with force flag to overwrite existing installation
    if cargo install --path . --force; then
        success_msg "cargo install completed successfully"
        return 0
    else
        error_msg "cargo install failed!"
        return 1
    fi
}

# ============================================================================
# BINARY VERIFICATION
# ============================================================================

# Verify that the switchboard binary is installed to the expected location
verify_binary_installed() {
    section_header "Verifying Binary Installation"
    
    local cargo_bin_dir
    local switchboard_binary
    
    # Get the cargo bin directory (use standard location)
    cargo_bin_dir="$HOME/.cargo/bin"
    switchboard_binary="$cargo_bin_dir/switchboard"
    
    if [ -f "$switchboard_binary" ]; then
        success_msg "Binary found at: $switchboard_binary"
        
        # Check if it's executable
        if [ -x "$switchboard_binary" ]; then
            success_msg "Binary is executable"
            return 0
        else
            error_msg "Binary exists but is not executable"
            return 1
        fi
    else
        error_msg "Binary not found at: $switchboard_binary"
        return 1
    fi
}

# ============================================================================
# FUNCTIONALITY TESTS
# ============================================================================

# Test the --version flag
test_version_flag() {
    section_header "Testing --version Flag"
    
    info_msg "Running: switchboard --version"
    
    if switchboard --version; then
        success_msg "switchboard --version works correctly"
        return 0
    else
        error_msg "switchboard --version failed!"
        return 1
    fi
}

# Test the validate command with switchboard.toml
test_validate_command() {
    section_header "Testing validate Command"
    
    # Check if switchboard.toml exists
    if [ ! -f "switchboard.toml" ]; then
        error_msg "switchboard.toml not found in current directory!"
        return 1
    fi
    
    info_msg "Running: switchboard validate"
    
    if switchboard validate; then
        success_msg "switchboard validate works correctly"
        return 0
    else
        error_msg "switchboard validate failed!"
        return 1
    fi
}

# ============================================================================
# CLEANUP
# ============================================================================

# Remove the installed switchboard binary
cleanup_binary() {
    section_header "Cleanup"
    
    local cargo_bin_dir
    local switchboard_binary
    
    # Get the cargo bin directory (use standard location)
    cargo_bin_dir="$HOME/.cargo/bin"
    switchboard_binary="$cargo_bin_dir/switchboard"
    
    if [ -f "$switchboard_binary" ]; then
        info_msg "Removing installed binary: $switchboard_binary"
        
        # Use cargo uninstall to properly remove the package
        if cargo uninstall switchboard 2>/dev/null; then
            success_msg "Binary removed successfully"
            return 0
        else
            # Fallback: remove the binary directly
            if rm -f "$switchboard_binary"; then
                success_msg "Binary removed (direct deletion)"
                return 0
            else
                error_msg "Failed to remove binary!"
                return 1
            fi
        fi
    else
        info_msg "No binary to remove (already clean)"
        return 0
    fi
}

# ============================================================================
# MAIN TEST EXECUTION
# ============================================================================

# Main function to run all tests
main() {
    local exit_code=0
    
    section_header "Switchboard Platform Compatibility Test"
    echo "This script tests cargo install and switchboard functionality"
    echo "on the current platform: $(detect_os) $(detect_arch)"
    
    # Step 1: Log platform information
    if ! log_platform_info; then
        error_msg "Platform detection failed!"
        exit 1
    fi
    
    # Step 2: Run cargo install
    if ! test_cargo_install; then
        error_msg "Cargo install test failed!"
        cleanup_binary
        exit 1
    fi
    
    # Step 3: Verify binary installation
    if ! verify_binary_installed; then
        error_msg "Binary verification failed!"
        cleanup_binary
        exit 1
    fi
    
    # Step 4: Test --version flag
    if ! test_version_flag; then
        error_msg "Version flag test failed!"
        cleanup_binary
        exit 1
    fi
    
    # Step 5: Test validate command
    if ! test_validate_command; then
        error_msg "Validate command test failed!"
        cleanup_binary
        exit 1
    fi
    
    # Step 6: Cleanup
    if ! cleanup_binary; then
        error_msg "Cleanup failed!"
        exit 1
    fi
    
    # All tests passed
    section_header "Test Summary"
    success_msg "All platform compatibility tests passed!"
    echo ""
    echo "Tests performed:"
    echo "  ✓ Platform detection and logging"
    echo "  ✓ cargo install --path ."
    echo "  ✓ Binary installation verification"
    echo "  ✓ switchboard --version"
    echo "  ✓ switchboard validate"
    echo "  ✓ Binary cleanup"
    echo ""
    success_msg "Platform: $(detect_os) $(detect_arch) is compatible!"
    
    exit 0
}

# ============================================================================
# SCRIPT ENTRY POINT
# ============================================================================

# Run main function
main "$@"
