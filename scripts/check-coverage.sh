#!/bin/bash
# Script to check test coverage locally

set -e

echo "🔍 Checking test coverage..."

# Install tarpaulin if not already installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "📦 Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

# Run coverage analysis
echo "🧪 Running tests with coverage..."
cargo tarpaulin --verbose --all-features --workspace --timeout 120 --out Stdout --exclude-files "*/main.rs" "*/mcp_no_test.rs" --ignore-tests | tee coverage.txt

# Extract coverage percentage
COVERAGE=$(grep "coverage" coverage.txt | grep -oP '\d+\.\d+(?=% coverage)' | tail -1)
echo ""
echo "📊 Code coverage: $COVERAGE%"

# Check if coverage is 100%
if (( $(echo "$COVERAGE < 100" | bc -l) )); then
    echo "❌ Coverage is below 100% ($COVERAGE%). Please add more tests!"
    exit 1
else
    echo "✅ Coverage is 100%!"
fi

# Clean up
rm -f coverage.txt