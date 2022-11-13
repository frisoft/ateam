#!/usr/bin/env bash

echo "======================================"
echo "Update all cargo binary packages"
cargo install cargo-update
cargo install-update -a

echo "======================================"
echo "Find unused dependencies in Cargo.toml"
cargo install cargo-udeps --locked
cargo +nightly udeps

echo
echo "========================================"
echo "Find outdated dependencies in Cargo.toml"
echo "Run cargo update to keep the packages updated."
cargo install --locked cargo-outdated
cargo outdated
