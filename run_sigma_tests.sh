#!/bin/bash
# SigmaOS Kernel Test Runner
# Runs all verification commands as required by AGENTS.md

set -e

echo "=== SigmaOS Kernel Verification ==="
echo "Started at: $(date)"
echo "========================================"

echo "Step 1: cargo fmt --check"
cargo fmt --check
echo "✓ cargo fmt --check passed"

echo ""
echo "Step 2: cargo clippy --all-targets -- -D warnings"
cargo clippy --all-targets -- -D warnings
echo "✓ cargo clippy passed"

echo ""
echo "Step 3: cargo check --lib"
cargo check --lib
echo "✓ cargo check --lib passed"

echo ""
echo "Step 4: cargo test --lib"
cargo test --lib
echo "✓ cargo test --lib passed"

echo ""
echo "========================================"
echo "All verification steps completed successfully!"
echo "Completed at: $(date)"
