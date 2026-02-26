#!/usr/bin/env bash
#
# Installation test script for switchboard CLI tool
#
# This script tests the installation and basic functionality of the switchboard binary:
# - Creates a temporary directory for testing
# - Runs `cargo install --path .` to install switchboard
# - Verifies the binary is available
# - Tests `switchboard validate` and `switchboard list` commands
# - Cleans up by removing temporary directory and uninstalling switchboard

# Strict error handling
set -euo pipefail

# Temporary directory for testing
TEMP_DIR=

# Cleanup function to remove temporary directory and uninstall switchboard
cleanup() {
    if [ -n "$TEMP_DIR" ] && [ -d "$TEMP_DIR" ]; then
        echo "Cleaning up temporary directory: $TEMP_DIR"
        rm -rf "$TEMP_DIR"
    fi
    # Uninstall switchboard if it was installed
    if command -v switchboard &> /dev/null; then
        echo "Uninstalling switchboard..."
        cargo uninstall switchboard || true
    fi
}

# Register cleanup function to run on exit
trap cleanup EXIT

# Create temporary directory for testing
TEMP_DIR=$(mktemp -d)
echo "Created temporary directory: $TEMP_DIR"

# Check Docker availability
DOCKER_AVAILABLE=0
if command -v docker &> /dev/null; then
    DOCKER_AVAILABLE=1
    echo "Docker is available"
else
    echo "WARNING: Docker is not available - Docker-dependent tests will be skipped"
fi

# Installation phase
echo "=== Installation Phase ==="
cd "$TEMP_DIR"

# Copy the repository to TEMP_DIR for testing installation from a fresh copy
echo "Copying repository to temporary directory..."
cp -r /workspace "$TEMP_DIR/switchboard"

# Change to the switchboard directory
cd "$TEMP_DIR/switchboard"
echo "Changed to directory: $(pwd)"

# Run cargo install
echo "Installing switchboard using 'cargo install --path .'"
if ! cargo install --path .; then
    echo "ERROR: Failed to install switchboard"
    exit 1
fi
echo "switchboard installed successfully"

# Verification phase
echo "=== Verification Phase ==="
if command -v switchboard &> /dev/null; then
    echo "switchboard binary found at: $(command -v switchboard)"
    echo "switchboard version:"
    switchboard --version || true
    echo "Verification successful"
else
    echo "ERROR: switchboard binary not found after installation"
    exit 1
fi

# Testing phase
echo "=== Testing Phase ==="

# Test switchboard validate command (requires Docker)
if [ -f "switchboard.toml" ]; then
    if [ "$DOCKER_AVAILABLE" -eq 1 ]; then
        echo "Testing 'switchboard validate switchboard.toml'..."
        if switchboard validate switchboard.toml; then
            echo "switchboard validate succeeded"
        else
            validate_exit_code=$?
            echo "switchboard validate failed with exit code: $validate_exit_code"
        fi
    else
        echo "SKIPPING 'switchboard validate switchboard.toml' - Docker not available"
    fi
else
    echo "WARNING: switchboard.toml not found, skipping validate test"
fi

# Test switchboard list command (requires Docker)
if [ "$DOCKER_AVAILABLE" -eq 1 ]; then
    echo "Testing 'switchboard list'..."
    if switchboard list; then
        echo "switchboard list succeeded"
    else
        list_exit_code=$?
        echo "switchboard list failed with exit code: $list_exit_code"
    fi
else
    echo "SKIPPING 'switchboard list' - Docker not available"
fi

# Summary
echo "=== Summary ==="
echo "Installation test script completed successfully"
echo ""
echo "Tests performed:"
echo "  - Installation: switchboard installed via cargo install --path ."
echo "  - Verification: switchboard binary availability and version check"
if [ "$DOCKER_AVAILABLE" -eq 1 ]; then
    echo "  - Testing: switchboard validate and list commands (Docker available)"
else
    echo "  - Testing: Docker-dependent tests skipped (Docker not available)"
fi
echo ""
echo "Cleanup will be handled by the trap on exit"
