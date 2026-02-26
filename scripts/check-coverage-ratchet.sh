#!/bin/bash
#
# check-coverage-ratchet.sh
#
# This script enforces a coverage ratchet, ensuring code coverage doesn't drop
# below a baseline threshold. It compares current coverage from coverage.json
# against a baseline and fails if coverage drops more than the allowed percentage.
#
# Usage: ./check-coverage-ratchet.sh [baseline_file] [max_allowed_drop]
#   baseline_file: Path to baseline JSON file (default: .github/coverage-baseline.json)
#   max_allowed_drop: Maximum allowed coverage drop percentage (default: 1.0)
#
# Exit codes:
#   0: Coverage is acceptable (or baseline was created)
#   1: Coverage drop exceeds threshold

set -euo pipefail

# Parse arguments with defaults
BASELINE_FILE="${1:-.github/coverage-baseline.json}"
MAX_ALLOWED_DROP="${2:-1.0}"
COVERAGE_FILE="coverage.json"

echo "Coverage Ratchet Check"
echo "======================="
echo "Baseline file: $BASELINE_FILE"
echo "Max allowed drop: $MAX_ALLOWED_DROP%"
echo ""

# Check if coverage.json exists
if [[ ! -f "$COVERAGE_FILE" ]]; then
  echo "Error: $COVERAGE_FILE not found in current directory"
  echo "Ensure coverage has been generated with 'cargo llvm-cov --json --output-path coverage.json'"
  exit 1
fi

# Extract current coverage from coverage.json
# Format: {"data":[{"totals":{"lines":{"percent":"XX.XX"},"branches":{"percent":"XX.XX"}}}]}
CURRENT_LINE_PERCENT=$(grep -o '"lines"[[:space:]]*:[[:space:]]*{[[:space:]]*"percent"[[:space:]]*:[[:space:]]*"[0-9.]*"' "$COVERAGE_FILE" | grep -o '[0-9.]*$' | head -n 1)
CURRENT_BRANCH_PERCENT=$(grep -o '"branches"[[:space:]]*:[[:space:]]*{[[:space:]]*"percent"[[:space:]]*:[[:space:]]*"[0-9.]*"' "$COVERAGE_FILE" | grep -o '[0-9.]*$' | head -n 1)

# Validate we extracted coverage values
if [[ -z "$CURRENT_LINE_PERCENT" || -z "$CURRENT_BRANCH_PERCENT" ]]; then
  echo "Error: Failed to extract coverage values from $COVERAGE_FILE"
  echo "Current line percent: $CURRENT_LINE_PERCENT"
  echo "Current branch percent: $CURRENT_BRANCH_PERCENT"
  exit 1
fi

echo "Current Coverage:"
echo "  Line coverage: ${CURRENT_LINE_PERCENT}%"
echo "  Branch coverage: ${CURRENT_BRANCH_PERCENT}%"
echo ""

# Check if baseline file exists
if [[ ! -f "$BASELINE_FILE" ]]; then
  echo "Baseline file $BASELINE_FILE does not exist"
  echo "Creating baseline with current coverage values..."
  
  # Create baseline directory if needed
  mkdir -p "$(dirname "$BASELINE_FILE")"
  
  # Write baseline file
  cat > "$BASELINE_FILE" << EOF
{
  "line_percent": "${CURRENT_LINE_PERCENT}",
  "branch_percent": "${CURRENT_BRANCH_PERCENT}"
}
EOF
  
  echo "Baseline created at $BASELINE_FILE"
  echo "  Line baseline: ${CURRENT_LINE_PERCENT}%"
  echo "  Branch baseline: ${CURRENT_BRANCH_PERCENT}%"
  echo ""
  echo "PASS (baseline initialized)"
  exit 0
fi

# Extract baseline coverage values
# Format: {"line_percent":"XX.XX","branch_percent":"XX.XX"}
BASELINE_LINE_PERCENT=$(grep -o '"line_percent"[[:space:]]*:[[:space:]]*"[0-9.]*"' "$BASELINE_FILE" | grep -o '[0-9.]*$')
BASELINE_BRANCH_PERCENT=$(grep -o '"branch_percent"[[:space:]]*:[[:space:]]*"[0-9.9]*"' "$BASELINE_FILE" | grep -o '[0-9.]*$')

# Validate we extracted baseline values
if [[ -z "$BASELINE_LINE_PERCENT" || -z "$BASELINE_BRANCH_PERCENT" ]]; then
  echo "Error: Failed to extract baseline values from $BASELINE_FILE"
  echo "Baseline line percent: $BASELINE_LINE_PERCENT"
  echo "Baseline branch percent: $BASELINE_BRANCH_PERCENT"
  exit 1
fi

echo "Baseline Coverage:"
echo "  Line baseline: ${BASELINE_LINE_PERCENT}%"
echo "  Branch baseline: ${BASELINE_BRANCH_PERCENT}%"
echo ""

# Calculate deltas (using awk for floating-point arithmetic)
LINE_DELTA=$(awk "BEGIN {printf \"%.2f\", $CURRENT_LINE_PERCENT - $BASELINE_LINE_PERCENT}")
BRANCH_DELTA=$(awk "BEGIN {printf \"%.2f\", $CURRENT_BRANCH_PERCENT - $BASELINE_BRANCH_PERCENT}")

echo "Coverage Deltas:"
if awk "BEGIN {exit !($LINE_DELTA >= 0)}"; then
  echo "  Line coverage: +${LINE_DELTA}% (improvement)"
else
  echo "  Line coverage: ${LINE_DELTA}% (drop)"
fi

if awk "BEGIN {exit !($BRANCH_DELTA >= 0)}"; then
  echo "  Branch coverage: +${BRANCH_DELTA}% (improvement)"
else
  echo "  Branch coverage: ${BRANCH_DELTA}% (drop)"
fi
echo ""

# Check if deltas exceed the maximum allowed drop
FAIL=0
if awk "BEGIN {exit !($LINE_DELTA < -$MAX_ALLOWED_DROP)}"; then
  echo "FAIL: Line coverage dropped by ${LINE_DELTA}% (exceeds threshold of -${MAX_ALLOWED_DROP}%)"
  FAIL=1
fi

if awk "BEGIN {exit !($BRANCH_DELTA < -$MAX_ALLOWED_DROP)}"; then
  echo "FAIL: Branch coverage dropped by ${BRANCH_DELTA}% (exceeds threshold of -${MAX_ALLOWED_DROP}%)"
  FAIL=1
fi

if [[ $FAIL -eq 1 ]]; then
  echo ""
  echo "Coverage ratchet check FAILED"
  echo "Please improve test coverage to meet the baseline threshold"
  exit 1
fi

echo "PASS: Coverage is within acceptable threshold"
exit 0
