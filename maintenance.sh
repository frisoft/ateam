#!/usr/bin/env bash

echo "======================================"
echo "Find unused dependencies in Cargo.toml"
cargo install cargo-udeps --locked
cargo +nightly udeps

echo
echo "========================================"
echo "Find outdated dependencies in Cargo.toml"
cargo install --locked cargo-outdated
cargo outdated
