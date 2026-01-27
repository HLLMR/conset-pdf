#!/bin/bash
set -e
cargo fmt --check || (echo "Run 'cargo fmt' to fix formatting" && exit 1)
cargo clippy -- -D warnings || exit 1
cargo test --lib || exit 1
echo "Pre-commit checks passed ✓"
