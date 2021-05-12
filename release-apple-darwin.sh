#!/usr/bin/env bash

# example: ./release-apple-darwin.sh 0.7.1

set -euo pipefail

ver=$1
targetdir="target/x86_64-apple-darwin/release"
rm -rf $targetdir/ateam
cargo build --release --target x86_64-apple-darwin
strip $targetdir/ateam
tar czf $targetdir/ateam-$ver-x86_64-apple-darwin.tar.gz -C $targetdir ateam
echo $(sha256sum $targetdir/ateam-$ver-x86_64-apple-darwin.tar.gz)
