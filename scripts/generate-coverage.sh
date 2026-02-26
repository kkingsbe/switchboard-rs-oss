#!/bin/bash
# Generate code coverage reports for the Switchboard project
# Supports generating HTML and LCOV coverage reports

set -e

# Default behavior
GENERATE_HTML=false
GENERATE_LCOV=false
SHOW_HELP=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --html)
            GENERATE_HTML=true
            shift
            ;;
        --lcov)
            GENERATE_LCOV=true
            shift
            ;;
        --all)
            GENERATE_HTML=true
            GENERATE_LCOV=true
            shift
            ;;
        --help|-h)
            SHOW_HELP=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            SHOW_HELP=true
            shift
            ;;
    esac
done

# Show help message
if [ "$SHOW_HELP" = true ]; then
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Generate code coverage reports using cargo-llvm-cov."
    echo "By default, generates both HTML and LCOV reports."
    echo ""
    echo "Options:"
    echo "  --html          Generate HTML coverage report only"
    echo "  --lcov          Generate LCOV coverage report only"
    echo "  --all           Generate both HTML and LCOV reports (default)"
    echo "  --help, -h      Show this help message"
    echo ""
    echo "Output Directory:"
    echo "  HTML Report:    target/llvm-cov/html/index.html"
    echo "  LCOV Report:    target/llvm-cov/lcov.info"
    echo ""
    echo "Examples:"
    echo "  $0               # Generate both reports"
    echo "  $0 --html       # Generate HTML only"
    echo "  $0 --lcov       # Generate LCOV only"
    exit 0
fi

# Default to generating both formats if no arguments provided
if [ "$GENERATE_HTML" = false ] && [ "$GENERATE_LCOV" = false ]; then
    GENERATE_HTML=true
    GENERATE_LCOV=true
fi

# Output directory from .cargo/config.toml
OUTPUT_DIR="target/llvm-cov"

echo "=========================================="
echo "Generating Coverage Reports"
echo "=========================================="

# Clean previous coverage data
echo ""
echo "Cleaning previous coverage data..."
cargo llvm-cov clean 2>/dev/null || true

# Run tests and collect coverage data
echo ""
echo "Running tests and collecting coverage data..."
# Use --ignore-run-fail to run all tests regardless of failure
# --workspace ensures we cover the entire workspace
cargo llvm-cov --no-report --workspace --ignore-run-fail

# Generate coverage reports based on requested formats
if [ "$GENERATE_HTML" = true ] && [ "$GENERATE_LCOV" = true ]; then
    echo ""
    echo "Generating HTML and LCOV reports..."
    # Generate HTML report
    cargo llvm-cov report --html
    # Generate LCOV report
    cargo llvm-cov report --lcov --output-path "$OUTPUT_DIR/lcov.info"
elif [ "$GENERATE_HTML" = true ]; then
    echo ""
    echo "Generating HTML report..."
    cargo llvm-cov report --html
elif [ "$GENERATE_LCOV" = true ]; then
    echo ""
    echo "Generating LCOV report..."
    # Ensure output directory exists
    mkdir -p "$OUTPUT_DIR"
    cargo llvm-cov report --lcov --output-path "$OUTPUT_DIR/lcov.info"
fi

# Check if reports were generated successfully
if [ "$GENERATE_HTML" = true ]; then
    HTML_REPORT="$OUTPUT_DIR/html/index.html"
    if [ -f "$HTML_REPORT" ]; then
        echo ""
        echo "✓ HTML report generated successfully: $HTML_REPORT"
        echo "  Open with: file://$(pwd)/$HTML_REPORT"
    else
        echo ""
        echo "✗ Failed to generate HTML report"
        exit 1
    fi
fi

if [ "$GENERATE_LCOV" = true ]; then
    LCOV_REPORT="$OUTPUT_DIR/lcov.info"
    if [ -f "$LCOV_REPORT" ]; then
        echo ""
        echo "✓ LCOV report generated successfully: $LCOV_REPORT"
    else
        echo ""
        echo "✗ Failed to generate LCOV report"
        exit 1
    fi
fi

echo ""
echo "=========================================="
echo "Coverage report generation complete!"
echo "=========================================="
echo ""
