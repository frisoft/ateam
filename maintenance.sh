#!/usr/bin/env bash
set -euo pipefail

echo "======================================"
echo "Find unused dependencies in Cargo.toml"
cargo machete .

echo
echo "========================================"
echo "Find outdated dependencies in Cargo.toml"
echo "Run cargo update to keep the packages updated."
cargo outdated

echo
echo "========================================"
echo "Audit..."
cargo audit
